use std::sync::{Arc, Mutex};

use chrono::Utc;
use ind_application::ports::IntegrationOperations;
use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_application::repos::integration_oauth_token::IntegrationOAuthTokenRepository;
use ind_auth::CredentialCipher;
use ind_auth::integration_oauth::{
    IntegrationOAuthError, IntegrationOAuthProviderAdapter, IntegrationOAuthService,
    ProviderTokens, RepositoryIntegrationOAuthFlowStore,
};
use ind_domain::{
    IntegrationConnection, IntegrationConnectionId, IntegrationOAuthProvider, IntegrationProvider,
    UserId,
};
use ind_integrations::IntegrationOperationsService;
use ind_persistence::repos::{
    PgAiOutputRepository, PgDocumentAssetRepository, PgDocumentRepository,
    PgExportCursorRepository, PgIntegrationConnectionRepository, PgIntegrationOAuthTokenRepository,
    PgJobOutboxRepository, PgMilaConfigRepository, PgOAuthFlowRepository,
    PgObsidianPreviewRepository,
};
use ind_test_support::factories::UserFactory;
use ind_test_support::{TEST_CIPHER_KEY_B64, TestDb};

struct RecordingAdapter {
    revoked: Mutex<Vec<String>>,
    result: Mutex<Option<IntegrationOAuthError>>,
}

impl RecordingAdapter {
    fn succeeding() -> Arc<Self> {
        Arc::new(Self {
            revoked: Mutex::new(Vec::new()),
            result: Mutex::new(None),
        })
    }

    fn failing() -> Arc<Self> {
        Arc::new(Self {
            revoked: Mutex::new(Vec::new()),
            result: Mutex::new(Some(IntegrationOAuthError::Exchange(
                "revocation refused".into(),
            ))),
        })
    }

    fn revoked(&self) -> Vec<String> {
        self.revoked.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl IntegrationOAuthProviderAdapter for RecordingAdapter {
    fn provider(&self) -> IntegrationOAuthProvider {
        IntegrationOAuthProvider::Notion
    }

    fn authorize_url(&self, _state: &str, _redirect_uri: &str) -> String {
        "https://notion.example/authorize".into()
    }

    async fn exchange_code(
        &self,
        _code: &str,
        _state: &str,
    ) -> Result<ProviderTokens, IntegrationOAuthError> {
        Err(IntegrationOAuthError::Exchange(
            "exchange is not exercised by disconnect".into(),
        ))
    }

    async fn revoke_token(&self, access_token: &str) -> Result<(), IntegrationOAuthError> {
        self.revoked.lock().unwrap().push(access_token.to_string());
        match self.result.lock().unwrap().take() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

struct Harness {
    ops: IntegrationOperationsService,
    adapter: Arc<RecordingAdapter>,
    user_id: UserId,
    connection_id: IntegrationConnectionId,
    pool: sqlx::PgPool,
}

async fn harness(
    db: &TestDb,
    adapter: Arc<RecordingAdapter>,
    provider: IntegrationProvider,
    seed_token: bool,
    with_cipher: bool,
) -> Harness {
    let pool = db.pool().clone();
    let user = UserFactory::new().insert(&pool).await;
    let now = Utc::now();
    let connection = PgIntegrationConnectionRepository::new(pool.clone())
        .create(IntegrationConnection {
            id: IntegrationConnectionId::new(),
            user_id: user.id,
            provider,
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

    let cipher = Arc::new(CredentialCipher::from_base64(TEST_CIPHER_KEY_B64).unwrap());
    if seed_token {
        PgIntegrationOAuthTokenRepository::new(pool.clone())
            .upsert(
                user.id,
                IntegrationOAuthProvider::Notion,
                cipher.seal(b"tok-live"),
                None,
                None,
                serde_json::json!({}),
            )
            .await
            .unwrap();
    }

    let oauth_service = Arc::new(IntegrationOAuthService::new(
        vec![adapter.clone()],
        Arc::new(RepositoryIntegrationOAuthFlowStore::new(Arc::new(
            PgOAuthFlowRepository::new(pool.clone()),
        ))),
        b"disconnect-revocation-test-secret",
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
        Arc::new(PgExportCursorRepository::new(pool.clone())),
        Arc::new(PgJobOutboxRepository::new(pool.clone())),
        export_summary,
        prepared_content,
        Arc::new(PgObsidianPreviewRepository::new(pool.clone())),
        oauth_service,
        with_cipher.then_some(cipher),
        "https://api.notion.com".into(),
    );

    Harness {
        ops,
        adapter,
        user_id: user.id,
        connection_id: connection.id,
        pool,
    }
}

async fn local_rows(h: &Harness) -> (i64, i64) {
    let connections: i64 =
        sqlx::query_scalar("SELECT count(*) FROM integration_connections WHERE id = $1")
            .bind(h.connection_id.into_uuid())
            .fetch_one(&h.pool)
            .await
            .unwrap();
    let tokens: i64 =
        sqlx::query_scalar("SELECT count(*) FROM integration_oauth_tokens WHERE user_id = $1")
            .bind(h.user_id.into_uuid())
            .fetch_one(&h.pool)
            .await
            .unwrap();
    (connections, tokens)
}

#[tokio::test]
async fn disconnect_revokes_the_upstream_grant_before_deleting_local_rows() {
    let db = TestDb::new().await;
    let h = harness(
        &db,
        RecordingAdapter::succeeding(),
        IntegrationProvider::Notion,
        true,
        true,
    )
    .await;

    h.ops
        .delete_connection(h.user_id, h.connection_id)
        .await
        .unwrap();

    assert_eq!(h.adapter.revoked(), vec!["tok-live".to_string()]);
    assert_eq!(local_rows(&h).await, (0, 0));
}

#[tokio::test]
async fn disconnect_fails_and_keeps_local_rows_when_revocation_fails() {
    let db = TestDb::new().await;
    let h = harness(
        &db,
        RecordingAdapter::failing(),
        IntegrationProvider::Notion,
        true,
        true,
    )
    .await;

    h.ops
        .delete_connection(h.user_id, h.connection_id)
        .await
        .unwrap_err();

    assert_eq!(h.adapter.revoked(), vec!["tok-live".to_string()]);
    assert_eq!(
        local_rows(&h).await,
        (1, 1),
        "a grant we could not revoke must keep its local rows for retry"
    );
}

#[tokio::test]
async fn disconnect_with_token_but_no_cipher_fails_and_keeps_local_rows() {
    let db = TestDb::new().await;
    let h = harness(
        &db,
        RecordingAdapter::succeeding(),
        IntegrationProvider::Notion,
        true,
        false,
    )
    .await;

    h.ops
        .delete_connection(h.user_id, h.connection_id)
        .await
        .unwrap_err();

    assert!(h.adapter.revoked().is_empty());
    assert_eq!(local_rows(&h).await, (1, 1));
}

#[tokio::test]
async fn disconnect_without_token_row_deletes_local_connection() {
    let db = TestDb::new().await;
    let h = harness(
        &db,
        RecordingAdapter::succeeding(),
        IntegrationProvider::Notion,
        false,
        true,
    )
    .await;

    h.ops
        .delete_connection(h.user_id, h.connection_id)
        .await
        .unwrap();

    assert!(h.adapter.revoked().is_empty());
    assert_eq!(local_rows(&h).await, (0, 0));
}

#[tokio::test]
async fn disconnect_of_non_oauth_provider_is_unchanged() {
    let db = TestDb::new().await;
    let h = harness(
        &db,
        RecordingAdapter::succeeding(),
        IntegrationProvider::Obsidian,
        false,
        true,
    )
    .await;

    h.ops
        .delete_connection(h.user_id, h.connection_id)
        .await
        .unwrap();

    assert!(h.adapter.revoked().is_empty());
    assert_eq!(local_rows(&h).await, (0, 0));
}
