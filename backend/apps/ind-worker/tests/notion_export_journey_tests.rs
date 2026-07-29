#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use chrono::Utc;
use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_application::repos::integration_oauth_token::IntegrationOAuthTokenRepository;
use ind_auth::CredentialCipher;
use ind_domain::{
    IntegrationConnection, IntegrationConnectionId, IntegrationOAuthProvider, IntegrationProvider,
    NotionExportDocumentJob, NotionSyncConnectionJob,
};
use ind_persistence::repos::{
    PgDocumentRepository, PgExportCursorRepository, PgHighlightRepository,
    PgIntegrationConnectionRepository, PgIntegrationOAuthTokenRepository, PgJobOutboxRepository,
    PgLibraryRepository, PgTagRepository,
};
use ind_test_support::{SavedDocumentFactory, TEST_CIPHER_KEY_B64, TestDb, UserFactory};
use ind_worker::context::{NotionJobDeps, NotionRateLimiterRegistry};
use ind_worker::jobs::integrations::notion::{handle_export_document, handle_sync_connection};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn notion_deps(db: &TestDb, server: &MockServer) -> NotionJobDeps {
    let pool = db.pool().clone();
    NotionJobDeps {
        connection_repo: Arc::new(PgIntegrationConnectionRepository::new(pool.clone())),
        oauth_token_repo: Arc::new(PgIntegrationOAuthTokenRepository::new(pool.clone())),
        export_cursor_repo: Arc::new(PgExportCursorRepository::new(pool.clone())),
        highlight_repo: Arc::new(PgHighlightRepository::new(pool.clone())),
        tag_repo: Arc::new(PgTagRepository::new(pool.clone())),
        document_repo: Arc::new(PgDocumentRepository::new(pool.clone())),
        library_repo: Arc::new(PgLibraryRepository::new(pool.clone())),
        outbox_repo: Arc::new(PgJobOutboxRepository::new(pool)),
        cipher: Arc::new(CredentialCipher::from_base64(TEST_CIPHER_KEY_B64).unwrap()),
        rate_limiters: Arc::new(NotionRateLimiterRegistry::new(100.0)),
        notion_api_base: server.uri(),
    }
}

async fn mount_managed_target(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/v1/search"))
        .and(header("authorization", "Bearer test-access-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{
                "id": "managed-source",
                "object": "data_source",
                "title": [{"plain_text": "Indelible"}],
                "parent": {"type": "database_id", "database_id": "managed-database"}
            }],
            "next_cursor": null,
            "has_more": false
        })))
        .expect(1)
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/data_sources/managed-source"))
        .and(header("authorization", "Bearer test-access-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "managed-source",
            "properties": {
                "Title": {"id": "title", "type": "title"},
                "Author": {"id": "author", "type": "rich_text"},
                "URL": {"id": "url", "type": "url"},
                "Canonical URL": {"id": "canonical", "type": "url"},
                "Source": {"id": "source", "type": "select"},
                "Saved At": {"id": "saved", "type": "date"},
                "Tags": {"id": "tags", "type": "multi_select"},
                "Category": {"id": "category", "type": "select"},
                "Reading Status": {"id": "status", "type": "select"},
                "Indelible ID": {"id": "indelible", "type": "rich_text"},
                "Last Synced At": {"id": "synced", "type": "date"}
            }
        })))
        .expect(1)
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/views"))
        .and(header("authorization", "Bearer test-access-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                {"name": "Articles"}, {"name": "Books"}, {"name": "Emails"},
                {"name": "PDFs"}, {"name": "Tweets"}, {"name": "Videos"},
                {"name": "Podcasts"}
            ],
            "next_cursor": null,
            "has_more": false
        })))
        .expect(1)
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/data_sources/managed-source/query"))
        .and(header("authorization", "Bearer test-access-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [], "next_cursor": null, "has_more": false
        })))
        .expect(1)
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/pages"))
        .and(header("authorization", "Bearer test-access-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "created-page"
        })))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn export_provisions_target_persists_cursor_and_syncs_saved_candidates_to_outbox() {
    let db = TestDb::new().await;
    let server = MockServer::start().await;
    mount_managed_target(&server).await;
    let user = UserFactory::new().insert(db.pool()).await;
    let first = SavedDocumentFactory::new(user.id)
        .with_title("Worker boundary article")
        .with_url("https://example.com/worker-boundary")
        .insert(db.pool())
        .await;
    let second = SavedDocumentFactory::new(user.id)
        .with_title("Candidate pagination article")
        .insert(db.pool())
        .await;
    let now = Utc::now();
    let connection = PgIntegrationConnectionRepository::new(db.pool().clone())
        .create(IntegrationConnection {
            id: IntegrationConnectionId::new(),
            user_id: user.id,
            provider: IntegrationProvider::Notion,
            config: serde_json::json!({}),
            status: "active".into(),
            last_sync_at: None,
            last_error: None,
            created_at: now,
            updated_at: now,
            version: 0,
        })
        .await
        .unwrap();
    let cipher = CredentialCipher::from_base64(TEST_CIPHER_KEY_B64).unwrap();
    PgIntegrationOAuthTokenRepository::new(db.pool().clone())
        .upsert(
            user.id,
            IntegrationOAuthProvider::Notion,
            cipher.seal(b"test-access-token"),
            None,
            None,
            serde_json::json!({"workspace_id": "workspace-1"}),
        )
        .await
        .unwrap();
    let deps = notion_deps(&db, &server);

    handle_export_document(
        &deps,
        NotionExportDocumentJob {
            connection_id: connection.id,
            user_id: user.id,
            library_entry_id: first.library_entry_id,
            document_id: first.document_id,
        },
    )
    .await
    .unwrap();

    let persisted = deps
        .connection_repo
        .find_by_id(user.id, connection.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(persisted.config["database_id"], "managed-database");
    assert_eq!(persisted.config["data_source_id"], "managed-source");
    let remote_page: Option<String> = sqlx::query_scalar(
        "SELECT remote_page_id FROM integration_export_cursor \
         WHERE connection_id = $1 AND library_entry_id = $2",
    )
    .bind(connection.id.into_uuid())
    .bind(first.library_entry_id.into_uuid())
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(remote_page.as_deref(), Some("created-page"));

    handle_sync_connection(
        &deps,
        NotionSyncConnectionJob {
            connection_id: connection.id,
            user_id: user.id,
            requested_by_user: true,
        },
    )
    .await
    .unwrap();

    let queued: Vec<(String, String)> = sqlx::query_as(
        "SELECT payload->>'library_entry_id', dedupe_key FROM job_outbox \
         WHERE job_type = 'integration.notion.export_document' ORDER BY dedupe_key",
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].0, second.library_entry_id.to_string());
    assert!(queued.iter().all(|row| row.1.starts_with("export:")));

    handle_sync_connection(
        &deps,
        NotionSyncConnectionJob {
            connection_id: connection.id,
            user_id: user.id,
            requested_by_user: true,
        },
    )
    .await
    .unwrap();
    let queued_after_repeat: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM job_outbox \
         WHERE job_type = 'integration.notion.export_document'",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(queued_after_repeat, 1);
    let last_sync_at: Option<chrono::DateTime<Utc>> =
        sqlx::query_scalar("SELECT last_sync_at FROM integration_connections WHERE id = $1")
            .bind(connection.id.into_uuid())
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert!(last_sync_at.is_some());
}
