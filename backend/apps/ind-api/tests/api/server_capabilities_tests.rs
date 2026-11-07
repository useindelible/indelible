use ind_test_support::spawn_app;
use reqwest::StatusCode;
use serde_json::json;

use super::common::assert_json_response;

#[tokio::test]
async fn capabilities_are_public_and_list_the_epub_chapter_list() {
    let app = spawn_app().await;

    let body =
        assert_json_response(app.get("/api/v1/server/capabilities").await, StatusCode::OK).await;

    assert_eq!(body, json!({ "capabilities": ["epub_chapter_list"] }));
}
