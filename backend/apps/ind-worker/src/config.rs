use config::{Config, File, FileFormat};
use ind_domain::ai::MILA_EMBEDDING_DIM;
use secrecy::SecretString;
use serde::Deserialize;

mod env;
mod settings;

use env::{EnvSource, ProcessEnv};
pub use settings::WorkerConfig;

#[derive(Clone, Debug, Deserialize)]
pub struct WorkerRuntimeSettings {
    pub max_concurrency: usize,
    pub claim_buffer_size: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CaptureWorkerSettings {
    pub max_concurrency: usize,
}

impl Default for CaptureWorkerSettings {
    fn default() -> Self {
        Self { max_concurrency: 1 }
    }
}

impl Default for WorkerRuntimeSettings {
    fn default() -> Self {
        Self {
            max_concurrency: 16,
            claim_buffer_size: 16,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct WorkerAuthSettings {
    pub credential_key: Option<SecretString>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct IntegrationsWorkerSettings {
    #[serde(default)]
    pub notion: NotionWorkerSettings,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NotionWorkerSettings {
    pub catch_up_enabled: bool,
    pub catch_up_interval_secs: u64,
    pub export_max_concurrency: usize,
    pub sync_max_concurrency: usize,
}

impl Default for NotionWorkerSettings {
    fn default() -> Self {
        Self {
            catch_up_enabled: true,
            catch_up_interval_secs: 86_400,
            export_max_concurrency: 2,
            sync_max_concurrency: 1,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct EmailIngestWorkerSettings {
    pub provider: Option<String>,
    pub webhook_secret: Option<SecretString>,
    pub resend_api_key: Option<SecretString>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WorkerServerSettings {
    pub environment: String,
    pub log_level: String,
    pub hostname: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RelaySettings {
    pub poll_interval_ms: u64,
    pub batch_size: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FeedWorkerSettings {
    pub enabled: bool,
    pub scheduler_interval_secs: u64,
    pub batch_size: i64,
    pub lease_secs: i64,
    pub default_poll_interval_minutes: i64,
    pub min_poll_interval_minutes: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AutoHealSettings {
    pub enabled: bool,
    pub interval_secs: u64,
    pub stale_after_secs: i64,
    pub lease_secs: i64,
    pub maintenance_lease_secs: i64,
    pub batch_size: i64,
    pub embedding_repair_interval_secs: u64,
    pub integrity_interval_secs: u64,
    pub tts_orphan_interval_secs: u64,
    pub tts_orphan_page_size: i32,
    /// Universal background job recovery: maximum sweeper-driven replays
    /// before a recovery row is force-terminalized + DLQ'd.
    pub job_recovery_max_attempts: i32,
    /// Optional override for the recovery sweeper batch size. When `None`,
    /// the sweeper reuses [`AutoHealSettings::batch_size`].
    pub job_recovery_batch_size: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TrashCleanupSettings {
    pub enabled: bool,
    pub interval_secs: u64,
    pub retention_days: i64,
}

impl Default for TrashCleanupSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 86400,
            retention_days: 30,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct FeedRetentionCleanupSettings {
    pub enabled: bool,
    pub interval_secs: u64,
    pub unseen_days: i64,
    pub seen_days: i64,
    pub dismissed_days: i64,
    pub document_grace_days: i64,
    pub compact_orphaned_source_entries: bool,
}

impl Default for FeedRetentionCleanupSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 86400,
            unseen_days: 60,
            seen_days: 21,
            dismissed_days: 14,
            document_grace_days: 7,
            compact_orphaned_source_entries: false,
        }
    }
}

impl WorkerConfig {
    /// Loads configuration from `configurations/base.toml` and
    /// `configurations/{environment}.toml` (both optional), merged with environment variables.
    ///
    /// Priority (highest to lowest):
    ///   env vars → configurations/{env}.toml → configurations/base.toml → built-in defaults.
    ///
    /// IND_ENV always wins for environment resolution; if absent, `[server].environment` inside
    /// `configurations/base.toml` is used so that file-only deployments select the right overlay.
    pub fn load() -> anyhow::Result<Self> {
        Self::load_from_env(&ProcessEnv)
    }

    fn load_from_env(env: &impl EnvSource) -> anyhow::Result<Self> {
        // IND_ENV wins; if absent, pre-read configurations/base.toml to select the right overlay.
        let environment = env.get("IND_ENV").unwrap_or_else(|| {
            Config::builder()
                .add_source(File::new("configurations/base.toml", FileFormat::Toml).required(false))
                .build()
                .ok()
                .and_then(|c| c.get_string("server.environment").ok())
                .unwrap_or_else(|| "development".to_string())
        });

        let mut cfg: Self = Config::builder()
            // Built-in defaults
            .set_default("server.environment", "development")?
            .set_default("server.log_level", "info")?
            .set_default("server.hostname", "unknown")?
            .set_default("worker.max_concurrency", 16_i64)?
            .set_default("worker.claim_buffer_size", 16_i64)?
            .set_default("capture.max_concurrency", 1_i64)?
            .set_default("renderer_url", "http://127.0.0.1:3100")?
            // Development defaults the egress guard open for private/loopback
            // targets so a local AI/feed endpoint works without extra config
            // (mirrors ind-renderer / ind-api). Production stays closed;
            // EGRESS_ALLOW_PRIVATE_TARGETS overrides either way.
            .set_default("egress.allow_private_targets", environment == "development")?
            .set_default("s3_enabled", false)?
            .set_default("s3_bucket", "indelible")?
            .set_default("s3_region", "us-east-1")?
            .set_default("s3_force_path_style", true)?
            .set_default("relay.poll_interval_ms", 1000_i64)?
            .set_default("relay.batch_size", 10_i64)?
            .set_default("feed.enabled", true)?
            .set_default("feed.scheduler_interval_secs", 60_i64)?
            .set_default("feed.batch_size", 50_i64)?
            .set_default("feed.lease_secs", 120_i64)?
            .set_default("feed.default_poll_interval_minutes", 15_i64)?
            .set_default("feed.min_poll_interval_minutes", 15_i64)?
            .set_default("auto_heal.enabled", true)?
            .set_default("auto_heal.interval_secs", 60_i64)?
            .set_default("auto_heal.stale_after_secs", 180_i64)?
            .set_default("auto_heal.lease_secs", 180_i64)?
            .set_default("auto_heal.maintenance_lease_secs", 900_i64)?
            .set_default("auto_heal.batch_size", 100_i64)?
            .set_default("auto_heal.embedding_repair_interval_secs", 900_i64)?
            .set_default("auto_heal.integrity_interval_secs", 3_600_i64)?
            .set_default("auto_heal.tts_orphan_interval_secs", 86_400_i64)?
            .set_default("auto_heal.tts_orphan_page_size", 1_000_i64)?
            .set_default("auto_heal.job_recovery_max_attempts", 3_i64)?
            .set_default("trash_cleanup.enabled", true)?
            .set_default("trash_cleanup.interval_secs", 86400_i64)?
            .set_default("trash_cleanup.retention_days", 30_i64)?
            .set_default("feed_retention_cleanup.enabled", true)?
            .set_default("feed_retention_cleanup.interval_secs", 86400_i64)?
            .set_default("feed_retention_cleanup.unseen_days", 60_i64)?
            .set_default("feed_retention_cleanup.seen_days", 21_i64)?
            .set_default("feed_retention_cleanup.dismissed_days", 14_i64)?
            .set_default("feed_retention_cleanup.document_grace_days", 7_i64)?
            .set_default(
                "feed_retention_cleanup.compact_orphaned_source_entries",
                false,
            )?
            .set_default("integrations.notion.catch_up_enabled", true)?
            .set_default("integrations.notion.catch_up_interval_secs", 86_400_i64)?
            .set_default("integrations.notion.export_max_concurrency", 2_i64)?
            .set_default("integrations.notion.sync_max_concurrency", 1_i64)?
            .set_default("mila.enabled", false)?
            .set_default("mila.chat_api_base", "https://api.openai.com/v1")?
            .set_default("mila.chat_model", "gpt-4.1-mini")?
            .set_default("mila.embedding_api_base", "https://api.openai.com/v1")?
            .set_default("mila.embedding_model", "text-embedding-3-small")?
            .set_default("mila.embedding_dim", 768_i64)?
            .set_default("mila.model_context_window", 12000_i64)?
            .set_default("mila.chunk_size", 512_i64)?
            .set_default("mila.chunk_overlap", 64_i64)?
            .set_default("mila.top_k", 6_i64)?
            .set_default("mila.cross_item_top_k", 20_i64)?
            .set_default("mila.cross_item_max_per_item", 3_i64)?
            .set_default("mila.supports_structured_output", true)?
            .set_default("mila.supports_reasoning_effort", false)?
            // TOML config files (optional; env-specific overlays base)
            .add_source(File::new("configurations/base.toml", FileFormat::Toml).required(false))
            .add_source(
                File::new(
                    &format!("configurations/{environment}.toml"),
                    FileFormat::Toml,
                )
                .required(false),
            )
            // Explicit env-var overrides
            .set_override_option("database_url", env.get("DATABASE_URL"))?
            .set_override_option("server.environment", env.get("IND_ENV"))?
            .set_override_option("server.log_level", env.get("RUST_LOG"))?
            .set_override_option("server.hostname", env.get("HOSTNAME"))?
            .set_override_option(
                "worker.max_concurrency",
                parse_i64(env, "WORKER_MAX_CONCURRENCY"),
            )?
            .set_override_option(
                "worker.claim_buffer_size",
                parse_i64(env, "WORKER_CLAIM_BUFFER_SIZE"),
            )?
            .set_override_option(
                "capture.max_concurrency",
                parse_i64(env, "CAPTURE_MAX_CONCURRENCY")
                    .or_else(|| parse_i64(env, "FEED_PREFETCH_MAX_CONCURRENCY")),
            )?
            .set_override_option("renderer_url", env.get("RENDERER_URL"))?
            .set_override_option("s3_enabled", parse_bool(env, "S3_ENABLED"))?
            .set_override_option("s3_bucket", env.get("S3_BUCKET"))?
            .set_override_option("s3_endpoint", env.get("S3_ENDPOINT"))?
            .set_override_option("s3_region", env.get("S3_REGION"))?
            .set_override_option("s3_access_key", env.get("S3_ACCESS_KEY"))?
            .set_override_option("s3_secret_key", env.get("S3_SECRET_KEY"))?
            .set_override_option(
                "s3_force_path_style",
                parse_bool(env, "S3_FORCE_PATH_STYLE"),
            )?
            .set_override_option(
                "relay.poll_interval_ms",
                parse_i64(env, "RELAY_POLL_INTERVAL_MS"),
            )?
            .set_override_option("relay.batch_size", parse_i64(env, "RELAY_BATCH_SIZE"))?
            .set_override_option("feed.enabled", parse_bool(env, "FEED_WORKER_ENABLED"))?
            .set_override_option(
                "feed.scheduler_interval_secs",
                parse_i64(env, "FEED_SCHEDULER_INTERVAL_SECS"),
            )?
            .set_override_option(
                "feed.batch_size",
                parse_i64(env, "FEED_SCHEDULER_BATCH_SIZE"),
            )?
            .set_override_option(
                "feed.lease_secs",
                parse_i64(env, "FEED_SCHEDULER_LEASE_SECS"),
            )?
            .set_override_option(
                "feed.default_poll_interval_minutes",
                parse_i64(env, "FEED_DEFAULT_POLL_INTERVAL_MINUTES"),
            )?
            .set_override_option(
                "feed.min_poll_interval_minutes",
                parse_i64(env, "FEED_MIN_POLL_INTERVAL_MINUTES"),
            )?
            .set_override_option("auto_heal.enabled", parse_bool(env, "AUTO_HEAL_ENABLED"))?
            .set_override_option(
                "auto_heal.interval_secs",
                parse_i64(env, "AUTO_HEAL_INTERVAL_SECS"),
            )?
            .set_override_option(
                "auto_heal.stale_after_secs",
                parse_i64(env, "AUTO_HEAL_STALE_AFTER_SECS"),
            )?
            .set_override_option(
                "auto_heal.lease_secs",
                parse_i64(env, "AUTO_HEAL_LEASE_SECS"),
            )?
            .set_override_option(
                "auto_heal.maintenance_lease_secs",
                parse_i64(env, "AUTO_HEAL_MAINTENANCE_LEASE_SECS"),
            )?
            .set_override_option(
                "auto_heal.batch_size",
                parse_i64(env, "AUTO_HEAL_BATCH_SIZE"),
            )?
            .set_override_option(
                "auto_heal.embedding_repair_interval_secs",
                parse_i64(env, "AUTO_HEAL_EMBEDDING_REPAIR_INTERVAL_SECS"),
            )?
            .set_override_option(
                "auto_heal.integrity_interval_secs",
                parse_i64(env, "AUTO_HEAL_INTEGRITY_INTERVAL_SECS"),
            )?
            .set_override_option(
                "auto_heal.tts_orphan_interval_secs",
                parse_i64(env, "AUTO_HEAL_TTS_ORPHAN_INTERVAL_SECS"),
            )?
            .set_override_option(
                "auto_heal.tts_orphan_page_size",
                parse_i64(env, "AUTO_HEAL_TTS_ORPHAN_PAGE_SIZE"),
            )?
            .set_override_option(
                "auto_heal.job_recovery_max_attempts",
                parse_i64(env, "AUTO_HEAL_JOB_RECOVERY_MAX_ATTEMPTS"),
            )?
            .set_override_option(
                "auto_heal.job_recovery_batch_size",
                parse_i64(env, "AUTO_HEAL_JOB_RECOVERY_BATCH_SIZE"),
            )?
            .set_override_option(
                "trash_cleanup.enabled",
                parse_bool(env, "TRASH_CLEANUP_ENABLED"),
            )?
            .set_override_option(
                "trash_cleanup.interval_secs",
                parse_i64(env, "TRASH_CLEANUP_INTERVAL_SECS"),
            )?
            .set_override_option(
                "trash_cleanup.retention_days",
                parse_i64(env, "TRASH_CLEANUP_RETENTION_DAYS"),
            )?
            .set_override_option(
                "feed_retention_cleanup.enabled",
                parse_bool(env, "FEED_RETENTION_CLEANUP_ENABLED"),
            )?
            .set_override_option(
                "feed_retention_cleanup.interval_secs",
                parse_i64(env, "FEED_RETENTION_CLEANUP_INTERVAL_SECS"),
            )?
            .set_override_option(
                "feed_retention_cleanup.unseen_days",
                parse_i64(env, "FEED_RETENTION_UNSEEN_DAYS"),
            )?
            .set_override_option(
                "feed_retention_cleanup.seen_days",
                parse_i64(env, "FEED_RETENTION_SEEN_DAYS"),
            )?
            .set_override_option(
                "feed_retention_cleanup.dismissed_days",
                parse_i64(env, "FEED_RETENTION_DISMISSED_DAYS"),
            )?
            .set_override_option(
                "feed_retention_cleanup.document_grace_days",
                parse_i64(env, "FEED_RETENTION_DOCUMENT_GRACE_DAYS"),
            )?
            .set_override_option(
                "feed_retention_cleanup.compact_orphaned_source_entries",
                parse_bool(env, "FEED_RETENTION_COMPACT_ORPHANED_SOURCE_ENTRIES"),
            )?
            .set_override_option("mila.enabled", parse_bool(env, "MILA_ENABLED"))?
            .set_override_option("mila.chat_api_base", env.get("MILA_CHAT_API_BASE"))?
            .set_override_option("mila.chat_model", env.get("MILA_CHAT_MODEL"))?
            .set_override_option(
                "mila.embedding_api_base",
                env.get("MILA_EMBEDDING_API_BASE"),
            )?
            .set_override_option("mila.embedding_model", env.get("MILA_EMBEDDING_MODEL"))?
            .set_override_option("mila.embedding_dim", parse_i64(env, "MILA_EMBEDDING_DIM"))?
            .set_override_option(
                "mila.model_context_window",
                parse_i64(env, "MILA_MODEL_CONTEXT_WINDOW"),
            )?
            .set_override_option(
                "mila.chat_context_pct",
                parse_i64(env, "MILA_CHAT_CONTEXT_PCT"),
            )?
            .set_override_option("mila.chunk_size", parse_i64(env, "MILA_CHUNK_SIZE"))?
            .set_override_option("mila.chunk_overlap", parse_i64(env, "MILA_CHUNK_OVERLAP"))?
            .set_override_option("mila.top_k", parse_i64(env, "MILA_TOP_K"))?
            .set_override_option(
                "mila.cross_item_top_k",
                parse_i64(env, "MILA_CROSS_ITEM_TOP_K"),
            )?
            .set_override_option(
                "mila.cross_item_max_per_item",
                parse_i64(env, "MILA_CROSS_ITEM_MAX_PER_ITEM"),
            )?
            .set_override_option(
                "mila.supports_structured_output",
                parse_bool(env, "MILA_SUPPORTS_STRUCTURED_OUTPUT"),
            )?
            .set_override_option(
                "mila.supports_reasoning_effort",
                parse_bool(env, "MILA_SUPPORTS_REASONING_EFFORT"),
            )?
            .set_override_option("email_ingest.provider", env.get("EMAIL_INGEST_PROVIDER"))?
            .set_override_option(
                "email_ingest.webhook_secret",
                env.get("EMAIL_INGEST_WEBHOOK_SECRET"),
            )?
            .set_override_option("email_ingest.resend_api_key", env.get("RESEND_API_KEY"))?
            .set_override_option(
                "integrations.notion.catch_up_enabled",
                parse_bool(env, "NOTION_CATCH_UP_ENABLED"),
            )?
            .set_override_option(
                "integrations.notion.catch_up_interval_secs",
                parse_i64(env, "NOTION_CATCH_UP_INTERVAL_SECS"),
            )?
            .set_override_option(
                "integrations.notion.export_max_concurrency",
                parse_i64(env, "NOTION_EXPORT_MAX_CONCURRENCY"),
            )?
            .set_override_option(
                "integrations.notion.sync_max_concurrency",
                parse_i64(env, "NOTION_SYNC_MAX_CONCURRENCY"),
            )?
            .set_override_option(
                "egress.allow_private_targets",
                parse_bool(env, "EGRESS_ALLOW_PRIVATE_TARGETS"),
            )?
            .set_override_option(
                "webhooks.allow_private_targets",
                parse_bool(env, "WEBHOOKS_ALLOW_PRIVATE_TARGETS"),
            )?
            .set_override_option("auth.credential_key", env.get("AUTH_CREDENTIAL_KEY"))?
            .build()?
            .try_deserialize()?;

        if parse_bool(env, "S3_ENABLED").is_none() && env.get("S3_ENDPOINT").is_some() {
            cfg.s3_enabled = true;
        }

        validate_positive_usize("worker.max_concurrency", cfg.worker.max_concurrency)?;
        validate_positive_usize("worker.claim_buffer_size", cfg.worker.claim_buffer_size)?;
        validate_positive_usize("capture.max_concurrency", cfg.capture.max_concurrency)?;
        validate_positive_usize(
            "integrations.notion.export_max_concurrency",
            cfg.integrations.notion.export_max_concurrency,
        )?;
        validate_positive_usize(
            "integrations.notion.sync_max_concurrency",
            cfg.integrations.notion.sync_max_concurrency,
        )?;
        if cfg.mila.embedding_dim != MILA_EMBEDDING_DIM {
            anyhow::bail!(
                "mila.embedding_dim must be {MILA_EMBEDDING_DIM}; pgvector storage is fixed at \
                 {MILA_EMBEDDING_DIM} dimensions for this release"
            );
        }
        cfg.mila.validate().map_err(|e| anyhow::anyhow!(e))?;

        if cfg.auto_heal.enabled {
            validate_positive_u64("auto_heal.interval_secs", cfg.auto_heal.interval_secs)?;
            validate_positive_i64("auto_heal.lease_secs", cfg.auto_heal.lease_secs)?;
            validate_positive_i64(
                "auto_heal.maintenance_lease_secs",
                cfg.auto_heal.maintenance_lease_secs,
            )?;
            validate_positive_i64("auto_heal.batch_size", cfg.auto_heal.batch_size)?;
            validate_positive_u64(
                "auto_heal.embedding_repair_interval_secs",
                cfg.auto_heal.embedding_repair_interval_secs,
            )?;
            validate_positive_u64(
                "auto_heal.integrity_interval_secs",
                cfg.auto_heal.integrity_interval_secs,
            )?;
            validate_positive_u64(
                "auto_heal.tts_orphan_interval_secs",
                cfg.auto_heal.tts_orphan_interval_secs,
            )?;
            if !(1..=1_000).contains(&cfg.auto_heal.tts_orphan_page_size) {
                anyhow::bail!(
                    "auto_heal.tts_orphan_page_size must be between 1 and 1000, got {}",
                    cfg.auto_heal.tts_orphan_page_size
                );
            }
        }

        if cfg.trash_cleanup.enabled && cfg.trash_cleanup.retention_days <= 0 {
            anyhow::bail!(
                "trash_cleanup.retention_days must be a positive integer, got {}",
                cfg.trash_cleanup.retention_days
            );
        }
        if cfg.feed_retention_cleanup.enabled {
            validate_positive_u64(
                "feed_retention_cleanup.interval_secs",
                cfg.feed_retention_cleanup.interval_secs,
            )?;
            validate_positive_i64(
                "feed_retention_cleanup.unseen_days",
                cfg.feed_retention_cleanup.unseen_days,
            )?;
            validate_positive_i64(
                "feed_retention_cleanup.seen_days",
                cfg.feed_retention_cleanup.seen_days,
            )?;
            validate_positive_i64(
                "feed_retention_cleanup.dismissed_days",
                cfg.feed_retention_cleanup.dismissed_days,
            )?;
            validate_positive_i64(
                "feed_retention_cleanup.document_grace_days",
                cfg.feed_retention_cleanup.document_grace_days,
            )?;
        }

        Ok(cfg)
    }
}

fn validate_positive_i64(field: &str, value: i64) -> anyhow::Result<()> {
    if value <= 0 {
        anyhow::bail!("{field} must be a positive integer, got {value}");
    }
    Ok(())
}

fn validate_positive_u64(field: &str, value: u64) -> anyhow::Result<()> {
    if value == 0 {
        anyhow::bail!("{field} must be a positive integer, got {value}");
    }
    Ok(())
}

fn validate_positive_usize(field: &str, value: usize) -> anyhow::Result<()> {
    if value == 0 {
        anyhow::bail!("{field} must be a positive integer, got {value}");
    }
    Ok(())
}

fn parse_i64(env: &impl EnvSource, key: &str) -> Option<i64> {
    env.get(key).and_then(|value| ind_config::parse_i64(&value))
}

fn parse_bool(env: &impl EnvSource, key: &str) -> Option<bool> {
    env.get(key)
        .and_then(|value| ind_config::parse_bool(&value))
}

fn default_s3_region() -> String {
    "us-east-1".to_string()
}

fn default_s3_force_path_style() -> bool {
    true
}

#[cfg(test)]
mod tests;
