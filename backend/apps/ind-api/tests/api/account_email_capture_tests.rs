#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ind_test_support::{TestAppOptions, spawn_app, spawn_app_with_options};

/// A client cannot distinguish "profile still loading" from "email capture is
/// not configured on this instance" unless the address fields are always
/// present: null must mean unavailable, a string must mean usable.
#[tokio::test]
async fn profile_reports_null_capture_addresses_when_no_domain_is_configured() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;

    let response = app.authed_client(&session).get("/api/v1/me").await;
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(
        body.get("ingest_email"),
        Some(&serde_json::Value::Null),
        "ingest_email must be present and null when unconfigured, got body {body}"
    );
    assert_eq!(
        body.get("ingest_library_email"),
        Some(&serde_json::Value::Null),
        "ingest_library_email must be present and null when unconfigured, got body {body}"
    );
}

#[tokio::test]
async fn profile_reports_capture_addresses_when_domains_are_configured() {
    let app = spawn_app_with_options(TestAppOptions {
        email_ingest_domains: Some(("feed.test.example".into(), "library.test.example".into())),
        ..TestAppOptions::default()
    })
    .await;
    let session = app.create_web_session().await;

    let response = app.authed_client(&session).get("/api/v1/me").await;
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();

    let feed = body["ingest_email"].as_str().unwrap();
    let library = body["ingest_library_email"].as_str().unwrap();
    assert!(
        feed.ends_with("@feed.test.example"),
        "feed address must use the configured domain, got {feed}"
    );
    assert!(
        library.ends_with("@library.test.example"),
        "library address must use the configured domain, got {library}"
    );
}

#[tokio::test]
async fn profile_reports_distinct_routable_addresses_on_a_shared_domain() {
    let app = spawn_app_with_options(TestAppOptions {
        email_ingest_domains: Some(("shared.test.example".into(), "shared.test.example".into())),
        ..TestAppOptions::default()
    })
    .await;
    let session = app.create_web_session().await;

    let response = app.authed_client(&session).get("/api/v1/me").await;
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();

    assert!(
        body["ingest_email"]
            .as_str()
            .unwrap()
            .ends_with("-feed@shared.test.example")
    );
    assert!(
        body["ingest_library_email"]
            .as_str()
            .unwrap()
            .ends_with("-lib@shared.test.example")
    );
}
