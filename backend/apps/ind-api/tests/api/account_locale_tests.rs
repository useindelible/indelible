use ind_test_support::TestApp;
use reqwest::StatusCode;

use super::common::assert_json_response;

#[tokio::test]
async fn profile_locale_distinguishes_system_default_from_an_explicit_preference() {
    let app = TestApp::new().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let initial = assert_json_response(client.get("/api/v1/me").await, StatusCode::OK).await;
    assert_eq!(initial["locale"], serde_json::Value::Null);

    let explicit = assert_json_response(
        client
            .patch_json(
                "/api/v1/me",
                &serde_json::json!({
                    "avatar_url": "https://example.com/avatar.png",
                    "locale": "fr-FR"
                }),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(explicit["locale"], "fr-FR");

    let unchanged = assert_json_response(
        client
            .patch_json(
                "/api/v1/me",
                &serde_json::json!({"display_name": "Locale User"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(unchanged["locale"], "fr-FR");

    let cleared = assert_json_response(
        client
            .patch_json(
                "/api/v1/me",
                &serde_json::json!({"avatar_url": null, "locale": null}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(cleared["locale"], serde_json::Value::Null);
    assert_eq!(cleared["avatar_url"], serde_json::Value::Null);

    assert_eq!(
        client
            .patch_json("/api/v1/me", &serde_json::json!({"locale": ""}))
            .await
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
}
