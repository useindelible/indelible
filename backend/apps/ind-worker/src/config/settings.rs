use ind_domain::MilaPlatformDefaults;
use ind_persistence::storage::{S3Config, S3ConfigError};
use secrecy::SecretString;
use serde::Deserialize;

use super::{
    AutoHealSettings, CaptureWorkerSettings, EmailIngestWorkerSettings,
    FeedRetentionCleanupSettings, FeedWorkerSettings, IntegrationsWorkerSettings, RelaySettings,
    TrashCleanupSettings, WorkerAuthSettings, WorkerRuntimeSettings, WorkerServerSettings,
    default_s3_force_path_style, default_s3_region,
};

#[derive(Clone, Debug, Deserialize)]
pub struct WorkerConfig {
    pub server: WorkerServerSettings,
    pub worker: WorkerRuntimeSettings,
    #[serde(default)]
    pub capture: CaptureWorkerSettings,
    pub database_url: SecretString,
    pub renderer_url: String,
    /// Longer than the renderer's own capture deadline so the renderer normally reports its
    /// classified timeout first; this is the backstop for a renderer that never answers at all.
    pub renderer_request_timeout_secs: u64,
    pub s3_enabled: bool,
    pub s3_bucket: String,
    pub s3_endpoint: Option<String>,
    #[serde(default = "default_s3_region")]
    pub s3_region: String,
    pub s3_access_key: Option<SecretString>,
    pub s3_secret_key: Option<SecretString>,
    #[serde(default = "default_s3_force_path_style")]
    pub s3_force_path_style: bool,
    pub relay: RelaySettings,
    pub feed: FeedWorkerSettings,
    pub auto_heal: AutoHealSettings,
    #[serde(default)]
    pub trash_cleanup: TrashCleanupSettings,
    #[serde(default)]
    pub feed_retention_cleanup: FeedRetentionCleanupSettings,
    pub mila: MilaPlatformDefaults,
    #[serde(default)]
    pub email_ingest: EmailIngestWorkerSettings,
    #[serde(default)]
    pub integrations: IntegrationsWorkerSettings,
    #[serde(default)]
    pub auth: WorkerAuthSettings,
    #[serde(default)]
    pub egress: EgressWorkerSettings,
    #[serde(default)]
    pub webhooks: WebhookWorkerSettings,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct EgressWorkerSettings {
    #[serde(default)]
    pub allow_private_targets: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct WebhookWorkerSettings {
    #[serde(default)]
    pub allow_private_targets: bool,
}

impl WorkerConfig {
    pub fn egress_policy(&self) -> ind_egress::EgressPolicy {
        ind_egress::EgressPolicy {
            allow_private_targets: self.egress.allow_private_targets,
            extra_allowed_ips: Vec::new(),
        }
    }

    pub fn webhook_egress_policy(&self) -> ind_egress::EgressPolicy {
        ind_egress::EgressPolicy {
            allow_private_targets: self.egress.allow_private_targets
                || self.webhooks.allow_private_targets,
            extra_allowed_ips: Vec::new(),
        }
    }

    pub fn s3_config(&self) -> Result<S3Config, S3ConfigError> {
        S3Config::from_required_parts(
            self.s3_endpoint.clone(),
            self.s3_region.clone(),
            self.s3_access_key.clone(),
            self.s3_secret_key.clone(),
            Some(self.s3_bucket.clone()),
            self.s3_force_path_style,
        )
    }
}
