use ind_application::asset_serving::AssetServingMode;
use ind_http_api::middleware::rate_limit::RateLimitConfig;
use ind_http_api::{AppConfig, Environment};
use secrecy::ExposeSecret;

use crate::config::ServerConfig;

pub(super) fn build_app_config(
    config: &ServerConfig,
    rate_limit_config: RateLimitConfig,
) -> anyhow::Result<AppConfig> {
    Ok(AppConfig {
        base_url: config.server.base_url.clone(),
        frontend_url: config.cors.frontend_url.clone(),
        extension_redirect_uris: config.extension.redirect_uris.clone(),
        cors_origins: config.cors.origins.clone(),
        environment: parse_environment(&config.server.environment),
        default_page_size: 50,
        max_page_size: 200,
        csrf_secret: config.auth.csrf_secret.expose_secret().as_bytes().to_vec(),
        cookie_domain: config.auth.cookie_domain.clone(),
        rate_limit: rate_limit_config,
        max_upload_bytes: config.storage.max_upload_bytes as usize,
        max_import_upload_bytes: config.storage.max_import_upload_bytes as usize,
        asset_serving_mode: config.storage.asset_serving_mode,
        asset_cookie_secret: decode_asset_cookie_secret(config)?,
        email_feed_domain: config.email_ingest.feed_domain.clone(),
        email_library_domain: config.email_ingest.library_domain.clone(),
        allow_private_webhook_targets: config.webhook_egress_policy().allow_private_targets,
        allow_signups: config.auth.allow_signups,
        // Read-ahead knobs (docs/document-feed-library-architecture.md defaults). Env-overridable;
        // there is no truncated_threshold_chars (preparation never trusts inline feed content).
        feed_prefetch: ind_application::FeedPreparationConfig {
            enabled: env_bool("FEED_PREFETCH_ENABLED", true),
            read_ahead_count: env_parse("FEED_PREFETCH_READ_AHEAD_COUNT", 10),
            active_within_days: env_parse("FEED_PREFETCH_ACTIVE_WITHIN_DAYS", 7),
        },
    })
}

fn env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_environment(s: &str) -> Environment {
    match s {
        "production" => Environment::Production,
        "staging" => Environment::Staging,
        _ => Environment::Development,
    }
}

fn decode_asset_cookie_secret(config: &ServerConfig) -> anyhow::Result<Option<Vec<u8>>> {
    if !matches!(
        config.storage.asset_serving_mode,
        AssetServingMode::Passthrough
    ) {
        return Ok(None);
    }

    config
        .storage
        .asset_cookie_secret
        .as_ref()
        .map(|secret| ind_auth::decode_asset_cookie_secret(secret.expose_secret()))
        .transpose()
        .map_err(anyhow::Error::from)
}
