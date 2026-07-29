use std::path::PathBuf;

use config::{Config, File, FileFormat};
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RendererConfig {
    pub server: RendererServerSettings,
    pub chromium: ChromiumSettings,
    pub capture: CaptureSettings,
    pub s3: S3Settings,
    #[serde(default)]
    pub egress: EgressSettings,
}

#[derive(Clone, Deserialize)]
pub struct CaptureSettings {
    pub max_concurrency: usize,
    pub locale: String,
    pub timezone: String,
}

#[derive(Default, Deserialize)]
pub struct EgressSettings {
    /// When true, navigation may target private/loopback hosts. Defaults to
    /// true in development (local test pages) and false elsewhere; the renderer
    /// container should also be network-isolated from internal hosts in prod.
    #[serde(default)]
    pub allow_private_targets: bool,
}

impl RendererConfig {
    pub fn egress_policy(&self) -> ind_egress::EgressPolicy {
        ind_egress::EgressPolicy {
            allow_private_targets: self.egress.allow_private_targets,
            extra_allowed_ips: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
pub struct RendererServerSettings {
    pub environment: String,
    pub log_level: String,
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct ChromiumSettings {
    pub path: PathBuf,
    pub single_process: bool,
    pub virtual_time_budget: Option<u32>,
    pub idle_timeout_secs: u64,
}

#[derive(Deserialize)]
pub struct S3Settings {
    pub endpoint: Option<String>,
    pub region: String,
    pub bucket: String,
    pub access_key: Option<SecretString>,
    pub secret_key: Option<SecretString>,
    pub force_path_style: bool,
}

impl RendererConfig {
    /// Loads configuration from `configurations/base.toml` and
    /// `configurations/{environment}.toml` (both optional), merged with environment variables.
    ///
    /// Priority (highest to lowest):
    ///   env vars → configurations/{env}.toml → configurations/base.toml → built-in defaults.
    ///
    /// IND_ENV always wins for environment resolution; if absent, `[server].environment` inside
    /// `configurations/base.toml` is used so that file-only deployments select the right overlay.
    pub fn load() -> anyhow::Result<Self> {
        Self::load_from_env(&|key| std::env::var(key).ok())
    }

    fn load_from_env(env: &impl Fn(&str) -> Option<String>) -> anyhow::Result<Self> {
        // IND_ENV wins; if absent, pre-read configurations/base.toml to select the right overlay.
        let environment = env("IND_ENV").unwrap_or_else(|| {
            Config::builder()
                .add_source(File::new("configurations/base.toml", FileFormat::Toml).required(false))
                .build()
                .ok()
                .and_then(|c| c.get_string("server.environment").ok())
                .unwrap_or_else(|| "development".to_string())
        });
        let chromium_default = detect_chromium().to_string_lossy().into_owned();

        let cfg: Self = Config::builder()
            // Built-in defaults
            .set_default("server.environment", "development")?
            .set_default("server.log_level", "info")?
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 3100_i64)?
            .set_default("chromium.path", chromium_default)?
            .set_default("chromium.single_process", false)?
            .set_default("chromium.idle_timeout_secs", 30_i64)?
            .set_default("capture.max_concurrency", 1_i64)?
            .set_default("capture.locale", "en-US")?
            .set_default("capture.timezone", "UTC")?
            .set_default("s3.region", "us-east-1")?
            .set_default("s3.bucket", "indelible")?
            .set_default("s3.force_path_style", true)?
            .set_default("egress.allow_private_targets", environment == "development")?
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
            .set_override_option("server.environment", env("IND_ENV"))?
            .set_override_option("server.log_level", env("RUST_LOG"))?
            .set_override_option("server.host", env("RENDERER_HOST"))?
            .set_override_option("server.port", parse_i64(env("RENDERER_PORT")))?
            .set_override_option("chromium.path", env("CHROMIUM_PATH"))?
            // Only override when the env var is actually set (avoid clobbering file values).
            .set_override_option(
                "chromium.single_process",
                parse_bool(env("CHROMIUM_SINGLE_PROCESS")),
            )?
            .set_override_option(
                "chromium.virtual_time_budget",
                parse_i64(env("CHROMIUM_VIRTUAL_TIME_BUDGET")),
            )?
            .set_override_option(
                "chromium.idle_timeout_secs",
                parse_i64(env("IDLE_TIMEOUT_SECS")),
            )?
            .set_override_option(
                "capture.max_concurrency",
                parse_i64(env("CAPTURE_MAX_CONCURRENCY"))
                    .or_else(|| parse_i64(env("FEED_PREFETCH_MAX_CONCURRENCY"))),
            )?
            .set_override_option("capture.locale", env("CAPTURE_LOCALE"))?
            .set_override_option("capture.timezone", env("CAPTURE_TIMEZONE"))?
            .set_override_option("s3.endpoint", env("S3_ENDPOINT"))?
            .set_override_option("s3.region", env("S3_REGION"))?
            .set_override_option("s3.bucket", env("S3_BUCKET"))?
            .set_override_option("s3.access_key", env("S3_ACCESS_KEY"))?
            .set_override_option("s3.secret_key", env("S3_SECRET_KEY"))?
            // Only override when the env var is actually set (avoid clobbering file values).
            .set_override_option(
                "s3.force_path_style",
                parse_bool(env("S3_FORCE_PATH_STYLE")),
            )?
            .set_override_option(
                "egress.allow_private_targets",
                parse_bool(env("EGRESS_ALLOW_PRIVATE_TARGETS")),
            )?
            .build()?
            .try_deserialize()?;

        if cfg.capture.max_concurrency == 0 {
            anyhow::bail!("capture.max_concurrency must be greater than zero");
        }

        Ok(cfg)
    }
}

/// Returns `Some(bool)` only when the env var is present, so callers can use
/// `set_override_option` and avoid clobbering values from the config file.
fn parse_bool(value: Option<String>) -> Option<bool> {
    value.map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

fn parse_i64(value: Option<String>) -> Option<i64> {
    value.and_then(|value| value.parse().ok())
}

fn detect_chromium() -> PathBuf {
    let candidates = [
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/headless-shell/headless-shell",
    ];

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return PathBuf::from(path);
        }
    }

    if let Ok(output) = std::process::Command::new("which").arg("chromium").output()
        && output.status.success()
    {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return PathBuf::from(path);
        }
    }

    PathBuf::from("chromium")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn load(values: &[(&str, &str)]) -> anyhow::Result<RendererConfig> {
        let values: HashMap<_, _> = values
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();
        RendererConfig::load_from_env(&|key| values.get(key).cloned())
    }

    #[test]
    fn capture_defaults_and_env_precedence_are_stable() {
        let defaults = load(&[]).unwrap();
        assert_eq!(
            (
                defaults.capture.max_concurrency,
                defaults.capture.locale.as_str(),
                defaults.capture.timezone.as_str()
            ),
            (1, "en-US", "UTC")
        );
        let overridden = load(&[
            ("CAPTURE_MAX_CONCURRENCY", "3"),
            ("FEED_PREFETCH_MAX_CONCURRENCY", "2"),
            ("CAPTURE_LOCALE", "fr-FR"),
            ("CAPTURE_TIMEZONE", "Europe/Paris"),
        ])
        .unwrap();
        assert_eq!(
            (
                overridden.capture.max_concurrency,
                overridden.capture.locale.as_str(),
                overridden.capture.timezone.as_str()
            ),
            (3, "fr-FR", "Europe/Paris")
        );
        assert_eq!(
            load(&[("FEED_PREFETCH_MAX_CONCURRENCY", "2")])
                .unwrap()
                .capture
                .max_concurrency,
            2
        );
    }

    #[test]
    fn zero_capture_concurrency_is_rejected() {
        let error = load(&[("CAPTURE_MAX_CONCURRENCY", "0")])
            .err()
            .expect("zero concurrency must fail");
        assert!(
            error
                .to_string()
                .contains("capture.max_concurrency must be greater than zero")
        );
    }
}
