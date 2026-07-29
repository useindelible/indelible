#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ind_test_support::{TestAppOptions, spawn_app, spawn_app_with_options};
use serde_json::json;

/// A self-hosted instance without Notion credentials must tell clients so
/// through the catalog, and an authorize attempt must fail with actionable
/// operator guidance rather than a leaked persistence identifier.
#[tokio::test]
async fn unconfigured_instance_reports_no_oauth_providers_and_rejects_authorize() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let response = client.get("/api/v1/integrations").await;
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        body["available_oauth_providers"],
        json!([]),
        "unconfigured instance must report an empty provider list, got {body}"
    );

    let response = client
        .post_json("/api/v1/integrations/notion/authorize", &json!({}))
        .await;
    assert_eq!(
        response.status(),
        503,
        "authorize against an unconfigured provider must be a service-unavailable error"
    );
    let error_body = response.text().await.unwrap();
    assert!(
        !error_body.contains("integration_provider not found"),
        "the internal entity identifier must not leak to clients, got {error_body}"
    );
    assert!(
        error_body.contains("NOTION_CLIENT_ID") && error_body.contains("AUTH_CREDENTIAL_KEY"),
        "the error must name every operator setup step, got {error_body}"
    );
}

#[tokio::test]
async fn configured_instance_lists_notion_as_available() {
    let app = spawn_app_with_options(TestAppOptions {
        notion_oauth_configured: true,
        ..TestAppOptions::default()
    })
    .await;
    let session = app.create_web_session().await;

    let response = app
        .authed_client(&session)
        .get("/api/v1/integrations")
        .await;
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        body["available_oauth_providers"],
        json!(["notion"]),
        "configured Notion must appear in the availability list, got {body}"
    );
}

/// Notion OAuth tokens are sealed with the credential cipher, so an instance
/// with Notion credentials but no AUTH_CREDENTIAL_KEY cannot complete the
/// flow. Advertising Connect there sends the user through authorization only
/// to fail at the callback.
#[tokio::test]
async fn notion_is_unavailable_without_a_credential_key() {
    let app = spawn_app_with_options(TestAppOptions {
        notion_oauth_configured: true,
        credential_key_configured: false,
        ..TestAppOptions::default()
    })
    .await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let response = client.get("/api/v1/integrations").await;
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        body["available_oauth_providers"],
        json!([]),
        "an adapter without a cipher cannot complete a connection, got {body}"
    );

    let response = client
        .post_json("/api/v1/integrations/notion/authorize", &json!({}))
        .await;
    assert_eq!(response.status(), 503);
    let error_body = response.text().await.unwrap();
    assert!(
        error_body.contains("AUTH_CREDENTIAL_KEY"),
        "the error must name every prerequisite, got {error_body}"
    );
}
