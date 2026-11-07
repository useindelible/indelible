use ind_application::repos::document_asset::DocumentAssetRepository;
use ind_domain::{
    ArchiveAssetKind, ArchiveAssetStatus, DocumentType, NewDocumentAsset, UserId, job_types,
};
use ind_persistence::repos::PgDocumentAssetRepository;
use ind_test_support::{AuthedClient, DocumentFactory, TestApp, spawn_app};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::assert_json_response;

/// Two in-flight writers can serialize by luck; eight cannot, so the losers really do meet a
/// primary key the winner has not committed yet.
const CONCURRENT_WRITERS: usize = 8;

const ID: &str = "hlt_018f5b1e-0000-7000-8000-000000000001";
const HIGHLIGHTED_EVENT: &str = "document.highlighted";

fn body(id: Option<&str>, text: &str, source: Option<Value>) -> Value {
    let mut body = json!({
        "color": "yellow",
        "text_content": text,
        "locator": {"type": "html", "start_offset": 0, "end_offset": 8}
    });
    if let Some(id) = id {
        body["id"] = json!(id);
    }
    if let Some(source) = source {
        body["source_locator"] = source;
    }
    body
}

fn quote(prefix: &str) -> Value {
    json!({"type": "text_quote", "prefix": prefix, "suffix": null})
}

async fn readable_article(app: &TestApp, user_id: UserId) -> String {
    let document = DocumentFactory::new(user_id)
        .with_document_type(DocumentType::Article)
        .insert(app.pool())
        .await;
    PgDocumentAssetRepository::new(app.pool().clone())
        .upsert_document_asset(NewDocumentAsset {
            document_id: document.id,
            asset_kind: ArchiveAssetKind::ReadableHtml,
            s3_key: format!("tests/idempotency/{}", document.id),
            s3_bucket: "test-bucket".into(),
            content_type: "text/html".into(),
            size_bytes: 64,
            status: ArchiveAssetStatus::Completed,
            failed_reason: None,
        })
        .await
        .unwrap();
    document.id.to_string()
}

async fn create(client: &AuthedClient<'_>, doc: &str, body: &Value) -> reqwest::Response {
    client
        .post_json(&format!("/api/v1/documents/{doc}/highlights"), body)
        .await
}

async fn create_expecting(
    client: &AuthedClient<'_>,
    doc: &str,
    body: &Value,
    status: StatusCode,
) -> Value {
    assert_json_response(create(client, doc, body).await, status).await
}

async fn count(client: &AuthedClient<'_>, doc: &str) -> u64 {
    let listed = assert_json_response(
        client
            .get(&format!("/api/v1/documents/{doc}/highlights"))
            .await,
        StatusCode::OK,
    )
    .await;
    listed["count"].as_u64().expect("count is a number")
}

fn reindex_dedupe_key(doc: &str) -> String {
    format!("{}:{doc}", job_types::SEARCH_REINDEX_DOCUMENT)
}

async fn highlighted_events(app: &TestApp, highlight_id: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM domain_events \
         WHERE event_type = $1 AND payload->>'highlight_id' = $2",
    )
    .bind(HIGHLIGHTED_EVENT)
    .bind(highlight_id)
    .fetch_one(app.pool())
    .await
    .expect("count document.highlighted events")
}

/// Row count plus undispatched count, because a repeated side-effect run upserts the reindex
/// row on its dedupe key rather than inserting a second one; only `dispatched_at` moves.
async fn reindex_outbox(app: &TestApp, doc: &str) -> (i64, i64) {
    sqlx::query_as::<_, (i64, i64)>(
        "SELECT count(*), count(*) FILTER (WHERE dispatched_at IS NULL) FROM job_outbox \
         WHERE job_type = $1 AND dedupe_key = $2",
    )
    .bind(job_types::SEARCH_REINDEX_DOCUMENT)
    .bind(reindex_dedupe_key(doc))
    .fetch_one(app.pool())
    .await
    .expect("count search reindex outbox rows")
}

async fn mark_reindex_dispatched(app: &TestApp, doc: &str) {
    let updated = sqlx::query(
        "UPDATE job_outbox SET dispatched_at = now() WHERE job_type = $1 AND dedupe_key = $2",
    )
    .bind(job_types::SEARCH_REINDEX_DOCUMENT)
    .bind(reindex_dedupe_key(doc))
    .execute(app.pool())
    .await
    .expect("mark reindex dispatched")
    .rows_affected();
    assert_eq!(updated, 1, "expected exactly one reindex row to dispatch");
}

#[tokio::test]
async fn exact_replay_returns_the_existing_highlight() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let doc = readable_article(&app, session.user.id).await;
    let request = body(Some(ID), "selected", Some(quote("before ")));

    let created = create_expecting(&client, &doc, &request, StatusCode::CREATED).await;
    let replayed = create_expecting(&client, &doc, &request, StatusCode::OK).await;

    assert_eq!(created["id"], ID);
    assert_eq!(replayed["id"], ID);
    assert_eq!(replayed["created_at"], created["created_at"]);
    assert_eq!(replayed["updated_at"], created["updated_at"]);
    assert_eq!(replayed["source_locator"], created["source_locator"]);
    assert_eq!(count(&client, &doc).await, 1);
}

#[tokio::test]
async fn concurrent_identical_creates_produce_one_highlight() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let doc = readable_article(&app, session.user.id).await;
    let request = body(Some(ID), "selected", None);

    let responses =
        futures::future::join_all((0..CONCURRENT_WRITERS).map(|_| create(&client, &doc, &request)))
            .await;

    let mut created = 0;
    let mut replayed = 0;
    for response in responses {
        let status = response.status();
        let raw = response.text().await.expect("read body");
        match status {
            StatusCode::CREATED => created += 1,
            StatusCode::OK => replayed += 1,
            other => panic!("unexpected {other}: {raw}"),
        }
        let parsed: Value = serde_json::from_str(&raw).expect("response was JSON");
        assert_eq!(parsed["id"], ID);
    }

    assert_eq!((created, replayed), (1, CONCURRENT_WRITERS - 1));
    assert_eq!(count(&client, &doc).await, 1);
}

#[tokio::test]
async fn reused_id_with_different_content_or_owner_conflicts() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let stranger = app.create_web_session().await;
    let owner_client = app.authed_client(&owner);
    let stranger_client = app.authed_client(&stranger);
    let first = readable_article(&app, owner.user.id).await;
    let second = readable_article(&app, owner.user.id).await;
    let strangers = readable_article(&app, stranger.user.id).await;
    let stored = body(Some(ID), "selected", Some(quote("a")));

    create_expecting(&owner_client, &first, &stored, StatusCode::CREATED).await;
    for divergent in [
        body(Some(ID), "different", Some(quote("a"))),
        body(Some(ID), "selected", Some(quote("b"))),
        body(Some(ID), "selected", None),
    ] {
        create_expecting(&owner_client, &first, &divergent, StatusCode::CONFLICT).await;
    }
    create_expecting(&owner_client, &second, &stored, StatusCode::CONFLICT).await;
    create_expecting(&stranger_client, &strangers, &stored, StatusCode::CONFLICT).await;

    let bare_uuid = body(
        Some("018f5b1e-0000-7000-8000-000000000001"),
        "selected",
        None,
    );
    create_expecting(&owner_client, &first, &bare_uuid, StatusCode::BAD_REQUEST).await;
    let generated = body(None, "no id still works", None);
    create_expecting(&owner_client, &first, &generated, StatusCode::CREATED).await;

    assert_eq!(count(&owner_client, &first).await, 2);
    assert_eq!(count(&owner_client, &second).await, 0);
    assert_eq!(count(&stranger_client, &strangers).await, 0);
}

#[tokio::test]
async fn replay_does_not_repeat_highlight_side_effects() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let doc = readable_article(&app, session.user.id).await;
    let request = body(Some(ID), "selected", Some(quote("before ")));

    create_expecting(&client, &doc, &request, StatusCode::CREATED).await;
    assert_eq!(highlighted_events(&app, ID).await, 1);
    assert_eq!(reindex_outbox(&app, &doc).await, (1, 1));
    mark_reindex_dispatched(&app, &doc).await;

    create_expecting(&client, &doc, &request, StatusCode::OK).await;

    assert_eq!(highlighted_events(&app, ID).await, 1);
    assert_eq!(reindex_outbox(&app, &doc).await, (1, 0));
}
