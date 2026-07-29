#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ind_domain::{DocumentId, UserId};
use ind_test_support::{DocumentFactory, LibraryEntryFactory, TestApp, spawn_app};

const FAILURE_REASON: &str = "renderer rejected a private or internal address";

async fn seed_entry(app: &TestApp, user_id: UserId) -> DocumentId {
    let document = DocumentFactory::new(user_id).insert(app.pool()).await;
    LibraryEntryFactory::new(user_id, document.id)
        .insert(app.pool())
        .await;
    document.id
}

/// Mirrors the worker's readable-html failure upsert: one row per
/// (document_id, asset_kind), transitioned in place on re-render.
async fn upsert_readable_html_status(
    app: &TestApp,
    document_id: DocumentId,
    status: &str,
    failed_reason: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO archive_assets \
             (id, document_id, asset_kind, s3_bucket, s3_key, content_type, size_bytes, \
              status, failed_reason, created_at) \
         VALUES (gen_random_uuid(), $1, 'readable_html', 'b', '', 'text/html', 0, $2, $3, now()) \
         ON CONFLICT (document_id, asset_kind) WHERE document_id IS NOT NULL \
         DO UPDATE SET status = $2, failed_reason = $3",
    )
    .bind(document_id.into_uuid())
    .bind(status)
    .bind(failed_reason)
    .execute(app.pool())
    .await
    .unwrap();
}

fn entry_for(body: &serde_json::Value, document_id: DocumentId) -> &serde_json::Value {
    body["data"]
        .as_array()
        .unwrap_or_else(|| panic!("library response must carry data, got {body}"))
        .iter()
        .find(|e| e["document_id"] == document_id.to_string())
        .unwrap_or_else(|| panic!("entry for {document_id} missing from {body}"))
}

/// A terminally failed ingestion must be distinguishable from a healthy row in
/// the list a client renders — the failure reason rides the entry itself.
#[tokio::test]
async fn library_list_carries_the_ingest_failure_reason() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;

    let failed_doc = seed_entry(&app, session.user.id).await;
    let healthy_doc = seed_entry(&app, session.user.id).await;
    upsert_readable_html_status(&app, failed_doc, "failed", Some(FAILURE_REASON)).await;

    let response = app.authed_client(&session).get("/api/v1/library").await;
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(
        entry_for(&body, failed_doc)["ingest_failure_reason"],
        serde_json::json!(FAILURE_REASON),
        "failed entry must carry the renderer reason"
    );
    assert_eq!(
        entry_for(&body, healthy_doc)["ingest_failure_reason"],
        serde_json::Value::Null,
        "healthy entry must be explicitly null, not failed"
    );
}

#[tokio::test]
async fn recovered_render_clears_the_failure_reason() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;

    let doc = seed_entry(&app, session.user.id).await;
    upsert_readable_html_status(&app, doc, "failed", Some(FAILURE_REASON)).await;
    // A later successful render converges onto the same row.
    upsert_readable_html_status(&app, doc, "completed", None).await;

    let response = app.authed_client(&session).get("/api/v1/library").await;
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(
        entry_for(&body, doc)["ingest_failure_reason"],
        serde_json::Value::Null,
        "a recovered document must not look failed"
    );
}

/// The failure must survive every read path a client uses, not just the list:
/// a detail fetch that reports a healthy entry contradicts the row beside it.
#[tokio::test]
async fn library_detail_carries_the_ingest_failure_reason() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let failed_doc = seed_entry(&app, session.user.id).await;
    upsert_readable_html_status(&app, failed_doc, "failed", Some(FAILURE_REASON)).await;

    let list: serde_json::Value = client.get("/api/v1/library").await.json().await.unwrap();
    let entry_id = entry_for(&list, failed_doc)["library_entry_id"]
        .as_str()
        .unwrap()
        .to_string();

    let response = client.get(&format!("/api/v1/library/{entry_id}")).await;
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        body["ingest_failure_reason"],
        serde_json::json!(FAILURE_REASON),
        "detail response must report the failure the list reports, got {body}"
    );
}
