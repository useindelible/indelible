#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ind_test_support::TestDb;

/// Public RSSHub instances rot and every clone inherits the seed list, so no
/// rsshub row may ship enabled: YouTube rides the native Atom feed and RSSHub
/// is a deliberate operator opt-in. A new enabled seed row must clear this
/// test by proving the instance is operator-maintained.
#[tokio::test]
async fn no_rsshub_instance_ships_enabled() {
    let db = TestDb::new().await;

    let enabled: Vec<String> = sqlx::query_scalar(
        "SELECT base_url FROM feed_provider_instances \
         WHERE provider_type = 'rsshub' AND enabled",
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert!(
        enabled.is_empty(),
        "rsshub instances must ship disabled, found enabled: {enabled:?}"
    );

    // The official instance stays seeded as the documented opt-in.
    let official: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM feed_provider_instances \
         WHERE provider_type = 'rsshub' AND base_url = 'https://rsshub.app' AND NOT enabled",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(official, 1, "rsshub.app must remain seeded but disabled");
}
