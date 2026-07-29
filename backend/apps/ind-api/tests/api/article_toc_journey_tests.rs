use reqwest::StatusCode;
use serde_json::json;

use super::common::{SaveScenario, assert_json_response, document_id_from_response};

use ind_application::repos::document_asset::DocumentAssetRepository;
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, DocumentId, NewDocumentAsset};
use ind_persistence::repos::PgDocumentAssetRepository;

const SECTIONED_HTML: &str = "<article><h2>History</h2><p>one two three four</p>\
     <h3>Origins</h3><p>five six</p><h2>Structure</h2><p>seven eight nine</p></article>";

const TOC_ENSURE: &str = "document.toc.ensure";
const ATTACH: &str = "document.attach_provided_content";

async fn reader_save_with_html(scenario: &SaveScenario, url: &str, html: &str) -> String {
    let resp = scenario
        .extension_client()
        .post_json(
            "/api/v1/extension/reader-save",
            &json!({
                "url": url,
                "title": "Sectioned Article",
                "reader_html": html,
                "word_count": 20,
                "reading_time_minutes": 1,
                "language": "en"
            }),
        )
        .await;
    let body = assert_json_response(resp, StatusCode::ACCEPTED).await;
    document_id_from_response(&body)
}

fn parse_doc_uuid(document_id: &str) -> DocumentId {
    let raw = document_id.strip_prefix("doc_").expect("doc_ prefix");
    DocumentId::from_uuid(raw.parse().expect("uuid"))
}

async fn get_toc(scenario: &SaveScenario, document_id: &str) -> (StatusCode, serde_json::Value) {
    let resp = scenario
        .web_client()
        .get(&format!("/api/v1/documents/{document_id}/toc"))
        .await;
    let status = resp.status();
    let body = resp.json().await.expect("toc response json");
    (status, body)
}

#[tokio::test]
async fn reader_save_derives_toc_inline_and_serves_ready() {
    let scenario = SaveScenario::new().await;
    let document_id =
        reader_save_with_html(&scenario, "https://example.com/toc/inline", SECTIONED_HTML).await;
    scenario.run_pending_jobs_of_type(ATTACH).await;

    let resp = scenario
        .web_client()
        .get(&format!("/api/v1/documents/{document_id}/toc"))
        .await;
    assert_eq!(
        resp.headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("private, max-age=3600")
    );
    let body = assert_json_response(resp, StatusCode::OK).await;
    assert_eq!(body["status"], "ready");
    assert_eq!(body["truncated"], false);
    let entries = body["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["title"], "History");
    assert_eq!(entries[0]["id"], "ind-toc-history");
    assert_eq!(entries[0]["depth"], 0);
    assert_eq!(entries[1]["title"], "Origins");
    assert_eq!(entries[1]["depth"], 1);
    assert_eq!(entries[2]["word_count"], 3);
    // Inline derivation means the read path never had to enqueue a job.
    assert_eq!(scenario.total_job_count_by_type(TOC_ENSURE).await, 0);
}

#[tokio::test]
async fn stale_toc_backfills_through_pending_and_deduped_ensure_job() {
    let scenario = SaveScenario::new().await;
    let document_id = reader_save_with_html(
        &scenario,
        "https://example.com/toc/backfill",
        SECTIONED_HTML,
    )
    .await;
    scenario.run_pending_jobs_of_type(ATTACH).await;

    // Simulate legacy/stale content: overwrite the readable asset with an
    // unprepared object at a fresh version stamp.
    let storage = scenario.app.storage().await;
    let legacy_key = format!("legacy/{document_id}/readable_html.html");
    storage
        .upload(
            &legacy_key,
            "text/html",
            "<h2>New A</h2><p>a b</p><h2>New B</h2><p>c</p>"
                .as_bytes()
                .to_vec()
                .into(),
        )
        .await
        .expect("upload legacy html");
    let assets = PgDocumentAssetRepository::new(scenario.app.pool().clone());
    assets
        .upsert_document_asset(NewDocumentAsset {
            document_id: parse_doc_uuid(&document_id),
            asset_kind: ArchiveAssetKind::ReadableHtml,
            s3_key: legacy_key.clone(),
            s3_bucket: "indelible-test".to_string(),
            content_type: "text/html".to_string(),
            size_bytes: 10,
            status: ArchiveAssetStatus::Completed,
            failed_reason: None,
        })
        .await
        .expect("bump readable version");

    // First read: stale -> pending + exactly one deduped ensure job.
    let (status, body) = get_toc(&scenario, &document_id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "pending");
    assert_eq!(scenario.total_job_count_by_type(TOC_ENSURE).await, 1);

    // Second read before the worker runs: still pending, still one job row.
    let (_, body) = get_toc(&scenario, &document_id).await;
    assert_eq!(body["status"], "pending");
    assert_eq!(scenario.total_job_count_by_type(TOC_ENSURE).await, 1);

    // Real worker dispatch runs document.toc.ensure.
    assert_eq!(scenario.run_pending_jobs_of_type(TOC_ENSURE).await, 1);

    let (status, body) = get_toc(&scenario, &document_id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ready");
    let entries = body["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["title"], "New A");

    // The legacy object was re-prepared onto an immutable content-addressed key.
    let readable = assets
        .find_by_document_and_kind(parse_doc_uuid(&document_id), ArchiveAssetKind::ReadableHtml)
        .await
        .expect("query readable")
        .expect("readable row");
    assert!(readable.s3_key.starts_with("documents/prepared/"));
}

#[tokio::test]
async fn headingless_document_serves_terminal_none_without_enqueue() {
    let scenario = SaveScenario::new().await;
    let url = "https://example.com/toc/headingless";
    let document_id = reader_save_with_html(
        &scenario,
        url,
        "<article><h1>Only Title</h1><p>words here</p></article>",
    )
    .await;
    scenario.run_pending_jobs_of_type(ATTACH).await;

    let (status, body) = get_toc(&scenario, &document_id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "none");
    assert!(body["entries"].as_array().expect("entries").is_empty());

    // Terminal: repeated reads never enqueue derivation work.
    let (_, body) = get_toc(&scenario, &document_id).await;
    assert_eq!(body["status"], "none");
    assert_eq!(scenario.total_job_count_by_type(TOC_ENSURE).await, 0);
}

#[tokio::test]
async fn unreadable_document_reports_pending_without_enqueue() {
    let scenario = SaveScenario::new().await;
    let created = scenario
        .extension_quick_save("https://example.com/toc/unready")
        .await;
    let document_id = document_id_from_response(&created);

    let (status, body) = get_toc(&scenario, &document_id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "pending");
    assert_eq!(scenario.total_job_count_by_type(TOC_ENSURE).await, 0);
}

#[tokio::test]
async fn other_users_cannot_read_a_foreign_toc() {
    let scenario = SaveScenario::new().await;
    let document_id =
        reader_save_with_html(&scenario, "https://example.com/toc/private", SECTIONED_HTML).await;
    scenario.run_pending_jobs_of_type(ATTACH).await;

    let stranger = scenario.app.create_web_session().await;
    let resp = scenario
        .app
        .authed_client(&stranger)
        .get(&format!("/api/v1/documents/{document_id}/toc"))
        .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
