use ind_test_support::{UserFactory, spawn_app};
use reqwest::StatusCode;
use serde_json::json;

use super::common::{assert_json_response, assert_status};

#[tokio::test]
async fn home_alias_and_webhook_settings_persist_with_tenant_isolation() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let stranger = app.create_web_session().await;
    let owner_client = app.authed_client(&owner);
    let stranger_client = app.authed_client(&stranger);

    let saved = assert_json_response(
        owner_client
            .post_json(
                "/api/v1/library",
                &json!({"url": "https://example.com/home-platform", "title": "Home Platform"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let dashboard = assert_json_response(
        owner_client
            .get("/api/v1/home?widgets=recently_added,reading_stats")
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        dashboard["recently_added"]["items"][0]["id"],
        saved["document_id"]
    );
    assert!(dashboard.get("continue_reading").is_none());
    assert!(dashboard["reading_stats"]["documents_read"].is_number());

    let settings = assert_json_response(
        owner_client
            .patch_json(
                "/api/v1/settings/home",
                &json!({
                    "widget_order": ["recently_added", "reading_stats"],
                    "hidden_widgets": ["feed_digest"]
                }),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let persisted = assert_json_response(
        owner_client.get("/api/v1/settings/home").await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(persisted, settings);

    let alias = assert_json_response(
        owner_client
            .post_json(
                "/api/v1/email-aliases",
                &json!({
                    "destination": "library",
                    "local_part": "platform.journey",
                    "is_default": false
                }),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(alias["local_part"], "platform.journey");
    assert_eq!(alias["destination"], "library");
    let alias_id = alias["id"].as_str().expect("alias id");
    assert_status(
        stranger_client
            .delete(&format!("/api/v1/email-aliases/{alias_id}"))
            .await,
        StatusCode::NOT_FOUND,
    )
    .await;
    let aliases = assert_json_response(
        owner_client.get("/api/v1/email-aliases").await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(aliases["data"][0]["id"], alias_id);

    let webhook = assert_json_response(
        owner_client
            .post_json(
                "/api/v1/webhooks",
                &json!({
                    "name": "Library events",
                    "url": "https://127.0.0.1:1/indelible-webhook",
                    "events": ["library_entry.saved"],
                    "is_active": true
                }),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let webhook_id = webhook["id"].as_str().expect("webhook id");
    let original_secret = webhook["raw_secret"].as_str().expect("webhook secret");
    assert_status(
        stranger_client
            .patch_json(
                &format!("/api/v1/webhooks/{webhook_id}"),
                &json!({"is_active": false}),
            )
            .await,
        StatusCode::NOT_FOUND,
    )
    .await;
    let paused = assert_json_response(
        owner_client
            .patch_json(
                &format!("/api/v1/webhooks/{webhook_id}"),
                &json!({"name": "Paused events", "is_active": false}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(paused["last_status"], "paused");
    assert_json_response(
        owner_client
            .patch_json(
                &format!("/api/v1/webhooks/{webhook_id}"),
                &json!({"is_active": true}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let rotated = assert_json_response(
        owner_client
            .post_json(
                &format!("/api/v1/webhooks/{webhook_id}/rotate-secret"),
                &json!({}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_ne!(rotated["raw_secret"], original_secret);
    let hooks =
        assert_json_response(owner_client.get("/api/v1/webhooks").await, StatusCode::OK).await;
    assert_eq!(hooks["data"][0]["events"][0], "library_entry.saved");
    let delivery = assert_json_response(
        owner_client
            .post_json(
                &format!("/api/v1/webhooks/{webhook_id}/test"),
                &json!({"event": "library_entry.saved"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert!(delivery["status_code"].is_null());
    assert_eq!(delivery["attempt"], 1);
    let deliveries = assert_json_response(
        owner_client
            .get(&format!("/api/v1/webhooks/{webhook_id}/deliveries"))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(deliveries["data"][0]["id"], delivery["id"]);
    assert_eq!(deliveries["data"][0]["target"], webhook["url"]);
    assert_status(
        owner_client
            .delete(&format!("/api/v1/webhooks/{webhook_id}"))
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;
}

#[tokio::test]
async fn email_alias_creation_preserves_namespace_and_default_atomicity() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let client = app.authed_client(&owner);

    assert_status(
        client
            .post_json(
                "/api/v1/email-aliases",
                &json!({
                    "destination": "feed",
                    "local_part": "bad!chars",
                    "is_default": false
                }),
            )
            .await,
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;

    let original = assert_json_response(
        client
            .post_json(
                "/api/v1/email-aliases",
                &json!({
                    "destination": "feed",
                    "local_part": "stable.default",
                    "is_default": true
                }),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    assert_status(
        client
            .post_json(
                "/api/v1/email-aliases",
                &json!({
                    "destination": "feed",
                    "local_part": "stable.default",
                    "is_default": true
                }),
            )
            .await,
        StatusCode::CONFLICT,
    )
    .await;

    let aliases =
        assert_json_response(client.get("/api/v1/email-aliases").await, StatusCode::OK).await;
    let original_after_failure = aliases["data"]
        .as_array()
        .expect("alias list")
        .iter()
        .find(|alias| alias["id"] == original["id"])
        .expect("original default alias");
    assert_eq!(original_after_failure["is_default"], true);
    assert!(original_after_failure["retire_at"].is_null());

    let victim = UserFactory::default().insert(app.pool()).await;
    assert_status(
        client
            .post_json(
                "/api/v1/email-aliases",
                &json!({
                    "destination": "feed",
                    "local_part": victim.email_token,
                    "is_default": false
                }),
            )
            .await,
        StatusCode::CONFLICT,
    )
    .await;
}

#[tokio::test]
async fn avatar_round_trip_stays_owner_scoped_through_the_api() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let stranger = app.create_web_session().await;
    let client = app.authed_client(&session);
    let avatar = b"\x89PNG\r\n\x1a\nindelible-avatar";

    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(avatar.to_vec())
            .file_name("avatar.png")
            .mime_str("image/png")
            .expect("valid mime"),
    );
    let profile = assert_json_response(
        client.post_multipart("/api/v1/me/avatar", form).await,
        StatusCode::OK,
    )
    .await;
    let read_url = profile["avatar_url"].as_str().expect("avatar URL");
    let api_prefix = format!("{}/api/v1/assets/", app.address);
    assert!(
        read_url.starts_with(&api_prefix),
        "avatar_url must target the asset proxy, got {read_url}"
    );

    let proxy_path = &read_url[app.address.len()..];
    let proxied = client.get(proxy_path).await;
    assert_eq!(proxied.status(), StatusCode::OK);
    assert_eq!(
        proxied.headers()[reqwest::header::CACHE_CONTROL],
        "private, max-age=3600"
    );
    assert_eq!(proxied.bytes().await.unwrap(), avatar.as_slice());
    assert_eq!(
        app.authed_client(&stranger).get(proxy_path).await.status(),
        StatusCode::NOT_FOUND
    );
}
