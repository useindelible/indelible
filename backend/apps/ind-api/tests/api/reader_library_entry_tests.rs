use ind_test_support::{AuthedClient, spawn_app};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::{assert_json_response, assert_status};

async fn save(client: &AuthedClient<'_>, url: &str) -> Value {
    assert_json_response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": url, "title": "Entry", "item_type": "article"}),
            )
            .await,
        StatusCode::OK,
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
async fn reader_reports_the_entrys_triage_state_and_saved_date() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let saved = save(&client, "https://example.com/reader/entry").await;
    let document_id = saved["document_id"].as_str().unwrap();
    let entry_id = saved["library_entry_id"].as_str().unwrap();

    let body = reader(&client, document_id).await;
    assert_eq!(body["triage_state"], saved["triage_state"]);
    assert_eq!(body["saved_at"], saved["saved_at"]);

    assert_status(
        client
            .post_json(
                &format!("/api/v1/library/{entry_id}/triage"),
                &json!({"triage_state": "later"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;

    assert_eq!(reader(&client, document_id).await["triage_state"], "later");
}

#[tokio::test]
async fn reader_omits_both_fields_for_a_document_that_left_the_library() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let saved = save(&client, "https://example.com/reader/removed").await;
    let document_id = saved["document_id"].as_str().unwrap();
    let entry_id = saved["library_entry_id"].as_str().unwrap();
    assert_status(
        client.delete(&format!("/api/v1/library/{entry_id}")).await,
        StatusCode::NO_CONTENT,
    )
    .await;

    let body = reader(&client, document_id).await;
    assert_eq!(body["saved"], false);
    assert!(body.get("triage_state").is_none(), "{body}");
    assert!(body.get("saved_at").is_none(), "{body}");
}
