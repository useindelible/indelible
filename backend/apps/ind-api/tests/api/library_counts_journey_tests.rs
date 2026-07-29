use ind_test_support::{AuthedClient, spawn_app};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::{assert_json_response as response, assert_status};

async fn save(client: &AuthedClient<'_>, url: &str, item_type: &str) -> (String, String) {
    let saved = response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": url, "title": "Counts", "item_type": item_type}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    (
        saved["library_entry_id"].as_str().unwrap().to_string(),
        saved["document_id"].as_str().unwrap().to_string(),
    )
}

async fn record_progress(client: &AuthedClient<'_>, document_id: &str, percent: f64) {
    assert_status(
        client
            .patch_json(
                &format!("/api/v1/documents/{document_id}/progress"),
                &json!({"progress_percent": percent}),
            )
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;
}

fn item_type_count(counts: &Value, item_type: &str) -> i64 {
    counts["by_item_type"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["item_type"] == item_type)
        .map(|entry| entry["count"].as_i64().unwrap())
        .unwrap_or(0)
}

#[tokio::test]
async fn library_counts_bucket_read_state_and_item_types_per_scope() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let (finished_entry, finished_doc) =
        save(&client, "https://example.com/counts/finished", "article").await;
    let (_, started_doc) = save(&client, "https://example.com/counts/started", "article").await;
    save(&client, "https://example.com/counts/untouched", "video").await;

    record_progress(&client, &finished_doc, 100.0).await;
    record_progress(&client, &started_doc, 40.0).await;

    let counts = response(client.get("/api/v1/library/counts").await, StatusCode::OK).await;
    assert_eq!(counts["total"], 3);
    assert_eq!(counts["done"], 1);
    assert_eq!(counts["reading"], 1);
    assert_eq!(counts["unread"], 1);
    assert_eq!(item_type_count(&counts, "article"), 2);
    assert_eq!(item_type_count(&counts, "video"), 1);
    assert_eq!(item_type_count(&counts, "pdf"), 0);

    assert_status(
        client
            .post_json(
                &format!("/api/v1/library/{finished_entry}/triage"),
                &json!({"triage_state": "archive"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;

    let archived = response(
        client
            .get("/api/v1/library/counts?triage_state=archive")
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(archived["total"], 1);
    assert_eq!(archived["done"], 1);
    assert_eq!(item_type_count(&archived, "article"), 1);

    let inbox = response(
        client
            .get("/api/v1/library/counts?triage_state=inbox")
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(inbox["total"], 2);
    assert_eq!(inbox["reading"], 1);
    assert_eq!(inbox["unread"], 1);

    assert_status(
        client.get("/api/v1/library/counts?triage_state=nope").await,
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;

    assert_status(
        client
            .delete(&format!("/api/v1/library/{finished_entry}"))
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;
    let after_trash = response(client.get("/api/v1/library/counts").await, StatusCode::OK).await;
    assert_eq!(after_trash["total"], 2);
    assert_eq!(after_trash["done"], 0);
}
