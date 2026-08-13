use std::sync::Arc;

use ind_ai::{AiActionRunner, AiHttpClientConfig, EmbeddingIndexer, ReqwestAiProviderClient};
use ind_application::export_summary::ExportSummaryProvider;
use ind_application::handlers::feed::FeedPollScheduleConfig;
use ind_application::renderer::RendererClient;
use ind_application::repos::RetentionCleanupRepository;
use ind_application::repos::apalis_job::ApalisJobRepository;
use ind_application::repos::background_job_recovery::BackgroundJobRecoveryRepository;
use ind_application::repos::collection::CollectionRepository;
use ind_application::repos::dead_letter::DeadLetterRepository;
use ind_application::repos::document::DocumentRepository;
use ind_application::repos::document_asset::DocumentAssetRepository;
use ind_application::repos::document_lifecycle::DocumentLifecycle;
use ind_application::repos::document_reprocess::DocumentReprocessRepository;
use ind_application::repos::embedding_backfill::EmbeddingBackfillRepository;
use ind_application::repos::event::EventRepository;
use ind_application::repos::export_cursor::ExportCursorRepository;
use ind_application::repos::feed::FeedRepository;
use ind_application::repos::feed_delivery::FeedDeliveryRepository;
use ind_application::repos::highlight::HighlightRepository;
use ind_application::repos::import_job::ImportJobRepository;
use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_application::repos::integration_oauth_token::IntegrationOAuthTokenRepository;
use ind_application::repos::integrity::IntegrityStatsRepository;
use ind_application::repos::library::LibraryRepository;
use ind_application::repos::maintenance::MaintenanceTaskRepository;
use ind_application::repos::mila_config::MilaConfigRepository;
use ind_application::repos::outbox::JobOutboxRepository;
use ind_application::repos::search_reindex::SearchReindexRepository;
use ind_application::repos::tag::TagRepository;
use ind_application::repos::user_document_state::UserDocumentStateRepository;
use ind_application::repos::user_preferences::UserPreferencesRepository;
use ind_application::repos::webhook::WebhookRepository;
use sqlx::PgPool;

use ind_application::repos::email_ingest::EmailIngestLogRepository;
use ind_application::repos::email_sender::EmailSenderRepository;
use ind_application::repos::email_unsubscribe_target::EmailUnsubscribeTargetRepository;
use std::collections::HashMap;
use std::sync::Mutex;

use ind_application::services::tts::synthesis::TtsOrphanSweeper;
use ind_application::storage::ObjectStorage;
use ind_domain::IntegrationConnectionId;
use ind_ingest::AssetBackedPreparedContentProvider;
use ind_integrations::email::InboundEmailProvider;
use ind_integrations::notion::NotionRateLimiter;
use ind_persistence::repos::{
    PgAiOutputRepository, PgAiPromptPresetRepository, PgAiRunRepository, PgApalisJobRepository,
    PgBackgroundJobRecoveryRepository, PgCollectionRepository, PgContentVectorRepository,
    PgDeadLetterRepository, PgDocumentAssetRepository, PgDocumentLifecycle, PgDocumentRepository,
    PgDocumentReprocessRepository, PgEmailIngestLogRepository, PgEmailSenderRepository,
    PgEmailUnsubscribeTargetRepository, PgEmbeddingBackfillRepository, PgEntityRepository,
    PgEventRepository, PgFeedDeliveryRepository, PgFeedRepository, PgHighlightRepository,
    PgImportJobRepository, PgIntegrationConnectionRepository, PgIntegrationOAuthTokenRepository,
    PgIntegrityStatsRepository, PgJobOutboxRepository, PgLibraryRepository,
    PgMaintenanceTaskRepository, PgMilaConfigRepository, PgRetentionCleanupRepository,
    PgSearchReindexRepository, PgSearchRepository, PgTagRepository, PgTtsAudioAssetRepository,
    PgUserDocumentStateRepository, PgUserPreferencesRepository, PgUserRepository,
    PgWebhookRepository,
};
use ind_search::SearchIndexer;

use crate::concurrency::ConcurrencyLimiter;
use crate::config::AutoHealSettings;

mod job_deps;

pub use job_deps::{
    AiSearchJobDeps, CaptureJobDeps, EmailJobDeps, FeedJobDeps, IndexQueueContext,
    IntegrationJobDeps, RecoveryJobDeps, WebhookJobDeps,
};

// Notion's 3-RPS cap is per-integration token, not per-host. A single
// shared limiter would serialize unrelated users behind one bucket. The
// registry hands out a separate limiter per connection, lazily.
pub struct NotionRateLimiterRegistry {
    rate_per_second: f64,
    map: Mutex<HashMap<IntegrationConnectionId, Arc<NotionRateLimiter>>>,
}

impl NotionRateLimiterRegistry {
    pub fn new(rate_per_second: f64) -> Self {
        Self {
            rate_per_second,
            map: Mutex::new(HashMap::new()),
        }
    }

    pub fn for_connection(&self, connection_id: IntegrationConnectionId) -> Arc<NotionRateLimiter> {
        #[expect(
            clippy::expect_used,
            reason = "registry mutex is never held across an await; poisoning implies an already-fatal prior panic"
        )]
        let mut map = self.map.lock().expect("notion rate limiter mutex poisoned");
        map.entry(connection_id)
            .or_insert_with(|| Arc::new(NotionRateLimiter::new(self.rate_per_second)))
            .clone()
    }
}

pub struct NotionJobDeps {
    pub connection_repo: Arc<dyn IntegrationConnectionRepository>,
    pub oauth_token_repo: Arc<dyn IntegrationOAuthTokenRepository>,
    pub export_cursor_repo: Arc<dyn ExportCursorRepository>,
    pub highlight_repo: Arc<dyn HighlightRepository>,
    pub tag_repo: Arc<dyn TagRepository>,
    pub document_repo: Arc<dyn DocumentRepository>,
    pub library_repo: Arc<dyn LibraryRepository>,
    pub outbox_repo: Arc<dyn JobOutboxRepository>,
    pub cipher: Arc<ind_auth::CredentialCipher>,
    pub rate_limiters: Arc<NotionRateLimiterRegistry>,
    pub notion_api_base: String,
}

pub struct WorkerContext {
    pub renderer: Arc<dyn RendererClient>,
    pub dead_letter_repo: Arc<dyn DeadLetterRepository>,
    pub collection_repo: Arc<dyn CollectionRepository>,
    pub apalis_job_repo: Arc<dyn ApalisJobRepository>,
    pub embedding_backfill_repo: Arc<dyn EmbeddingBackfillRepository>,
    pub mila_platform_defaults: ind_domain::MilaPlatformDefaults,
    pub integrity_stats_repo: Arc<dyn IntegrityStatsRepository>,
    pub maintenance_task_repo: Arc<dyn MaintenanceTaskRepository>,
    pub tts_orphan_sweeper: Option<Arc<TtsOrphanSweeper>>,
    pub event_repo: Arc<dyn EventRepository>,
    pub feed_repo: Arc<dyn FeedRepository>,
    pub feed_delivery_repo: Arc<dyn FeedDeliveryRepository>,
    pub document_repo: Arc<dyn DocumentRepository>,
    pub document_asset_repo: Arc<dyn DocumentAssetRepository>,
    pub document_reprocess_repo: Arc<dyn DocumentReprocessRepository>,
    pub lifecycle: Arc<dyn DocumentLifecycle>,
    pub user_document_state_repo: Arc<dyn UserDocumentStateRepository>,
    pub library_repo: Arc<dyn LibraryRepository>,
    pub retention_cleanup_repo: Arc<dyn RetentionCleanupRepository>,
    pub user_preferences_repo: Arc<dyn UserPreferencesRepository>,
    pub outbox_repo: Arc<dyn JobOutboxRepository>,
    pub search_reindex_repo: Arc<dyn SearchReindexRepository>,
    pub search_indexer: Arc<SearchIndexer>,
    pub embedding_indexer: Arc<EmbeddingIndexer>,
    pub ai_action_runner: Arc<AiActionRunner>,
    pub export_summary_provider: Arc<dyn ExportSummaryProvider>,
    pub mila_config_repo: Arc<dyn MilaConfigRepository>,
    pub webhook_repo: Arc<dyn WebhookRepository>,
    pub object_storage: Option<Arc<dyn ObjectStorage>>,
    /// YouTube player API base URL; `None` resolves to `https://www.youtube.com`. Overridden in
    /// tests so document YouTube ingest can be exercised end-to-end against a mock server.
    pub youtube_player_base_url: Option<String>,
    pub pool: PgPool,
    pub concurrency: ConcurrencyLimiter,
    pub s3_bucket: String,
    pub worker_id: String,
    pub auto_heal_lease_secs: i64,
    pub maintenance_lease_secs: i64,
    pub auto_heal_interval_secs: u64,
    pub auto_heal_stale_after_secs: i64,
    pub auto_heal_batch_size: i64,
    pub embedding_repair_interval_secs: u64,
    pub integrity_interval_secs: u64,
    pub tts_orphan_interval_secs: u64,
    pub tts_orphan_page_size: i32,
    pub background_recovery_repo: Arc<dyn BackgroundJobRecoveryRepository>,
    pub job_recovery_max_attempts: i32,
    pub job_recovery_batch_size: i64,
    pub feed_poll_schedule: FeedPollScheduleConfig,
    pub email_ingest_provider: Option<Arc<dyn InboundEmailProvider>>,
    pub email_ingest_log_repo: Option<Arc<dyn EmailIngestLogRepository>>,
    pub email_sender_repo: Option<Arc<dyn EmailSenderRepository>>,
    pub email_unsubscribe_target_repo: Option<Arc<dyn EmailUnsubscribeTargetRepository>>,
    pub email_unsubscribe_url_policy: crate::jobs::email_unsubscribe::OneClickPolicy,
    pub user_repo: Option<Arc<dyn ind_application::repos::user::UserRepository>>,
    pub import_job_repo: Arc<dyn ImportJobRepository>,
    pub tag_repo: Arc<dyn TagRepository>,
    // Integration OAuth worker fields are optional; missing fields produce a clear error at job time.
    pub integration_oauth_token_repo: Option<Arc<dyn IntegrationOAuthTokenRepository>>,
    pub integration_connection_repo: Option<Arc<dyn IntegrationConnectionRepository>>,
    pub highlight_repo: Option<Arc<dyn HighlightRepository>>,
    pub credential_cipher: Option<Arc<ind_auth::CredentialCipher>>,
    pub notion_job_deps: Option<Arc<NotionJobDeps>>,
    /// SSRF policy for outbound fetches (feed polling, Readwise import).
    pub egress_policy: ind_egress::EgressPolicy,
    /// Hoisted guarded client for webhook delivery (built once; webhook-surface
    /// policy with redirects disabled).
    pub webhook_http: ind_egress::GuardedHttpClient,
}

pub struct WorkerServicesBuilder {
    context: WorkerContext,
}

impl WorkerServicesBuilder {
    pub fn new(
        pool: PgPool,
        renderer: Arc<dyn RendererClient>,
        object_storage: Option<Arc<dyn ObjectStorage>>,
        s3_bucket: String,
        mila_defaults: ind_domain::MilaPlatformDefaults,
        egress_policy: ind_egress::EgressPolicy,
        credential_cipher: Option<Arc<ind_auth::CredentialCipher>>,
    ) -> anyhow::Result<Self> {
        let document_repo: Arc<dyn DocumentRepository> =
            Arc::new(PgDocumentRepository::new(pool.clone()));
        let document_asset_repo: Arc<dyn DocumentAssetRepository> =
            Arc::new(PgDocumentAssetRepository::new(pool.clone()));
        let mila_config_repo: Arc<dyn MilaConfigRepository> =
            Arc::new(PgMilaConfigRepository::new(pool.clone()));
        let defaulting_mila_repo: Arc<dyn MilaConfigRepository> = Arc::new(
            ind_application::repos::mila_config::DefaultingMilaConfigRepository::new(
                mila_config_repo.clone(),
                mila_defaults.clone(),
            ),
        );
        let content_provider = Arc::new(AssetBackedPreparedContentProvider::new(
            document_repo.clone(),
            document_asset_repo.clone(),
            defaulting_mila_repo.clone(),
            object_storage.clone(),
        ));
        let ai_client = Arc::new(ReqwestAiProviderClient::new(
            AiHttpClientConfig::default(),
            egress_policy.clone(),
        )?);
        let search_indexer = Arc::new(SearchIndexer::new(
            document_repo.clone(),
            content_provider.clone(),
            Arc::new(PgSearchRepository::new(pool.clone())),
        ));
        let embedding_indexer = EmbeddingIndexer::new(
            content_provider.clone(),
            defaulting_mila_repo.clone(),
            Arc::new(PgContentVectorRepository::new(pool.clone())),
            document_repo.clone(),
            ai_client.clone(),
            mila_defaults.clone(),
        )
        .with_credential_cipher(credential_cipher.clone());
        let ai_action_runner = AiActionRunner::new(
            document_repo.clone(),
            content_provider,
            defaulting_mila_repo,
            Arc::new(PgAiPromptPresetRepository::new(pool.clone())),
            Arc::new(PgAiOutputRepository::new(pool.clone())),
            Arc::new(PgAiRunRepository::new(pool.clone())),
            Arc::new(PgEntityRepository::new(pool.clone())),
            ai_client,
        )
        .with_credential_cipher(credential_cipher.clone());
        let embedding_indexer = Arc::new(embedding_indexer);
        let ai_action_runner = Arc::new(ai_action_runner);
        let webhook_http =
            ind_integrations::webhook_delivery::build_webhook_http_client(egress_policy.clone())?;
        let tts_orphan_sweeper = object_storage.as_ref().map(|storage| {
            Arc::new(TtsOrphanSweeper::new(
                Arc::new(PgTtsAudioAssetRepository::new(pool.clone())),
                storage.clone(),
            ))
        });

        Ok(Self {
            context: WorkerContext {
                renderer,
                dead_letter_repo: Arc::new(PgDeadLetterRepository::new(pool.clone())),
                collection_repo: Arc::new(PgCollectionRepository::new(pool.clone())),
                apalis_job_repo: Arc::new(PgApalisJobRepository::new(pool.clone())),
                embedding_backfill_repo: Arc::new(PgEmbeddingBackfillRepository::new(pool.clone())),
                mila_platform_defaults: mila_defaults,
                integrity_stats_repo: Arc::new(PgIntegrityStatsRepository::new(pool.clone())),
                maintenance_task_repo: Arc::new(PgMaintenanceTaskRepository::new(pool.clone())),
                tts_orphan_sweeper,
                event_repo: Arc::new(PgEventRepository::new(pool.clone())),
                feed_repo: Arc::new(PgFeedRepository::new(pool.clone())),
                feed_delivery_repo: Arc::new(PgFeedDeliveryRepository::new(pool.clone())),
                document_repo,
                document_asset_repo,
                document_reprocess_repo: Arc::new(PgDocumentReprocessRepository::new(pool.clone())),
                lifecycle: Arc::new(PgDocumentLifecycle::new(pool.clone())),
                user_document_state_repo: Arc::new(PgUserDocumentStateRepository::new(
                    pool.clone(),
                )),
                library_repo: Arc::new(PgLibraryRepository::new(pool.clone())),
                retention_cleanup_repo: Arc::new(PgRetentionCleanupRepository::new(pool.clone())),
                user_preferences_repo: Arc::new(PgUserPreferencesRepository::new(pool.clone())),
                outbox_repo: Arc::new(PgJobOutboxRepository::new(pool.clone())),
                search_reindex_repo: Arc::new(PgSearchReindexRepository::new(pool.clone())),
                search_indexer,
                embedding_indexer,
                ai_action_runner,
                export_summary_provider: Arc::new(
                    ind_application::export_summary::StoredExportSummaryProvider::new(Arc::new(
                        PgAiOutputRepository::new(pool.clone()),
                    )),
                ),
                mila_config_repo,
                webhook_repo: Arc::new(PgWebhookRepository::new(pool.clone())),
                object_storage,
                youtube_player_base_url: None,
                pool: pool.clone(),
                concurrency: ConcurrencyLimiter::default(),
                s3_bucket,
                worker_id: "worker".to_string(),
                auto_heal_lease_secs: 30,
                maintenance_lease_secs: 300,
                auto_heal_interval_secs: 60,
                auto_heal_stale_after_secs: 300,
                auto_heal_batch_size: 10,
                embedding_repair_interval_secs: 900,
                integrity_interval_secs: 3_600,
                tts_orphan_interval_secs: 86_400,
                tts_orphan_page_size: 100,
                background_recovery_repo: Arc::new(PgBackgroundJobRecoveryRepository::new(
                    pool.clone(),
                )),
                job_recovery_max_attempts: 3,
                job_recovery_batch_size: 50,
                feed_poll_schedule: FeedPollScheduleConfig {
                    default_public_poll_interval_minutes: 60,
                    min_public_poll_interval_minutes: 15,
                }
                .normalized(),
                email_ingest_provider: None,
                email_ingest_log_repo: Some(Arc::new(PgEmailIngestLogRepository::new(
                    pool.clone(),
                ))),
                email_sender_repo: Some(Arc::new(PgEmailSenderRepository::new(pool.clone()))),
                email_unsubscribe_target_repo: Some(Arc::new(
                    PgEmailUnsubscribeTargetRepository::new(pool.clone()),
                )),
                email_unsubscribe_url_policy:
                    crate::jobs::email_unsubscribe::OneClickPolicy::strict(),
                user_repo: Some(Arc::new(PgUserRepository::new(pool.clone()))),
                import_job_repo: Arc::new(PgImportJobRepository::new(pool.clone())),
                tag_repo: Arc::new(PgTagRepository::new(pool.clone())),
                integration_oauth_token_repo: Some(Arc::new(
                    PgIntegrationOAuthTokenRepository::new(pool.clone()),
                )),
                integration_connection_repo: Some(Arc::new(
                    PgIntegrationConnectionRepository::new(pool.clone()),
                )),
                highlight_repo: Some(Arc::new(PgHighlightRepository::new(pool.clone()))),
                credential_cipher,
                notion_job_deps: None,
                egress_policy,
                webhook_http,
            },
        })
    }

    pub fn with_worker_id(mut self, worker_id: impl Into<String>) -> Self {
        self.context.worker_id = worker_id.into();
        self
    }

    pub fn from_context(context: WorkerContext) -> Self {
        Self { context }
    }

    pub fn with_concurrency(mut self, concurrency: ConcurrencyLimiter) -> Self {
        self.context.concurrency = concurrency;
        self
    }

    pub fn with_embedding_backfill_repo(
        mut self,
        repo: Arc<dyn EmbeddingBackfillRepository>,
    ) -> Self {
        self.context.embedding_backfill_repo = repo;
        self
    }

    pub fn with_search_reindex_repo(mut self, repo: Arc<dyn SearchReindexRepository>) -> Self {
        self.context.search_reindex_repo = repo;
        self
    }

    pub fn with_feed_poll_schedule(mut self, schedule: FeedPollScheduleConfig) -> Self {
        self.context.feed_poll_schedule = schedule;
        self
    }

    pub fn with_recovery_settings(mut self, settings: &AutoHealSettings) -> Self {
        self.context.auto_heal_lease_secs = settings.lease_secs;
        self.context.maintenance_lease_secs = settings.maintenance_lease_secs;
        self.context.auto_heal_interval_secs = settings.interval_secs;
        self.context.auto_heal_stale_after_secs = settings.stale_after_secs;
        self.context.auto_heal_batch_size = settings.batch_size;
        self.context.embedding_repair_interval_secs = settings.embedding_repair_interval_secs;
        self.context.integrity_interval_secs = settings.integrity_interval_secs;
        self.context.tts_orphan_interval_secs = settings.tts_orphan_interval_secs;
        self.context.tts_orphan_page_size = settings.tts_orphan_page_size;
        self.context.job_recovery_max_attempts = settings.job_recovery_max_attempts;
        self.context.job_recovery_batch_size = settings
            .job_recovery_batch_size
            .unwrap_or(settings.batch_size);
        self
    }

    pub fn with_email_ingest_provider_option(
        mut self,
        provider: Option<Arc<dyn InboundEmailProvider>>,
    ) -> Self {
        self.context.email_ingest_provider = provider;
        self
    }

    pub fn with_integration_repositories(
        mut self,
        oauth_tokens: Arc<dyn IntegrationOAuthTokenRepository>,
        connections: Arc<dyn IntegrationConnectionRepository>,
        highlights: Arc<dyn HighlightRepository>,
    ) -> Self {
        self.context.integration_oauth_token_repo = Some(oauth_tokens);
        self.context.integration_connection_repo = Some(connections);
        self.context.highlight_repo = Some(highlights);
        self
    }

    pub fn with_youtube_player_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.context.youtube_player_base_url = Some(base_url.into());
        self
    }

    pub fn without_object_storage(mut self) -> Self {
        self.context.object_storage = None;
        self.context.tts_orphan_sweeper = None;
        self
    }

    pub fn with_email_ingest_provider(mut self, provider: Arc<dyn InboundEmailProvider>) -> Self {
        self.context.email_ingest_provider = Some(provider);
        self
    }

    pub fn without_email_services(mut self) -> Self {
        self.context.email_ingest_provider = None;
        self.context.email_ingest_log_repo = None;
        self.context.email_sender_repo = None;
        self.context.email_unsubscribe_target_repo = None;
        self.context.user_repo = None;
        self
    }

    pub fn with_email_sender_repo(mut self, repo: Arc<dyn EmailSenderRepository>) -> Self {
        self.context.email_sender_repo = Some(repo);
        self
    }

    pub fn with_email_unsubscribe_target_repo(
        mut self,
        repo: Arc<dyn EmailUnsubscribeTargetRepository>,
    ) -> Self {
        self.context.email_unsubscribe_target_repo = Some(repo);
        self
    }

    pub fn with_email_unsubscribe_url_policy(
        mut self,
        policy: crate::jobs::email_unsubscribe::OneClickPolicy,
    ) -> Self {
        self.context.email_unsubscribe_url_policy = policy;
        self
    }

    pub fn with_notion_job_deps(mut self, deps: Arc<NotionJobDeps>) -> Self {
        self.context.notion_job_deps = Some(deps);
        self
    }

    pub fn with_notion_job_deps_option(mut self, deps: Option<Arc<NotionJobDeps>>) -> Self {
        self.context.notion_job_deps = deps;
        self
    }

    pub fn with_webhook_http(mut self, client: ind_egress::GuardedHttpClient) -> Self {
        self.context.webhook_http = client;
        self
    }

    pub fn build(self) -> WorkerContext {
        self.context
    }
}
