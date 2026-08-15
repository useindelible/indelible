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

#[test]
fn defaults_and_overrides_are_loaded() {
    let config = WorkerConfig::load_from_env(&TestEnv::new(&[
        ("WORKER_MAX_CONCURRENCY", "4"),
        ("WORKER_CLAIM_BUFFER_SIZE", "8"),
        ("CAPTURE_MAX_CONCURRENCY", "3"),
        ("NOTION_EXPORT_MAX_CONCURRENCY", "2"),
        ("NOTION_SYNC_MAX_CONCURRENCY", "1"),
        ("AUTO_HEAL_TTS_ORPHAN_PAGE_SIZE", "250"),
        ("TRASH_CLEANUP_RETENTION_DAYS", "14"),
        ("EGRESS_ALLOW_PRIVATE_TARGETS", "no"),
    ]))
    .unwrap();
    assert_eq!(
        (
            config.worker.max_concurrency,
            config.worker.claim_buffer_size
        ),
        (4, 8)
    );
    assert_eq!(config.capture.max_concurrency, 3);
    assert_eq!(config.integrations.notion.export_max_concurrency, 2);
    assert_eq!(config.integrations.notion.sync_max_concurrency, 1);
    assert_eq!(config.auto_heal.tts_orphan_page_size, 250);
    assert_eq!(config.trash_cleanup.retention_days, 14);
    assert!(!config.egress.allow_private_targets);
    assert_eq!(config.feed.default_poll_interval_minutes, 15);
    assert_eq!(config.mila.model_context_window, 12000);
    assert_eq!(config.mila.summary_max_output_tokens, 1024);
    assert_eq!(config.mila.tags_max_output_tokens, 1024);
    assert_eq!(config.mila.entities_max_output_tokens, 2000);
    assert_eq!(config.mila.chat_max_output_tokens, 1024);
}

#[test]
fn mila_output_budgets_accept_environment_overrides() {
    let config = WorkerConfig::load_from_env(&TestEnv::new(&[
        ("MILA_SUMMARY_MAX_OUTPUT_TOKENS", "2048"),
        ("MILA_TAGS_MAX_OUTPUT_TOKENS", "1536"),
        ("MILA_ENTITIES_MAX_OUTPUT_TOKENS", "8000"),
        ("MILA_CHAT_MAX_OUTPUT_TOKENS", "4096"),
    ]))
    .unwrap();

    assert_eq!(config.mila.summary_max_output_tokens, 2048);
    assert_eq!(config.mila.tags_max_output_tokens, 1536);
    assert_eq!(config.mila.entities_max_output_tokens, 8000);
    assert_eq!(config.mila.chat_max_output_tokens, 4096);
}

#[test]
fn invalid_limits_are_rejected() {
    for (key, value, expected) in [
        ("WORKER_MAX_CONCURRENCY", "0", "positive integer"),
        ("WORKER_CLAIM_BUFFER_SIZE", "0", "positive integer"),
        ("NOTION_EXPORT_MAX_CONCURRENCY", "0", "positive integer"),
        ("NOTION_SYNC_MAX_CONCURRENCY", "0", "positive integer"),
        ("TRASH_CLEANUP_RETENTION_DAYS", "0", "retention_days"),
        ("MILA_MODEL_CONTEXT_WINDOW", "0", "model_context_window"),
        ("MILA_CHAT_CONTEXT_PCT", "150", "chat_context_pct"),
        (
            "MILA_ENTITIES_MAX_OUTPUT_TOKENS",
            "0",
            "entities_max_output_tokens",
        ),
    ] {
        let error = WorkerConfig::load_from_env(&TestEnv::new(&[(key, value)])).unwrap_err();
        assert!(error.to_string().contains(expected), "{key}: {error}");
    }
}

#[test]
fn s3_boolean_grammar_and_endpoint_contract_are_preserved() {
    for (raw, expected) in [
        ("true", true),
        ("yes", true),
        ("1", true),
        ("false", false),
        ("no", false),
        ("0", false),
    ] {
        let config = WorkerConfig::load_from_env(&TestEnv::new(&[("S3_ENABLED", raw)])).unwrap();
        assert_eq!(config.s3_enabled, expected, "{raw}");
    }
    let config = WorkerConfig::load_from_env(&TestEnv::new(&[("S3_ENABLED", "true")])).unwrap();
    assert_eq!(
        config.s3_config().unwrap_err().to_string(),
        "S3_ENDPOINT is required when S3_ENABLED=true"
    );
}
