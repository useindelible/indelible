use std::sync::Arc;
use std::time::Duration;

use apalis::prelude::*;
use apalis_postgres::{Config as ApalisConfig, PostgresStorage};
use ind_application::repos::embedding_backfill::EmbeddingBackfillRepository;
use ind_application::repos::search_reindex::SearchReindexRepository;
use ind_domain::GenericJobEnvelope;
use ind_persistence::repos::{
    PgEmbeddingBackfillRepository, PgOutboxHandoff, PgSearchReindexRepository,
};
use secrecy::{ExposeSecret, SecretString};

use crate::concurrency::ConcurrencyLimiter;
use crate::config::WorkerConfig;
use crate::context::{NotionRateLimiterRegistry, WorkerContext};
use crate::renderer_client::HttpRendererClient;
use crate::repositories::Repositories;
use crate::{auto_heal, failure, jobs, providers, relay, schedulers, shutdown};

const APALIS_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const APALIS_ORPHAN_RECLAIM_AFTER: Duration = Duration::from_secs(5 * 60);

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = WorkerConfig::load()?;
    ind_observability::init_tracing(&config.server.environment, &config.server.log_level);

    tracing::info!(environment = %config.server.environment, "starting ind-worker");

    // Fail closed at boot: in production a set-but-invalid credential key must
    // abort startup rather than silently disabling integration token decryption
    // and surfacing as failing Notion jobs later.
    validate_credential_key(
        &config.server.environment,
        config.auth.credential_key.as_ref(),
    )?;

    let pool = ind_persistence::create_pool(config.database_url.expose_secret()).await?;
    ind_persistence::run_migrations(&pool).await?;

    let apalis_poll_strategy = StrategyBuilder::new()
        .apply(IntervalStrategy::new(Duration::from_secs(1)))
        .build();
    let apalis_config = ApalisConfig::new(std::any::type_name::<GenericJobEnvelope>())
        .set_buffer_size(config.worker.claim_buffer_size)
        .set_keep_alive(APALIS_HEARTBEAT_INTERVAL)
        .set_reenqueue_orphaned_after(APALIS_ORPHAN_RECLAIM_AFTER)
        .with_poll_interval(apalis_poll_strategy);
    let handoff = Arc::new(PgOutboxHandoff::new(pool.clone(), apalis_config.clone()));
    let repos = Repositories::new(&pool);
    let pg_embedding_backfill_repo = Arc::new(PgEmbeddingBackfillRepository::new(pool.clone()));
    let reconciled_jobs = pg_embedding_backfill_repo
        .reconcile_platform_defaults(&config.mila)
        .await?;
    if reconciled_jobs > 0 {
        tracing::info!(
            queued_jobs = reconciled_jobs,
            embedding_model = %config.mila.embedding_model,
            embedding_dim = config.mila.embedding_dim,
            "queued Mila embedding reindex after platform default drift"
        );
    }
    let embedding_backfill_repo: Arc<dyn EmbeddingBackfillRepository> = pg_embedding_backfill_repo;
    let search_reindex_repo: Arc<dyn SearchReindexRepository> =
        Arc::new(PgSearchReindexRepository::new(pool.clone()));
    let search_reindex = search_reindex_repo
        .enqueue_full_reindex(
            250,
            Some(ind_search::SEARCH_INDEX_VERSION),
            chrono::Utc::now(),
        )
        .await?;
    if search_reindex.queued {
        tracing::info!(
            target_version = ind_search::SEARCH_INDEX_VERSION,
            "queued search reindex for index-version upgrade"
        );
    }
    let ctx = Arc::new(
        build_context(
            &config,
            pool.clone(),
            &repos,
            embedding_backfill_repo.clone(),
            search_reindex_repo,
        )
        .await?,
    );

    spawn_feed_source_entry_canonical_url_backfill(ctx.clone());

    let backend = PostgresStorage::<GenericJobEnvelope>::new_with_notify(&pool, &apalis_config);
    let handles = spawn_background_loops(ctx.clone(), &config, handoff);
    let worker = WorkerBuilder::new("content-pipeline")
        .backend(backend)
        .data(ctx)
        .concurrency(config.worker.max_concurrency)
        .build(failure::handle_job);

    tokio::select! {
        res = worker.run() => {
            if let Err(e) = res {
                tracing::error!(error = ?e, "worker exited with error");
            }
        }
        _ = shutdown::shutdown_signal() => {
            tracing::info!("shutdown signal received");
        }
    }

    handles.abort_all();
    tracing::info!("ind-worker shut down");
    Ok(())
}

/// In production, a credential key that is present but cannot build a cipher is a
/// configuration error: refuse to boot rather than run a worker that fails every
/// integration-decryption job. In non-production it degrades (warn) as before.
fn validate_credential_key(environment: &str, key: Option<&SecretString>) -> anyhow::Result<()> {
    if environment == "production"
        && let Some(key) = key
        && let Err(e) = ind_auth::CredentialCipher::from_base64(key.expose_secret())
    {
        anyhow::bail!("auth.credential_key is set but invalid: {e}");
    }
    Ok(())
}

async fn build_context(
    config: &WorkerConfig,
    pool: sqlx::PgPool,
    repos: &Repositories,
    embedding_backfill_repo: Arc<dyn EmbeddingBackfillRepository>,
    search_reindex_repo: Arc<dyn SearchReindexRepository>,
) -> anyhow::Result<WorkerContext> {
    let renderer = Arc::new(HttpRendererClient::new(&config.renderer_url));
    let object_storage = build_object_storage(config).await?;
    let credential_cipher = providers::build_credential_cipher(config);
    let startup_id = chrono::Utc::now().timestamp_micros();
    let worker_id = format!(
        "{}:{}:{}",
        config.server.hostname,
        std::process::id(),
        startup_id
    );

    #[expect(
        clippy::expect_used,
        reason = "webhook guarded client builds from a valid static egress policy; construction is infallible"
    )]
    let webhook_http = ind_integrations::webhook_delivery::build_webhook_http_client(
        config.webhook_egress_policy(),
    )
    .expect("webhook guarded client builds");

    Ok(crate::context::WorkerServicesBuilder::new(
        pool,
        renderer,
        object_storage,
        config.s3_bucket.clone(),
        config.mila.clone(),
        config.egress_policy(),
        credential_cipher,
    )?
    .with_worker_id(worker_id)
    .with_embedding_backfill_repo(embedding_backfill_repo)
    .with_search_reindex_repo(search_reindex_repo)
    .with_concurrency(
        ConcurrencyLimiter::new()
            .with_limit(
                ind_domain::job_types::FEED_PREPARE_DOCUMENT,
                config.capture.max_concurrency,
            )
            .with_limit(
                ind_domain::job_types::INTEGRATION_NOTION_EXPORT_DOCUMENT,
                config.integrations.notion.export_max_concurrency,
            )
            .with_limit(
                ind_domain::job_types::INTEGRATION_NOTION_SYNC_CONNECTION,
                config.integrations.notion.sync_max_concurrency,
            ),
    )
    .with_recovery_settings(&config.auto_heal)
    .with_feed_poll_schedule(schedulers::feed_poll_schedule(config))
    .with_email_ingest_provider_option(providers::build_email_ingest_provider(config))
    .with_integration_repositories(
        repos.integration_oauth_token.clone(),
        repos.integration_connection.clone(),
        repos.highlight.clone(),
    )
    .with_notion_job_deps_option(build_notion_job_deps(config, repos))
    .with_webhook_http(webhook_http)
    .build())
}

async fn build_object_storage(
    config: &WorkerConfig,
) -> anyhow::Result<Option<Arc<dyn ind_application::storage::ObjectStorage>>> {
    if !config.s3_enabled {
        return Ok(None);
    }

    let s3 = ind_persistence::storage::S3Client::from_config(config.s3_config()?);
    Ok(Some(Arc::new(s3)))
}

fn build_notion_job_deps(
    config: &WorkerConfig,
    repos: &Repositories,
) -> Option<Arc<crate::context::NotionJobDeps>> {
    let key = config.auth.credential_key.as_ref()?;
    match ind_auth::CredentialCipher::from_base64(key.expose_secret()) {
        Ok(cipher) => Some(Arc::new(crate::context::NotionJobDeps {
            connection_repo: repos.integration_connection.clone(),
            oauth_token_repo: repos.integration_oauth_token.clone(),
            export_cursor_repo: repos.export_cursor.clone(),
            highlight_repo: repos.highlight.clone(),
            tag_repo: repos.tag.clone(),
            document_repo: repos.document.clone(),
            library_repo: repos.library.clone(),
            outbox_repo: repos.job_outbox.clone(),
            cipher: Arc::new(cipher),
            rate_limiters: Arc::new(NotionRateLimiterRegistry::new(3.0)),
            notion_api_base: "https://api.notion.com".into(),
        })),
        Err(e) => {
            tracing::warn!(error = %e, "auth.credential_key is set but invalid; Notion export disabled");
            None
        }
    }
}

fn spawn_feed_source_entry_canonical_url_backfill(ctx: Arc<WorkerContext>) {
    // TASK-239: the backfill keyset-paginates to termination internally, so it runs ONCE here --
    // wrapping it in an until-zero loop would re-scan rows whose url never canonicalizes forever.
    tokio::spawn(async move {
        match jobs::backfill::run_feed_source_entry_canonical_url_backfill(
            ctx.feed_repo.as_ref(),
            100,
        )
        .await
        {
            Ok(stats) => tracing::info!(
                updated = stats.updated,
                skipped = stats.skipped,
                "feed canonical_url backfill finished"
            ),
            Err(err) => {
                tracing::error!(error = %err, "feed canonical_url backfill failed")
            }
        }
    });
}

fn spawn_background_loops(
    ctx: Arc<WorkerContext>,
    config: &WorkerConfig,
    handoff: Arc<PgOutboxHandoff>,
) -> WorkerHandles {
    let relay_config = config.clone();
    let relay_handle = tokio::spawn(async move {
        relay::run_relay(handoff, &relay_config).await;
    });

    let auto_heal_handle = if config.auto_heal.enabled {
        let auto_heal_ctx = Arc::new(ctx.recovery_jobs());
        Some(tokio::spawn(async move {
            auto_heal::run_auto_heal_loop(auto_heal_ctx).await;
        }))
    } else {
        tracing::info!("auto-heal disabled");
        None
    };

    let feed_scheduler_handle = if config.feed.enabled {
        let feed_ctx = Arc::new(ctx.feed_jobs());
        let feed_config = config.clone();
        Some(tokio::spawn(async move {
            schedulers::run_feed_scheduler_loop(feed_ctx, feed_config).await;
        }))
    } else {
        tracing::info!("feed scheduler disabled");
        None
    };

    let trash_cleanup_handle = if config.trash_cleanup.enabled {
        let cleanup_repo = ctx.library_repo.clone();
        let cleanup_settings = config.trash_cleanup.clone();
        Some(tokio::spawn(async move {
            jobs::trash_cleanup::run_trash_cleanup_loop(cleanup_repo, cleanup_settings).await;
        }))
    } else {
        tracing::info!("trash cleanup disabled");
        None
    };

    let retention_cleanup_handle = if config.feed_retention_cleanup.enabled {
        let retention_repo = ctx.retention_cleanup_repo.clone();
        let retention_settings = config.feed_retention_cleanup.clone();
        Some(tokio::spawn(async move {
            jobs::retention_cleanup::run_retention_cleanup_loop(retention_repo, retention_settings)
                .await;
        }))
    } else {
        tracing::info!("feed retention cleanup disabled");
        None
    };

    let notion_catch_up_handle = if config.integrations.notion.catch_up_enabled {
        let notion_job_deps = ctx.notion_job_deps.clone();
        let notion_config = config.clone();
        Some(tokio::spawn(async move {
            schedulers::run_notion_catch_up_loop(notion_job_deps, notion_config).await;
        }))
    } else {
        tracing::info!("notion catch-up scheduler disabled");
        None
    };

    let webhook_projector_handle = {
        let webhook_deps = ctx.webhook_jobs();
        tokio::spawn(async move {
            schedulers::run_webhook_projector_loop(webhook_deps).await;
        })
    };

    WorkerHandles {
        relay_handle,
        auto_heal_handle,
        feed_scheduler_handle,
        trash_cleanup_handle,
        retention_cleanup_handle,
        notion_catch_up_handle,
        webhook_projector_handle,
    }
}

struct WorkerHandles {
    relay_handle: tokio::task::JoinHandle<()>,
    auto_heal_handle: Option<tokio::task::JoinHandle<()>>,
    feed_scheduler_handle: Option<tokio::task::JoinHandle<()>>,
    trash_cleanup_handle: Option<tokio::task::JoinHandle<()>>,
    retention_cleanup_handle: Option<tokio::task::JoinHandle<()>>,
    notion_catch_up_handle: Option<tokio::task::JoinHandle<()>>,
    webhook_projector_handle: tokio::task::JoinHandle<()>,
}

impl WorkerHandles {
    fn abort_all(self) {
        self.relay_handle.abort();
        if let Some(handle) = self.auto_heal_handle {
            handle.abort();
        }
        if let Some(handle) = self.feed_scheduler_handle {
            handle.abort();
        }
        if let Some(handle) = self.trash_cleanup_handle {
            handle.abort();
        }
        if let Some(handle) = self.retention_cleanup_handle {
            handle.abort();
        }
        if let Some(handle) = self.notion_catch_up_handle {
            handle.abort();
        }
        self.webhook_projector_handle.abort();
    }
}
