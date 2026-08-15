#![allow(clippy::unwrap_used, clippy::expect_used)]

use ind_application::repos::document_lifecycle::{
    DocumentLifecycle, MaterializeIdentity, SaveToLibraryRequest,
};
use ind_application::repos::export_cursor::ExportCursorRepository;
use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_domain::{
    ContentSource, DocumentId, DocumentType, IntegrationProvider, NewUrlDocument, UserId, job_types,
};
use ind_persistence::repos::{
    PgDocumentLifecycle, PgExportCursorRepository, PgIntegrationConnectionRepository,
};
use ind_test_support::{TestDb, UserFactory};
use serde_json::json;

fn save_url(user_id: UserId, canonical_url: &str) -> SaveToLibraryRequest {
    SaveToLibraryRequest {
        identity: MaterializeIdentity::Url {
            document: NewUrlDocument {
                id: DocumentId::new(),
                user_id,
                document_type: DocumentType::Article,
                canonical_url: canonical_url.to_string(),
                original_url: None,
                content_hash: None,
                title: "Notion Replaced".into(),
                author: None,
                excerpt: None,
                published_at: None,
                language: None,
                domain: None,
                lead_image_url: None,
                thumbnail_url: None,
            },
            origin: None,
        },
        source: ContentSource::Manual,
        source_delivery_id: None,
        hide_deliveries: false,
        enqueue_engaged_ai: false,
        restore_policy: Default::default(),
        side_effects: None,
    }
}

#[tokio::test]
async fn refresh_clears_the_export_cursor_and_queues_a_replacement_carrying_the_old_page() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let canonical = format!("https://example.com/{}", uuid::Uuid::now_v7().simple());
    let saved = PgDocumentLifecycle::new(pool.clone())
        .save_to_library(save_url(user.id, &canonical))
        .await
        .unwrap();
    let connection = PgIntegrationConnectionRepository::new(pool.clone())
        .upsert_by_user_provider(
            user.id,
            IntegrationProvider::Notion,
            json!({"workspace_id": "workspace-1", "workspace_name": "Research"}),
            "active",
        )
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO integration_export_cursor
             (connection_id, library_entry_id, remote_page_id, last_synced_at,
              last_attempted_at, last_error, cursor_version, created_at, updated_at)
         VALUES ($1, $2, 'page_old', now(), now(), 'stale export', 1, now(), now())",
    )
    .bind(connection.id.into_uuid())
    .bind(saved.entry.id.into_uuid())
    .execute(&pool)
    .await
    .unwrap();

    let outbox = PgExportCursorRepository::new(pool.clone())
        .reset_document_export_and_enqueue_notion(
            user.id,
            connection.id,
            saved.entry.id,
            saved.document.id,
            Some("page_old".to_string()),
        )
        .await
        .unwrap();

    let cursor: (
        Option<String>,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<String>,
        i32,
    ) = sqlx::query_as(
        "SELECT remote_page_id, last_synced_at, last_error, cursor_version
         FROM integration_export_cursor
         WHERE connection_id = $1 AND library_entry_id = $2",
    )
    .bind(connection.id.into_uuid())
    .bind(saved.entry.id.into_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(cursor.0, None, "the replaced page must not linger");
    assert_eq!(cursor.1, None);
    assert_eq!(cursor.2, None);
    assert_eq!(cursor.3, 2, "cursor_version advances so exports re-run");

    let (job_type, payload, dedupe_key): (String, serde_json::Value, Option<String>) =
        sqlx::query_as("SELECT job_type, payload, dedupe_key FROM job_outbox WHERE id = $1")
            .bind(outbox.id.into_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(job_type, job_types::INTEGRATION_NOTION_EXPORT_DOCUMENT);
    assert_eq!(payload["replaced_page_id"], "page_old");
    assert_eq!(payload["library_entry_id"], saved.entry.id.to_string());
    assert_eq!(
        dedupe_key,
        Some(format!(
            "export:{}:{}",
            connection.id.into_uuid(),
            saved.entry.id.into_uuid()
        )),
        "a second refresh must collapse onto the same queued replacement"
    );
}
