use std::sync::Arc;

use chrono::Utc;
use ind_application::ports::IntegrationOperations;
use ind_application::repos::export_cursor::ExportCursorRepository;
use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_application::repos::integration_oauth_token::IntegrationOAuthTokenRepository;
use ind_auth::CredentialCipher;
use ind_auth::integration_oauth::{IntegrationOAuthService, RepositoryIntegrationOAuthFlowStore};
use ind_domain::{
    IntegrationConnection, IntegrationConnectionId, IntegrationOAuthProvider, IntegrationProvider,
};
use ind_integrations::IntegrationOperationsService;
use ind_persistence::repos::{
    PgAiOutputRepository, PgDocumentAssetRepository, PgDocumentRepository,
    PgExportCursorRepository, PgIntegrationConnectionRepository, PgIntegrationOAuthTokenRepository,
    PgJobOutboxRepository, PgMilaConfigRepository, PgOAuthFlowRepository,
    PgObsidianPreviewRepository,
};
use ind_test_support::factories::{DocumentFactory, LibraryEntryFactory, UserFactory};
use ind_test_support::{TEST_CIPHER_KEY_B64, TestDb};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct Harness {
    _db: TestDb,
    ops: IntegrationOperationsService,
    user_id: ind_domain::UserId,
    connection_id: IntegrationConnectionId,
    library_entry_id: ind_domain::LibraryEntryId,
    pool: sqlx::PgPool,
}

async fn harness(server: &MockServer) -> Harness {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::new().insert(&pool).await;
    let document = DocumentFactory::new(user.id).insert(&pool).await;
    let entry = LibraryEntryFactory::new(user.id, document.id)
        .insert(&pool)
        .await;
    let now = Utc::now();
    let connection = PgIntegrationConnectionRepository::new(pool.clone())
        .create(IntegrationConnection {
            id: IntegrationConnectionId::new(),
            user_id: user.id,
            provider: IntegrationProvider::Notion,
            config: json!({}),
            status: "active".into(),
            last_sync_at: None,
            last_error: None,
            created_at: now,
            updated_at: now,
            version: 0,
        })
        .await
        .unwrap();
    let cipher = Arc::new(CredentialCipher::from_base64(TEST_CIPHER_KEY_B64).unwrap());
    PgIntegrationOAuthTokenRepository::new(pool.clone())
        .upsert(
            user.id,
            IntegrationOAuthProvider::Notion,
            cipher.seal(b"notion-access-token"),
            None,
            None,
            json!({}),
        )
        .await
        .unwrap();
    let cursor_repo = Arc::new(PgExportCursorRepository::new(pool.clone()));
    cursor_repo.upsert(connection.id, entry.id).await.unwrap();
    cursor_repo
        .mark_remote_page_resolved(connection.id, entry.id, "page-old", now)
        .await
        .unwrap();

    let oauth_service = Arc::new(IntegrationOAuthService::new(
        Vec::new(),
        Arc::new(RepositoryIntegrationOAuthFlowStore::new(Arc::new(
            PgOAuthFlowRepository::new(pool.clone()),
        ))),
        b"notion-refresh-test-secret",
        "https://api.example.com".into(),
    ));
    let export_summary = Arc::new(
        ind_application::export_summary::StoredExportSummaryProvider::new(Arc::new(
            PgAiOutputRepository::new(pool.clone()),
        )),
    );
    let prepared_content = Arc::new(ind_ingest::AssetBackedPreparedContentProvider::new(
        Arc::new(PgDocumentRepository::new(pool.clone())),
        Arc::new(PgDocumentAssetRepository::new(pool.clone())),
        Arc::new(PgMilaConfigRepository::new(pool.clone())),
        None,
    ));
    let ops = IntegrationOperationsService::new(
        Arc::new(PgIntegrationConnectionRepository::new(pool.clone())),
        Arc::new(PgIntegrationOAuthTokenRepository::new(pool.clone())),
        cursor_repo,
        Arc::new(PgJobOutboxRepository::new(pool.clone())),
        export_summary,
        prepared_content,
        Arc::new(PgObsidianPreviewRepository::new(pool.clone())),
        oauth_service,
        Some(cipher),
        server.uri(),
    );

    Harness {
        _db: db,
        ops,
        user_id: user.id,
        connection_id: connection.id,
        library_entry_id: entry.id,
        pool,
    }
}

async fn remote_page_and_export_jobs(h: &Harness) -> (Option<String>, i64, Option<String>) {
    sqlx::query_as(
        "SELECT remote_page_id, (SELECT count(*) FROM job_outbox WHERE job_type = \
         'integration.notion.export_document' AND payload->>'connection_id' = $3 \
         AND payload->>'library_entry_id' = $4), \
         (SELECT payload->>'replaced_page_id' FROM job_outbox WHERE job_type = \
         'integration.notion.export_document' AND payload->>'connection_id' = $3 \
         AND payload->>'library_entry_id' = $4) \
         FROM integration_export_cursor WHERE connection_id = $1 AND library_entry_id = $2",
    )
    .bind(h.connection_id.into_uuid())
    .bind(h.library_entry_id.into_uuid())
    .bind(h.connection_id.to_string())
    .bind(h.library_entry_id.to_string())
    .fetch_one(&h.pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn refresh_archives_the_old_page_before_resetting_and_queues_replacement() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/pages/page-old"))
        .and(header("authorization", "Bearer notion-access-token"))
        .and(body_json(json!({"in_trash": true})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "page-old",
            "in_trash": true,
            "url": "https://www.notion.so/Old-page-old"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let h = harness(&server).await;

    let refreshed = h
        .ops
        .refresh_notion_export_item(h.user_id, h.connection_id, h.library_entry_id)
        .await
        .unwrap();

    assert_eq!(
        refreshed.archived_page_url.as_deref(),
        Some("https://www.notion.so/Old-page-old")
    );
    assert!(!refreshed.job_id.is_empty());
    assert_eq!(
        remote_page_and_export_jobs(&h).await,
        (None, 1, Some("page-old".into()))
    );

    h.ops
        .refresh_notion_export_item(h.user_id, h.connection_id, h.library_entry_id)
        .await
        .unwrap_err();
    assert_eq!(
        remote_page_and_export_jobs(&h).await,
        (None, 1, Some("page-old".into()))
    );
}

#[tokio::test]
async fn refresh_preserves_the_cursor_and_does_not_queue_when_archival_fails() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/pages/page-old"))
        .respond_with(ResponseTemplate::new(503).set_body_json(json!({"message": "unavailable"})))
        .expect(1)
        .mount(&server)
        .await;
    let h = harness(&server).await;

    h.ops
        .refresh_notion_export_item(h.user_id, h.connection_id, h.library_entry_id)
        .await
        .unwrap_err();

    assert_eq!(
        remote_page_and_export_jobs(&h).await,
        (Some("page-old".into()), 0, None)
    );
}
