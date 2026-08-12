use std::sync::Arc;

use ind_application::FeedPreparationConfig;
use ind_application::asset_serving::AssetServingMode;
use ind_application::ports::{
    AccountOperations, ApiTokenOperations, ArticleTocOperations, AuthOperations,
    CollectionOperations, DocumentReaderOperations, EmailAliasOperations, EmailIngestOperations,
    EmailSenderOperations, EntityOperations, ExportOperations, ExtensionAuthOperations,
    ExtensionSaveOperations, FeedDeliveryOperations, FeedOperations, FeedPreparationOperations,
    HighlightOperations, HomeOperations, ImportOperations, IntegrationOperations,
    LibraryOperations, LibraryUploadOperations, MilaActionRetryPort, MilaChatPort, MilaConfigPort,
    MilaPromptPresetPort, MilaSessionPort, OAuthOperations, OnboardingOperations, SearchOperations,
    SettingsOperations, SmartListOperations, TagOperations, TokenValidator, TtsOperations,
    UserLookup, WebhookOperations,
};
use ind_application::repos::event::EventRepository;
use ind_application::repos::oauth_flow::OAuthFlowRepository;
use ind_application::storage::ObjectStorage;
use ind_auth::oauth::OAuthConfig;
use sqlx::PgPool;

use crate::middleware::ip_extract::TrustedProxies;
use crate::middleware::rate_limit::{RateLimitConfig, UserRateLimiter};
use crate::realtime::RealtimeHub;

pub use ind_application::HighlightWithNote;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: AppConfig,
    pub jwt_secret: Vec<u8>,
    pub user_rate_limiter: UserRateLimiter,
    /// Reverse-proxy allowlist used by the `ClientIp` extractor to resolve a
    /// spoof-resistant client IP for audit/session attribution. Defaults empty
    /// (trust no forwarded headers); populated in `ind-api`'s router from
    /// `TRUSTED_PROXIES`.
    pub trusted_proxies: TrustedProxies,
    pub token_validator: Arc<dyn TokenValidator>,
    pub user_lookup: Arc<dyn UserLookup>,
    pub auth_service: Arc<dyn AuthOperations>,
    pub oauth_service: Option<Arc<dyn OAuthOperations>>,
    pub oauth_config: Option<OAuthConfig>,
    pub oauth_flow_repo: Option<Arc<dyn OAuthFlowRepository>>,
    pub account_ops: Arc<dyn AccountOperations>,
    pub onboarding_ops: Arc<dyn OnboardingOperations>,
    pub api_token_ops: Arc<dyn ApiTokenOperations>,
    pub webhook_ops: Option<Arc<dyn WebhookOperations>>,
    pub settings_ops: Arc<dyn SettingsOperations>,
    pub library_ops: Option<Arc<dyn LibraryOperations>>,
    pub library_upload_ops: Option<Arc<dyn LibraryUploadOperations>>,
    pub extension_auth_ops: Option<Arc<dyn ExtensionAuthOperations>>,
    pub storage: Option<Arc<dyn ObjectStorage>>,
    pub extension_save_ops: Option<Arc<dyn ExtensionSaveOperations>>,
    pub feed_ops: Option<Arc<dyn FeedOperations>>,
    pub feed_delivery_ops: Option<Arc<dyn FeedDeliveryOperations>>,
    pub feed_preparation_ops: Option<Arc<dyn FeedPreparationOperations>>,
    pub highlight_ops: Option<Arc<dyn HighlightOperations>>,
    pub document_reader_ops: Option<Arc<dyn DocumentReaderOperations>>,
    pub article_toc_ops: Option<Arc<dyn ArticleTocOperations>>,
    pub home_ops: Option<Arc<dyn HomeOperations>>,
    pub search_ops: Option<Arc<dyn SearchOperations>>,
    pub mila_config_ops: Option<Arc<dyn MilaConfigPort>>,
    pub mila_prompt_preset_ops: Option<Arc<dyn MilaPromptPresetPort>>,
    pub mila_session_ops: Option<Arc<dyn MilaSessionPort>>,
    pub mila_chat_ops: Option<Arc<dyn MilaChatPort>>,
    pub mila_action_retry_ops: Option<Arc<dyn MilaActionRetryPort>>,
    pub entity_ops: Option<Arc<dyn EntityOperations>>,
    pub email_ingest_ops: Option<Arc<dyn EmailIngestOperations>>,
    pub email_ingest_provider: Option<Arc<dyn ind_integrations::email::InboundEmailProvider>>,
    pub email_sender_ops: Option<Arc<dyn EmailSenderOperations>>,
    pub email_alias_ops: Option<Arc<dyn EmailAliasOperations>>,
    pub collection_ops: Option<Arc<dyn CollectionOperations>>,
    pub tag_ops: Option<Arc<dyn TagOperations>>,
    pub smart_list_ops: Option<Arc<dyn SmartListOperations>>,
    pub tts_ops: Option<Arc<dyn TtsOperations>>,
    pub integration_ops: Option<Arc<dyn IntegrationOperations>>,
    pub import_ops: Option<Arc<dyn ImportOperations>>,
    pub export_ops: Option<Arc<dyn ExportOperations>>,
    pub export_summary_provider:
        Option<Arc<dyn ind_application::export_summary::ExportSummaryProvider>>,
    pub event_repo: Option<Arc<dyn EventRepository>>,
    pub realtime_hub: RealtimeHub,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub base_url: String,
    pub frontend_url: String,
    pub extension_redirect_uris: Vec<String>,
    /// Every browser-facing origin the operator configured. CSRF accepts any of
    /// them, so reaching one server through a second hostname (LAN name plus
    /// tailnet name) does not 403 on session refresh.
    pub cors_origins: Vec<String>,
    pub environment: Environment,
    pub default_page_size: u32,
    pub max_page_size: u32,
    pub csrf_secret: Vec<u8>,
    pub cookie_domain: Option<String>,
    pub rate_limit: RateLimitConfig,
    pub max_upload_bytes: usize,
    pub max_import_upload_bytes: usize,
    pub asset_serving_mode: AssetServingMode,
    pub asset_cookie_secret: Option<Vec<u8>>,
    pub email_feed_domain: Option<String>,
    pub email_library_domain: Option<String>,
    pub allow_private_webhook_targets: bool,
    pub allow_signups: bool,
    pub feed_prefetch: FeedPreparationConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:38473".to_string(),
            frontend_url: "http://localhost:5173".to_string(),
            extension_redirect_uris: Vec::new(),
            cors_origins: vec!["http://localhost:5173".to_string()],
            environment: Environment::Development,
            default_page_size: 50,
            max_page_size: 200,
            csrf_secret: b"dev-csrf-secret-change-in-production".to_vec(),
            cookie_domain: None,
            rate_limit: RateLimitConfig::default(),
            max_upload_bytes: ind_ingest::MAX_UPLOAD_BYTES,
            max_import_upload_bytes: 200 * 1024 * 1024,
            asset_serving_mode: AssetServingMode::Passthrough,
            asset_cookie_secret: None,
            email_feed_domain: None,
            email_library_domain: None,
            allow_private_webhook_targets: false,
            allow_signups: true,
            feed_prefetch: FeedPreparationConfig {
                enabled: true,
                read_ahead_count: 10,
                active_within_days: 7,
            },
        }
    }
}
