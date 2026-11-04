use ind_test_support::{AuthedClient, spawn_app};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::{assert_json_response, assert_status};

async fn save(client: &AuthedClient<'_>, url: &str) -> String {
    let saved = assert_json_response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": url, "title": "Unread", "item_type": "article"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    saved["document_id"].as_str().unwrap().to_string()
}

async fn mark_unread(client: &AuthedClient<'_>, document_id: &str) -> reqwest::Response {
    client
        .post_json(
            &format!("/api/v1/documents/{document_id}/mark-unread"),
            &json!({}),
        )
        .await
}

async fn reader(client: &AuthedClient<'_>, document_id: &str) -> Value {
    assert_json_response(
        client
            .get(&format!("/api/v1/documents/{document_id}"))
            .await,
        StatusCode::OK,
    )
    .await
}

#[tokio::test]
async fn mark_unread_clears_the_reader_state_for_the_owner_only() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let stranger = app.create_web_session().await;
    let owner_client = app.authed_client(&owner);
    let stranger_client = app.authed_client(&stranger);

    let document = save(&owner_client, "https://example.com/unread/owned").await;
    assert_status(
        owner_client
            .patch_json(
                &format!("/api/v1/documents/{document}/progress"),
                &json!({"progress_percent": 60.0}),
            )
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;
    assert!(reader(&owner_client, &document).await["last_read_at"].is_string());

    assert_status(
        mark_unread(&stranger_client, &document).await,
        StatusCode::NOT_FOUND,
    )
    .await;
    assert!(reader(&owner_client, &document).await["last_read_at"].is_string());

    assert_status(
        mark_unread(&owner_client, &document).await,
        StatusCode::NO_CONTENT,
    )
    .await;
    assert_status(
        mark_unread(&owner_client, &document).await,
        StatusCode::NO_CONTENT,
    )
    .await;

    let after = reader(&owner_client, &document).await;
    assert!(after["last_read_at"].is_null(), "{after}");
    assert!(after["progress_percent"].is_null(), "{after}");
}
