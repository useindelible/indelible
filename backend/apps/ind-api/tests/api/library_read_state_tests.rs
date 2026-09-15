use ind_test_support::{AuthedClient, spawn_app};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::{assert_json_response, assert_status};

async fn save_response(client: &AuthedClient<'_>, url: &str) -> Value {
    assert_json_response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": url, "title": "Read state", "item_type": "article"}),
            )
            .await,
        StatusCode::OK,
    )
    .await
}

async fn save(client: &AuthedClient<'_>, url: &str) -> String {
    save_response(client, url).await["document_id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn field<'a>(entry: &'a Value, key: &str) -> &'a Value {
    entry
        .get(key)
        .unwrap_or_else(|| panic!("{key} must always be present: {entry}"))
}

async fn list_entry(client: &AuthedClient<'_>, document_id: &str) -> Value {
    let page = assert_json_response(
        client.post_json("/api/v1/library/query", &json!({})).await,
        StatusCode::OK,
    )
    .await;
    page["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["document_id"] == document_id)
        .cloned()
        .unwrap_or_else(|| panic!("entry {document_id} missing from {page}"))
}

#[tokio::test]
async fn library_list_carries_the_read_state_of_each_entry() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let unread = save(&client, "https://example.com/read-state/unread").await;
    let reading = save(&client, "https://example.com/read-state/reading").await;
    assert_status(
        client
            .patch_json(
                &format!("/api/v1/documents/{reading}/progress"),
                &json!({"progress_percent": 60.0}),
            )
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;

    let entry = list_entry(&client, &reading).await;
    assert_eq!(field(&entry, "progress_percent"), 60, "{entry}");
    assert_eq!(field(&entry, "max_progress_percent"), 60, "{entry}");
    assert!(field(&entry, "last_read_at").is_string(), "{entry}");
    assert!(field(&entry, "finished_at").is_null(), "{entry}");

    let entry = list_entry(&client, &unread).await;
    for key in [
        "progress_percent",
        "max_progress_percent",
        "last_read_at",
        "finished_at",
    ] {
        assert!(field(&entry, key).is_null(), "{key}: {entry}");
    }

    let resaved = save_response(&client, "https://example.com/read-state/reading").await;
    assert_eq!(resaved["document_id"], reading, "{resaved}");
    assert_eq!(field(&resaved, "progress_percent"), 60, "{resaved}");
    assert!(field(&resaved, "last_read_at").is_string(), "{resaved}");
}
