use std::collections::HashMap;

use super::*;

struct TestEnv(HashMap<String, String>);

impl TestEnv {
    fn new(vars: &[(&str, &str)]) -> Self {
        let mut values =
            HashMap::from([("DATABASE_URL".into(), "postgres://localhost/test".into())]);
        values.extend(
            vars.iter()
                .map(|(key, value)| ((*key).into(), (*value).into())),
        );
        Self(values)
    }
}

impl EnvSource for TestEnv {
    fn get(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
}

fn load(vars: &[(&str, &str)]) -> anyhow::Result<ServerConfig> {
    ServerConfig::load_from_env(&TestEnv::new(vars))
}

fn production(extra: &[(&str, &str)]) -> anyhow::Result<ServerConfig> {
    let mut vars = vec![
        ("IND_ENV", "production"),
        ("CSRF_SECRET", "prod-csrf-secret-not-default"),
        ("JWT_SECRET", "prod-jwt-secret-not-default-32-bytes!"),
        ("IND_BASE_URL", "https://ind.example.com"),
        ("FRONTEND_URL", "https://app.example.com"),
        ("CORS_ORIGINS", "https://app.example.com"),
        (
            "ASSET_COOKIE_SECRET",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ),
    ];
    vars.extend_from_slice(extra);
    load(&vars)
}

#[test]
fn environment_projection_and_shared_boolean_grammar() {
    let config = load(&[
        ("IND_HOST", "127.0.0.1"),
        ("IND_PORT", "8080"),
        ("CORS_ORIGINS", "https://one.example, https://two.example"),
        ("APPLE_CLIENT_SECRET", "legacy-apple-secret"),
        ("S3_ENDPOINT", "http://localhost:9000"),
        ("MILA_ENABLED", "yes"),
        ("TTS_USE_MOCK_ADAPTER", "1"),
        ("TTS_DASHSCOPE_TRANSCRIPT_SUPPORTED", "false"),
        ("TTS_UNREAL_SPEECH_TRANSCRIPT_SUPPORTED", "true"),
    ])
    .unwrap();
    assert_eq!(
        (config.server.host.as_str(), config.server.port),
        ("127.0.0.1", 8080)
    );
    assert_eq!(
        config.cors.origins,
        ["https://one.example", "https://two.example"]
    );
    assert_eq!(
        config
            .oauth
            .apple_private_key_pem
            .as_ref()
            .map(|secret| secret.expose_secret()),
        Some("legacy-apple-secret")
    );
    assert!(config.storage.s3_enabled);
    assert!(config.mila.enabled);
    assert!(config.tts.use_mock_adapter);
    assert!(!config.tts.dashscope.transcript_supported);
    assert!(config.tts.unreal_speech.transcript_supported);

    for (raw, expected) in [
        ("true", true),
        ("yes", true),
        ("1", true),
        ("false", false),
        ("no", false),
        ("0", false),
    ] {
        assert_eq!(
            load(&[("S3_ENABLED", raw)]).unwrap().storage.s3_enabled,
            expected
        );
    }
}

#[test]
fn extension_redirect_uris_default_to_the_fixed_store_identity_callbacks() {
    let config = load(&[]).unwrap();
    assert_eq!(
        config.extension.redirect_uris,
        [
            "https://lblngpkieoichinegfhgacmcjbahjbek.chromiumapp.org/indelible",
            "https://38bd18db5de5caccb6ab6c1271fec03ec1662d5c.extensions.allizom.org/indelible",
        ]
    );
}

#[test]
fn extension_redirect_uris_environment_replaces_defaults() {
    let config = load(&[(
        "EXTENSION_REDIRECT_URIS",
        "https://abcdefghijklmnop.chromiumapp.org/indelible, https://edge.example/indelible",
    )])
    .unwrap();
    assert_eq!(
        config.extension.redirect_uris,
        [
            "https://abcdefghijklmnop.chromiumapp.org/indelible",
            "https://edge.example/indelible",
        ]
    );
}

#[test]
fn invalid_extension_redirect_uris_fail_at_startup() {
    for value in [
        "",
        "not a URL",
        "http://abcdefghijklmnop.chromiumapp.org/indelible",
        "https://*.chromiumapp.org/indelible",
        "https://user:pass@example.com/indelible",
        "https://example.com/indelible?code=1",
        "https://example.com/indelible#callback",
        "https://example.com/indelible,https://EXAMPLE.com/indelible",
    ] {
        let error = load(&[("EXTENSION_REDIRECT_URIS", value)])
            .err()
            .unwrap_or_else(|| panic!("invalid extension callback was accepted: {value}"));
        assert!(error.to_string().contains("EXTENSION_REDIRECT_URIS"));
    }
}

#[test]
fn mila_model_context_window_defaults_when_unset() {
    let config = load(&[]).unwrap();
    assert_eq!(config.mila.model_context_window, 12000);
}

#[test]
fn assets_default_to_passthrough_so_self_hosters_need_no_public_object_store() {
    let config = load(&[]).unwrap();
    assert!(matches!(
        config.storage.asset_serving_mode,
        AssetServingMode::Passthrough
    ));
}

#[test]
fn production_rejects_plaintext_http_origins() {
    for (key, value) in [
        ("IND_BASE_URL", "http://ind.lan"),
        ("FRONTEND_URL", "http://ind.lan"),
    ] {
        let error = production(&[(key, value)])
            .err()
            .unwrap_or_else(|| panic!("{key} over http must be rejected in production"));
        assert!(error.to_string().contains(key), "{key}: {error}");
    }
}

#[test]
fn invalid_mila_budgets_are_rejected() {
    for (key, value, expected) in [
        ("MILA_MODEL_CONTEXT_WINDOW", "0", "model_context_window"),
        ("MILA_CHAT_CONTEXT_PCT", "150", "chat_context_pct"),
    ] {
        let error = load(&[(key, value)])
            .err()
            .expect("invalid budget must fail");
        assert!(error.to_string().contains(expected), "{key}: {error}");
    }
}

#[test]
fn production_provider_configuration_fails_closed() {
    for (name, vars) in [
        ("resend", &[("EMAIL_INGEST_PROVIDER", "resend")][..]),
        ("google", &[("GOOGLE_CLIENT_ID", "id")][..]),
        ("oidc", &[("OIDC_ENABLED", "true")][..]),
        ("notion", &[("NOTION_CLIENT_ID", "id")][..]),
    ] {
        assert!(
            production(vars).is_err(),
            "{name} accepted incomplete production config"
        );
    }
    assert!(production(&[]).is_ok());
}

#[test]
fn enabled_s3_requires_an_endpoint() {
    let config = load(&[("S3_ENABLED", "true")]).unwrap();
    assert_eq!(
        config.storage.s3_config().unwrap_err().to_string(),
        "S3_ENDPOINT is required when S3_ENABLED=true"
    );
}
