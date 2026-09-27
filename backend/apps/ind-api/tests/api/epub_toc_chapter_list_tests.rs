use ind_application::repos::epub_format::EpubFormatIssue;
use ind_domain::DocumentId;
use ind_test_support::{AuthedClient, build_epub, epub_format, set_epub_format, spawn_app};
use reqwest::StatusCode;
use reqwest::multipart::{Form, Part};
use serde_json::Value;

use super::common::assert_json_response;

async fn upload_book(client: &AuthedClient<'_>) -> DocumentId {
    let epub = build_epub("Chapter List", &[("One", "First."), ("Two", "Second.")]);
    let form = Form::new().part(
        "file",
        Part::bytes(epub)
            .file_name("book.epub")
            .mime_str("application/epub+zip")
            .unwrap(),
    );
    let uploaded = assert_json_response(
        client.post_multipart("/api/v1/library/uploads", form).await,
        StatusCode::OK,
    )
    .await;
    uploaded["document_id"].as_str().unwrap().parse().unwrap()
}

async fn toc(client: &AuthedClient<'_>, document_id: DocumentId) -> (Value, String) {
    let response = client
        .get(&format!("/api/v1/documents/{document_id}/epub/toc"))
        .await;
    let cache_control = response.headers()[reqwest::header::CACHE_CONTROL]
        .to_str()
        .unwrap()
        .to_owned();
    (
        assert_json_response(response, StatusCode::OK).await,
        cache_control,
    )
}

#[tokio::test]
async fn new_epub_upload_is_version_2() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let document_id = upload_book(&client).await;

    assert_eq!(epub_format(app.pool(), document_id).await, (2, None));
    let (body, _) = toc(&client, document_id).await;
    assert_eq!(
        body["metadata"]["chapters"].as_array().map(Vec::len),
        Some(2)
    );
}

#[tokio::test]
async fn toc_reports_its_chapter_list_state_and_caches_only_ready() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let document_id = upload_book(&client).await;

    for (version, issue, expected, cache_control) in [
        (2, None, "ready", "private, max-age=3600"),
        (1, None, "pending", "private, no-store"),
        (
            1,
            Some(EpubFormatIssue::MissingChapter),
            "needs_reprocess",
            "private, no-store",
        ),
        (
            1,
            Some(EpubFormatIssue::NeedsReupload),
            "needs_reupload",
            "private, no-store",
        ),
    ] {
        set_epub_format(app.pool(), document_id, version, issue).await;
        let (body, cache) = toc(&client, document_id).await;
        assert_eq!(body["chapter_list"], expected);
        assert_eq!(cache, cache_control, "{expected}");
    }
}
