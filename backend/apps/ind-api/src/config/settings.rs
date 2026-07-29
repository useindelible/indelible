use ind_persistence::storage::{S3Config, S3ConfigError};
use secrecy::SecretString;
use serde::Deserialize;

use ind_application::asset_serving::AssetServingMode;
use ind_domain::MilaPlatformDefaults;

#[derive(Deserialize)]
pub struct ServerConfig {
    pub server: ServerSettings,
    pub database_url: SecretString,
    pub auth: AuthSettings,
    pub cors: CorsSettings,
    #[serde(default)]
    pub extension: ExtensionSettings,
    #[serde(default)]
    pub oauth: OAuthSettings,
    pub storage: StorageSettings,
    pub mila: MilaPlatformDefaults,
    #[serde(default)]
    pub tts: TtsSettings,
    pub rate_limit: RateLimitSettings,
    #[serde(default)]
    pub email_ingest: EmailIngestSettings,
    #[serde(default)]
    pub integrations: IntegrationsSettings,
    #[serde(default)]
    pub webhooks: WebhookSettings,
    #[serde(default)]
    pub egress: EgressSettings,
    #[serde(default)]
    pub network: NetworkSettings,
    pub log_level: String,
}

#[derive(Deserialize)]
pub struct ExtensionSettings {
    #[serde(default = "default_extension_redirect_uris")]
    pub redirect_uris: Vec<String>,
}

impl Default for ExtensionSettings {
    fn default() -> Self {
        Self {
            redirect_uris: default_extension_redirect_uris(),
        }
    }
}

pub(super) fn default_extension_redirect_uris() -> Vec<String> {
    vec![
        "https://lblngpkieoichinegfhgacmcjbahjbek.chromiumapp.org/indelible".to_string(),
        "https://38bd18db5de5caccb6ab6c1271fec03ec1662d5c.extensions.allizom.org/indelible"
            .to_string(),
    ]
}

impl ServerConfig {
    /// Egress policy for general outbound fetches (article ingest, feeds, AI
    /// providers, renderer pre-flight).
    pub fn egress_policy(&self) -> ind_egress::EgressPolicy {
        ind_egress::EgressPolicy {
            allow_private_targets: self.egress.allow_private_targets,
            extra_allowed_ips: Vec::new(),
        }
    }

    /// Egress policy for webhook delivery. Honors the webhook-specific
    /// `allow_private_targets` flag (used at creation time) so an operator who
    /// permits a private endpoint also gets deliveries to it.
    pub fn webhook_egress_policy(&self) -> ind_egress::EgressPolicy {
        ind_egress::EgressPolicy {
            allow_private_targets: self.egress.allow_private_targets
                || self.webhooks.allow_private_targets,
            extra_allowed_ips: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub environment: String,
    pub base_url: String,
    /// Directory holding the built web app. Served on the same origin as the
    /// API when present; ignored when it is not, so source checkouts and tests
    /// run API-only.
    #[serde(default = "default_web_root")]
    pub web_root: String,
}

#[derive(Deserialize)]
pub struct AuthSettings {
    pub csrf_secret: SecretString,
    pub jwt_secret: SecretString,
    pub cookie_domain: Option<String>,
    pub credential_key: Option<SecretString>,
    #[serde(default = "default_allow_signups")]
    pub allow_signups: bool,
}

// Closed by default: the first account on an empty instance is always
// permitted (see create_first_user), so a fresh deployment still gets its
// owner without ever exposing open registration to the internet.
fn default_allow_signups() -> bool {
    false
}

fn default_web_root() -> String {
    "./web".to_string()
}

#[derive(Deserialize)]
pub struct CorsSettings {
    pub origins: Vec<String>,
    pub frontend_url: String,
}

#[derive(Deserialize)]
pub struct OAuthSettings {
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<SecretString>,
    pub apple_client_id: Option<String>,
    pub apple_team_id: Option<String>,
    pub apple_key_id: Option<String>,
    pub apple_private_key_pem: Option<SecretString>,
    #[serde(default)]
    pub oidc_enabled: bool,
    pub oidc_issuer_url: Option<String>,
    pub oidc_client_id: Option<String>,
    pub oidc_client_secret: Option<SecretString>,
    #[serde(default = "default_oidc_provider_name")]
    pub oidc_provider_name: String,
    #[serde(default = "default_oidc_scopes")]
    pub oidc_scopes: Vec<String>,
    #[serde(default = "default_oidc_auto_create_users")]
    pub oidc_auto_create_users: bool,
}

impl Default for OAuthSettings {
    fn default() -> Self {
        Self {
            google_client_id: None,
            google_client_secret: None,
            apple_client_id: None,
            apple_team_id: None,
            apple_key_id: None,
            apple_private_key_pem: None,
            oidc_enabled: false,
            oidc_issuer_url: None,
            oidc_client_id: None,
            oidc_client_secret: None,
            oidc_provider_name: default_oidc_provider_name(),
            oidc_scopes: default_oidc_scopes(),
            oidc_auto_create_users: default_oidc_auto_create_users(),
        }
    }
}

#[derive(Deserialize)]
pub struct StorageSettings {
    #[serde(default)]
    pub s3_enabled: bool,
    pub max_upload_bytes: u64,
    pub max_import_upload_bytes: u64,
    pub asset_serving_mode: AssetServingMode,
    pub asset_cookie_secret: Option<SecretString>,
    pub s3_endpoint: Option<String>,
    #[serde(default = "default_s3_region")]
    pub s3_region: String,
    pub s3_access_key: Option<SecretString>,
    pub s3_secret_key: Option<SecretString>,
    pub s3_bucket: Option<String>,
    #[serde(default = "default_s3_force_path_style")]
    pub s3_force_path_style: bool,
}

impl StorageSettings {
    pub fn s3_config(&self) -> Result<S3Config, S3ConfigError> {
        S3Config::from_required_parts(
            self.s3_endpoint.clone(),
            self.s3_region.clone(),
            self.s3_access_key.clone(),
            self.s3_secret_key.clone(),
            self.s3_bucket.clone(),
            self.s3_force_path_style,
        )
    }
}

fn default_s3_region() -> String {
    "us-east-1".to_string()
}

fn default_s3_force_path_style() -> bool {
    true
}

#[derive(Deserialize)]
pub struct RateLimitSettings {
    pub login: RateLimitEntry,
    pub registration: RateLimitEntry,
    pub password_reset: RateLimitEntry,
    pub user_api: RateLimitEntry,
}

#[derive(Deserialize)]
pub struct RateLimitEntry {
    pub requests: u32,
    pub window_secs: u64,
}

#[derive(Default, Deserialize)]
pub struct EmailIngestSettings {
    pub provider: Option<String>,
    pub feed_domain: Option<String>,
    pub library_domain: Option<String>,
    pub webhook_secret: Option<SecretString>,
    pub resend_api_key: Option<SecretString>,
}

#[derive(Default, Deserialize)]
pub struct WebhookSettings {
    #[serde(default)]
    pub allow_private_targets: bool,
}

#[derive(Clone, Default, Deserialize)]
pub struct NetworkSettings {
    /// Reverse-proxy IPs/CIDRs whose `X-Forwarded-For`/`X-Real-IP` headers are
    /// trusted. Empty (default) = trust no forwarded headers (use direct peer).
    #[serde(default)]
    pub trusted_proxies: Vec<String>,
}

#[derive(Clone, Default, Deserialize)]
pub struct EgressSettings {
    /// When true, outbound fetches may target private/loopback/internal hosts.
    /// Off by default; hosted SaaS never enables it. Self-host operators opt in
    /// to reach a local renderer, feed source, or AI provider.
    #[serde(default)]
    pub allow_private_targets: bool,
}

#[derive(Clone, Default, Deserialize)]
pub struct IntegrationsSettings {
    #[serde(default)]
    pub notion: IntegrationNotionOAuthSettings,
}

#[derive(Clone, Default, Deserialize)]
pub struct IntegrationNotionOAuthSettings {
    pub client_id: Option<String>,
    pub client_secret: Option<SecretString>,
    pub redirect_url: Option<String>,
}

#[derive(Clone, Default, Deserialize)]
pub struct TtsSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub hosted_managed_custom_persona: bool,
    #[serde(default)]
    pub use_mock_adapter: bool,
    #[serde(default)]
    pub deployment: DeploymentSetting,
    #[serde(default)]
    pub dashscope: TtsProviderSettings,
    #[serde(default)]
    pub unreal_speech: TtsProviderSettings,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentSetting {
    #[default]
    Hosted,
    SelfHosted,
}

impl From<DeploymentSetting> for ind_application::services::tts::Deployment {
    fn from(value: DeploymentSetting) -> Self {
        match value {
            DeploymentSetting::Hosted => Self::Hosted,
            DeploymentSetting::SelfHosted => Self::SelfHosted,
        }
    }
}

#[derive(Clone, Default, Deserialize)]
pub struct TtsProviderSettings {
    pub api_key: Option<SecretString>,
    pub api_base: Option<String>,
    #[serde(default)]
    pub transcript_supported: bool,
}

fn default_oidc_provider_name() -> String {
    "OpenID Connect".to_string()
}

fn default_oidc_scopes() -> Vec<String> {
    vec![
        "openid".to_string(),
        "email".to_string(),
        "profile".to_string(),
    ]
}

fn default_oidc_auto_create_users() -> bool {
    true
}
