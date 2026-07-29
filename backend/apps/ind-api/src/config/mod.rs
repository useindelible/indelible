use std::time::Duration;

use config::{Config, File, FileFormat};
use secrecy::{ExposeSecret, SecretString};

use ind_application::asset_serving::AssetServingMode;
use ind_domain::ai::MILA_EMBEDDING_DIM;
use ind_http_api::middleware::rate_limit::{RateLimitConfig, RateLimitRule};

mod settings;

pub use settings::*;

pub(super) const DEFAULT_DEV_PORT: u16 = 38473;

trait EnvSource {
    fn get(&self, key: &str) -> Option<String>;
}

struct ProcessEnv;

impl EnvSource for ProcessEnv {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

impl ServerConfig {
    /// Loads configuration from `configurations/base.toml` and
    /// `configurations/{environment}.toml` (both optional), merged with environment variables.
    ///
    /// Priority (highest to lowest):
    ///   env vars → configurations/{env}.toml → configurations/base.toml → built-in defaults.
    ///
    /// The environment is resolved from `IND_ENV` first; if that is absent,
    /// `[server].environment` inside `configurations/base.toml` is used to determine which
    /// env-specific overlay to load. This means file-only deployments (no `IND_ENV` set)
    /// still select the right overlay.
    ///
    /// Dev-only convenience defaults (csrf/jwt secrets, CORS origins) are always registered
    /// so the config deserialises cleanly even before the environment is resolved. Production
    /// validation in `validate()` rejects any startup that still uses those placeholder values.
    ///
    /// The following environment variables are recognised:
    ///   IND_ENV, IND_HOST, IND_PORT, IND_BASE_URL, DATABASE_URL, CSRF_SECRET,
    ///   COOKIE_DOMAIN, CORS_ORIGINS (comma-separated), JWT_SECRET, FRONTEND_URL,
    ///   RUST_LOG, GOOGLE_CLIENT_ID/SECRET, APPLE_CLIENT_ID/TEAM_ID/KEY_ID,
    ///   APPLE_PRIVATE_KEY_PEM (or APPLE_CLIENT_SECRET as fallback),
    ///   S3_ENABLED, S3_ENDPOINT (legacy detection), UPLOAD_MAX_BYTES,
    ///   ASSET_SERVING_MODE, ASSET_COOKIE_SECRET,
    ///   LOGIN/REGISTRATION/PASSWORD_RESET rate-limit knobs, TTS_ENABLED,
    ///   TTS_HOSTED_MANAGED_CUSTOM_PERSONA, TTS_DASHSCOPE_API_KEY,
    ///   TTS_DASHSCOPE_API_BASE, TTS_UNREAL_SPEECH_API_KEY,
    ///   TTS_UNREAL_SPEECH_API_BASE, TTS_USE_MOCK_ADAPTER, INDELIBLE_DEPLOYMENT.
    pub fn load() -> anyhow::Result<Self> {
        Self::load_from_env(&ProcessEnv)
    }

    fn load_from_env(env: &impl EnvSource) -> anyhow::Result<Self> {
        // Resolve the environment before building the full config so we can:
        //   1. select the correct env-specific overlay file, and
        //   2. re-derive is_dev from the *fully resolved* value after deserialization.
        // IND_ENV always wins; if absent we do a quick pre-read of configurations/base.toml.
        let environment = env.get("IND_ENV").unwrap_or_else(|| {
            Config::builder()
                .add_source(File::new("configurations/base.toml", FileFormat::Toml).required(false))
                .build()
                .ok()
                .and_then(|c| c.get_string("server.environment").ok())
                .unwrap_or_else(|| "development".to_string())
        });

        // APPLE_PRIVATE_KEY_PEM with APPLE_CLIENT_SECRET as a legacy alias.
        let apple_pem = env
            .get("APPLE_PRIVATE_KEY_PEM")
            .or_else(|| env.get("APPLE_CLIENT_SECRET"));

        // CORS_ORIGINS: comma-separated, elements trimmed.
        let cors_origins = env.get("CORS_ORIGINS").map(|s| {
            s.split(',')
                .map(|o| o.trim().to_string())
                .filter(|o| !o.is_empty())
                .collect::<Vec<_>>()
        });

        // Dev-convenience defaults are always registered. Production validation rejects them
        // if the resolved environment is not "development", so registering them unconditionally
        // is safe and avoids the stale-is_dev bug where file-configured environments were
        // ignored when deciding whether to inject these defaults.
        let builder = Config::builder()
            // Built-in defaults
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", DEFAULT_DEV_PORT as i64)?
            .set_default("server.environment", "development")?
            .set_default(
                "server.base_url",
                format!("http://localhost:{DEFAULT_DEV_PORT}"),
            )?
            .set_default("server.web_root", "./web")?
            .set_default("log_level", "info")?
            .set_default("rate_limit.login.requests", 5_i64)?
            .set_default("rate_limit.login.window_secs", 30_i64)?
            .set_default("rate_limit.registration.requests", 3_i64)?
            .set_default("rate_limit.registration.window_secs", 60_i64)?
            .set_default("rate_limit.password_reset.requests", 3_i64)?
            .set_default("rate_limit.password_reset.window_secs", 900_i64)?
            .set_default("rate_limit.user_api.requests", 1000_i64)?
            .set_default("rate_limit.user_api.window_secs", 60_i64)?
            .set_default("auth.allow_signups", true)?
            .set_default("webhooks.allow_private_targets", false)?
            // Development defaults the egress guard open for private/loopback
            // targets so a local AI/feed/renderer endpoint works without extra
            // config (mirrors ind-renderer). Production/staging stay closed;
            // EGRESS_ALLOW_PRIVATE_TARGETS overrides either way.
            .set_default("egress.allow_private_targets", environment == "development")?
            .set_default("storage.s3_enabled", false)?
            .set_default("storage.s3_region", "us-east-1")?
            .set_default("storage.s3_force_path_style", true)?
            .set_default(
                "storage.max_upload_bytes",
                ind_ingest::MAX_UPLOAD_BYTES as i64,
            )?
            .set_default("storage.max_import_upload_bytes", 200_i64 * 1024 * 1024)?
            // Passthrough serves assets from the API origin, so a self-hoster needs no
            // publicly reachable object store. Presigned signs against S3_ENDPOINT
            // verbatim, which is an internal hostname in every shipped compose.
            .set_default("storage.asset_serving_mode", "passthrough")?
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
            .set_default("tts.enabled", true)?
            .set_default("tts.hosted_managed_custom_persona", false)?
            .set_default("tts.use_mock_adapter", false)?
            .set_default("tts.deployment", "hosted")?
            .set_default(
                "tts.dashscope.api_base",
                "https://dashscope-intl.aliyuncs.com",
            )?
            .set_default("tts.dashscope.transcript_supported", false)?
            .set_default(
                "tts.unreal_speech.api_base",
                "https://api.v8.unrealspeech.com",
            )?
            .set_default("tts.unreal_speech.transcript_supported", true)?
            .set_default("auth.csrf_secret", "dev-csrf-secret-change-in-production")?
            .set_default(
                "auth.jwt_secret",
                "dev-jwt-secret-change-in-production-min32",
            )?
            .set_default("cors.frontend_url", "http://localhost:5173")?
            .set_default("cors.origins", vec!["http://localhost:5173"])?;

        let mut cfg: Self = builder
            // TOML config files (optional; env-specific overlays base)
            .add_source(File::new("configurations/base.toml", FileFormat::Toml).required(false))
            .add_source(
                File::new(
                    &format!("configurations/{environment}.toml"),
                    FileFormat::Toml,
                )
                .required(false),
            )
            // Explicit env-var overrides (highest priority)
            .set_override_option("server.host", env.get("IND_HOST"))?
            .set_override_option("server.port", parse_i64(env, "IND_PORT"))?
            .set_override_option("server.environment", env.get("IND_ENV"))?
            .set_override_option("server.base_url", env.get("IND_BASE_URL"))?
            .set_override_option("server.web_root", env.get("WEB_ROOT"))?
            .set_override_option("database_url", env.get("DATABASE_URL"))?
            .set_override_option("auth.csrf_secret", env.get("CSRF_SECRET"))?
            .set_override_option("auth.jwt_secret", env.get("JWT_SECRET"))?
            .set_override_option("auth.cookie_domain", env.get("COOKIE_DOMAIN"))?
            .set_override_option("auth.credential_key", env.get("AUTH_CREDENTIAL_KEY"))?
            .set_override_option("auth.allow_signups", parse_bool(env, "AUTH_ALLOW_SIGNUPS"))?
            .set_override_option("cors.origins", cors_origins)?
            .set_override_option("cors.frontend_url", env.get("FRONTEND_URL"))?
            .set_override_option("oauth.google_client_id", env.get("GOOGLE_CLIENT_ID"))?
            .set_override_option(
                "oauth.google_client_secret",
                env.get("GOOGLE_CLIENT_SECRET"),
            )?
            .set_override_option("oauth.apple_client_id", env.get("APPLE_CLIENT_ID"))?
            .set_override_option("oauth.apple_team_id", env.get("APPLE_TEAM_ID"))?
            .set_override_option("oauth.apple_key_id", env.get("APPLE_KEY_ID"))?
            .set_override_option("oauth.apple_private_key_pem", apple_pem)?
            .set_override_option("oauth.oidc_enabled", parse_bool(env, "OIDC_ENABLED"))?
            .set_override_option("oauth.oidc_issuer_url", env.get("OIDC_ISSUER_URL"))?
            .set_override_option("oauth.oidc_client_id", env.get("OIDC_CLIENT_ID"))?
            .set_override_option("oauth.oidc_client_secret", env.get("OIDC_CLIENT_SECRET"))?
            .set_override_option("oauth.oidc_provider_name", env.get("OIDC_PROVIDER_NAME"))?
            .set_override_option("oauth.oidc_scopes", parse_csv(env, "OIDC_SCOPES"))?
            .set_override_option(
                "oauth.oidc_auto_create_users",
                parse_bool(env, "OIDC_AUTO_CREATE_USERS"),
            )?
            .set_override_option("log_level", env.get("RUST_LOG"))?
            .set_override_option("storage.s3_enabled", parse_bool(env, "S3_ENABLED"))?
            .set_override_option("storage.s3_endpoint", env.get("S3_ENDPOINT"))?
            .set_override_option("storage.s3_region", env.get("S3_REGION"))?
            .set_override_option("storage.s3_access_key", env.get("S3_ACCESS_KEY"))?
            .set_override_option("storage.s3_secret_key", env.get("S3_SECRET_KEY"))?
            .set_override_option("storage.s3_bucket", env.get("S3_BUCKET"))?
            .set_override_option(
                "storage.s3_force_path_style",
                parse_bool(env, "S3_FORCE_PATH_STYLE"),
            )?
            .set_override_option(
                "storage.max_upload_bytes",
                parse_i64(env, "UPLOAD_MAX_BYTES"),
            )?
            .set_override_option(
                "storage.max_import_upload_bytes",
                parse_i64(env, "IMPORT_UPLOAD_MAX_BYTES"),
            )?
            .set_override_option("storage.asset_serving_mode", env.get("ASSET_SERVING_MODE"))?
            .set_override_option(
                "storage.asset_cookie_secret",
                env.get("ASSET_COOKIE_SECRET"),
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
            .set_override_option("tts.enabled", parse_bool(env, "TTS_ENABLED"))?
            .set_override_option(
                "tts.hosted_managed_custom_persona",
                parse_bool(env, "TTS_HOSTED_MANAGED_CUSTOM_PERSONA"),
            )?
            .set_override_option(
                "tts.use_mock_adapter",
                parse_bool(env, "TTS_USE_MOCK_ADAPTER"),
            )?
            .set_override_option("tts.deployment", env.get("INDELIBLE_DEPLOYMENT"))?
            .set_override_option("tts.dashscope.api_key", env.get("TTS_DASHSCOPE_API_KEY"))?
            .set_override_option("tts.dashscope.api_base", env.get("TTS_DASHSCOPE_API_BASE"))?
            .set_override_option(
                "tts.dashscope.transcript_supported",
                parse_bool(env, "TTS_DASHSCOPE_TRANSCRIPT_SUPPORTED"),
            )?
            .set_override_option(
                "tts.unreal_speech.api_key",
                env.get("TTS_UNREAL_SPEECH_API_KEY"),
            )?
            .set_override_option(
                "tts.unreal_speech.api_base",
                env.get("TTS_UNREAL_SPEECH_API_BASE"),
            )?
            .set_override_option(
                "tts.unreal_speech.transcript_supported",
                parse_bool(env, "TTS_UNREAL_SPEECH_TRANSCRIPT_SUPPORTED"),
            )?
            .set_override_option(
                "rate_limit.login.requests",
                parse_i64(env, "LOGIN_RATE_LIMIT_REQUESTS"),
            )?
            .set_override_option(
                "rate_limit.login.window_secs",
                parse_i64(env, "LOGIN_RATE_LIMIT_WINDOW_SECS"),
            )?
            .set_override_option(
                "rate_limit.registration.requests",
                parse_i64(env, "REGISTRATION_RATE_LIMIT_REQUESTS"),
            )?
            .set_override_option(
                "rate_limit.registration.window_secs",
                parse_i64(env, "REGISTRATION_RATE_LIMIT_WINDOW_SECS"),
            )?
            .set_override_option(
                "rate_limit.password_reset.requests",
                parse_i64(env, "PASSWORD_RESET_RATE_LIMIT_REQUESTS"),
            )?
            .set_override_option(
                "rate_limit.password_reset.window_secs",
                parse_i64(env, "PASSWORD_RESET_RATE_LIMIT_WINDOW_SECS"),
            )?
            .set_override_option(
                "rate_limit.user_api.requests",
                parse_i64(env, "USER_API_RATE_LIMIT_REQUESTS"),
            )?
            .set_override_option(
                "rate_limit.user_api.window_secs",
                parse_i64(env, "USER_API_RATE_LIMIT_WINDOW_SECS"),
            )?
            .set_override_option("email_ingest.provider", env.get("EMAIL_INGEST_PROVIDER"))?
            .set_override_option("email_ingest.feed_domain", env.get("EMAIL_FEED_DOMAIN"))?
            .set_override_option(
                "email_ingest.library_domain",
                env.get("EMAIL_LIBRARY_DOMAIN"),
            )?
            .set_override_option(
                "email_ingest.webhook_secret",
                env.get("EMAIL_INGEST_WEBHOOK_SECRET"),
            )?
            .set_override_option("email_ingest.resend_api_key", env.get("RESEND_API_KEY"))?
            .set_override_option(
                "webhooks.allow_private_targets",
                parse_bool(env, "WEBHOOKS_ALLOW_PRIVATE_TARGETS"),
            )?
            .set_override_option(
                "egress.allow_private_targets",
                parse_bool(env, "EGRESS_ALLOW_PRIVATE_TARGETS"),
            )?
            .set_override_option("integrations.notion.client_id", env.get("NOTION_CLIENT_ID"))?
            .set_override_option(
                "integrations.notion.client_secret",
                env.get("NOTION_CLIENT_SECRET"),
            )?
            .set_override_option(
                "integrations.notion.redirect_url",
                env.get("NOTION_REDIRECT_URL"),
            )?
            .build()?
            .try_deserialize()?;

        // Re-derive is_dev from the fully-resolved environment so that setting
        // [server].environment = "production" in config.toml is respected even when
        // IND_ENV is not set in the process environment.
        let is_dev = cfg.server.environment == "development";

        if parse_bool(env, "S3_ENABLED").is_none() && env.get("S3_ENDPOINT").is_some() {
            cfg.storage.s3_enabled = true;
        }

        // TRUSTED_PROXIES is a comma-separated list of proxy IPs/CIDRs.
        if let Some(raw) = env.get("TRUSTED_PROXIES") {
            cfg.network.trusted_proxies = raw
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // In dev passthrough mode, auto-supply a default cookie secret so the server starts
        // without any additional configuration.
        if matches!(
            cfg.storage.asset_serving_mode,
            AssetServingMode::Passthrough
        ) && is_dev
            && cfg.storage.asset_cookie_secret.is_none()
        {
            cfg.storage.asset_cookie_secret = Some(SecretString::from(
                "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string(),
            ));
        }

        cfg.validate(is_dev)?;
        Ok(cfg)
    }

    fn validate(&self, is_dev: bool) -> anyhow::Result<()> {
        self.mila.validate().map_err(|e| anyhow::anyhow!(e))?;

        if !is_dev {
            let dev_csrf = "dev-csrf-secret-change-in-production";
            if self.auth.csrf_secret.expose_secret() == dev_csrf {
                anyhow::bail!("CSRF_SECRET must be set in production");
            }

            let dev_jwt = "dev-jwt-secret-change-in-production-min32";
            if self.auth.jwt_secret.expose_secret() == dev_jwt {
                anyhow::bail!("JWT_SECRET must be set in production");
            }

            if self.cors.frontend_url == "http://localhost:5173" {
                anyhow::bail!("FRONTEND_URL must be set in production");
            }

            if self.cors.origins == vec!["http://localhost:5173"] {
                anyhow::bail!("CORS_ORIGINS must be set in production");
            }

            // The refresh cookie is Secure outside development, so a plaintext origin
            // makes the browser discard it: login succeeds, then the session dies on
            // first refresh with nothing in the logs. Fail at boot instead.
            for (name, url) in [
                ("IND_BASE_URL", &self.server.base_url),
                ("FRONTEND_URL", &self.cors.frontend_url),
            ] {
                if url.starts_with("http://") {
                    anyhow::bail!(
                        "{name} must use https in production; the refresh cookie is \
                         Secure-only and browsers silently drop it over plain http"
                    );
                }
            }
        }

        if matches!(
            self.storage.asset_serving_mode,
            AssetServingMode::Passthrough
        ) {
            match &self.storage.asset_cookie_secret {
                None => {
                    if !is_dev {
                        anyhow::bail!(
                            "ASSET_COOKIE_SECRET must be set when ASSET_SERVING_MODE=passthrough \
                             in production"
                        );
                    }
                }
                Some(secret) => {
                    if secret.expose_secret().len() < 64 {
                        anyhow::bail!(
                            "ASSET_COOKIE_SECRET must be at least 32 bytes (64 hex chars)"
                        );
                    }
                }
            }
        }

        // Feature-gated credential completeness (applies in all environments — a
        // half-configured provider is a bug, not a deployment choice). Note: S3
        // creds are intentionally NOT required here — the AWS SDK supports
        // instance-profile/IAM-role credentials with no explicit keys.
        if self.email_ingest.provider.as_deref() == Some("resend")
            && (self.email_ingest.resend_api_key.is_none()
                || self.email_ingest.webhook_secret.is_none())
        {
            anyhow::bail!(
                "EMAIL_INGEST_PROVIDER=resend requires RESEND_API_KEY and \
                 EMAIL_INGEST_WEBHOOK_SECRET"
            );
        }

        if self.oauth.google_client_id.is_some() != self.oauth.google_client_secret.is_some() {
            anyhow::bail!("Google OAuth requires both GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET");
        }

        if self.oauth.oidc_enabled
            && (self.oauth.oidc_issuer_url.is_none()
                || self.oauth.oidc_client_id.is_none()
                || self.oauth.oidc_client_secret.is_none())
        {
            anyhow::bail!(
                "OIDC is enabled but OIDC_ISSUER_URL, OIDC_CLIENT_ID, and OIDC_CLIENT_SECRET \
                 are not all set"
            );
        }

        if self.oauth.apple_client_id.is_some()
            && (self.oauth.apple_team_id.is_none()
                || self.oauth.apple_key_id.is_none()
                || self.oauth.apple_private_key_pem.is_none())
        {
            anyhow::bail!(
                "Apple Sign-In requires APPLE_TEAM_ID, APPLE_KEY_ID, and APPLE_PRIVATE_KEY \
                 when APPLE_CLIENT_ID is set"
            );
        }

        if self.mila.embedding_dim != MILA_EMBEDDING_DIM {
            anyhow::bail!(
                "mila.embedding_dim must be {MILA_EMBEDDING_DIM}; pgvector storage is fixed at \
                 {MILA_EMBEDDING_DIM} dimensions for this release"
            );
        }

        // M.9: integration OAuth tokens are encrypted with auth.credential_key.
        // In production, refuse to boot with integrations configured but no key
        // rather than silently degrading to a broken integration flow.
        if !is_dev
            && self.integrations.notion.client_id.is_some()
            && self.auth.credential_key.is_none()
        {
            anyhow::bail!(
                "AUTH_CREDENTIAL_KEY is required in production when integrations are configured \
                 (NOTION_CLIENT_ID is set): integration tokens cannot be encrypted without it"
            );
        }

        Ok(())
    }

    pub fn is_production(&self) -> bool {
        self.server.environment == "production"
    }

    pub fn rate_limit_config(&self) -> RateLimitConfig {
        RateLimitConfig {
            login: RateLimitRule::new(
                self.rate_limit.login.requests,
                Duration::from_secs(self.rate_limit.login.window_secs),
            ),
            registration: RateLimitRule::new(
                self.rate_limit.registration.requests,
                Duration::from_secs(self.rate_limit.registration.window_secs),
            ),
            password_reset: RateLimitRule::new(
                self.rate_limit.password_reset.requests,
                Duration::from_secs(self.rate_limit.password_reset.window_secs),
            ),
            user_api: RateLimitRule::new(
                self.rate_limit.user_api.requests,
                Duration::from_secs(self.rate_limit.user_api.window_secs),
            ),
        }
    }
}

fn parse_i64(env: &impl EnvSource, key: &str) -> Option<i64> {
    env.get(key).and_then(|value| ind_config::parse_i64(&value))
}

fn parse_bool(env: &impl EnvSource, key: &str) -> Option<bool> {
    env.get(key)
        .and_then(|value| ind_config::parse_bool(&value))
}

fn parse_csv(env: &impl EnvSource, key: &str) -> Option<Vec<String>> {
    env.get(key).map(|value| {
        value
            .split(',')
            .flat_map(|part| part.split_whitespace())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    })
}

#[cfg(test)]
mod tests;
