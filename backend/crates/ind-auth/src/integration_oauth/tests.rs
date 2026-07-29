use std::sync::Arc;

use ind_domain::{IntegrationOAuthProvider, UserId};
use ind_persistence::repos::PgOAuthFlowRepository;
use ind_test_support::TestDb;

use super::{
    IntegrationOAuthError, IntegrationOAuthProviderAdapter, IntegrationOAuthService,
    ProviderTokens, RepositoryIntegrationOAuthFlowStore, integration_oauth_error_to_app_error,
};

struct NotionAdapter;

#[async_trait::async_trait]
impl IntegrationOAuthProviderAdapter for NotionAdapter {
    fn provider(&self) -> IntegrationOAuthProvider {
        IntegrationOAuthProvider::Notion
    }

    fn authorize_url(&self, state: &str, redirect_uri: &str) -> String {
        format!("https://notion.example/authorize?state={state}&redirect_uri={redirect_uri}")
    }

    async fn exchange_code(
        &self,
        code: &str,
        state: &str,
    ) -> Result<ProviderTokens, IntegrationOAuthError> {
        assert_eq!(code, "provider-code");
        assert!(!state.is_empty());
        Ok(ProviderTokens {
            access_token: "notion-access".into(),
            refresh_token: Some("notion-refresh".into()),
            expires_at: None,
            extra: serde_json::json!({"workspace_id": "workspace-1"}),
        })
    }
}

#[tokio::test]
async fn sealed_flow_round_trips_once_with_provider_scope_and_error_projection() {
    let db = TestDb::new().await;
    let store = Arc::new(RepositoryIntegrationOAuthFlowStore::new(Arc::new(
        PgOAuthFlowRepository::new(db.pool().clone()),
    )));
    let service = IntegrationOAuthService::new(
        vec![Arc::new(NotionAdapter)],
        store,
        b"integration-oauth-boundary-secret",
        "https://api.example.com".into(),
    );
    assert_eq!(
        service.configured_providers(),
        vec![IntegrationOAuthProvider::Notion]
    );
    assert!(service.has_provider(IntegrationOAuthProvider::Notion));
    let user_id = UserId::new();
    let started = service
        .start(
            user_id,
            IntegrationOAuthProvider::Notion,
            Some("/settings/integrations".into()),
        )
        .await
        .unwrap();
    assert!(started.authorize_url.contains(&started.state));
    assert!(
        started
            .authorize_url
            .contains("/api/v1/integrations/notion/callback")
    );

    let completed = service
        .complete(
            IntegrationOAuthProvider::Notion,
            "provider-code",
            &started.state,
        )
        .await
        .unwrap();
    assert_eq!(completed.user_id, user_id);
    assert_eq!(completed.provider, IntegrationOAuthProvider::Notion);
    assert_eq!(completed.tokens.access_token, "notion-access");
    assert_eq!(
        completed.redirect_after.as_deref(),
        Some("/settings/integrations")
    );
    assert!(matches!(
        service
            .complete(
                IntegrationOAuthProvider::Notion,
                "provider-code",
                &started.state,
            )
            .await,
        Err(IntegrationOAuthError::InvalidState)
    ));

    let unconfigured = IntegrationOAuthService::new(
        Vec::new(),
        Arc::new(RepositoryIntegrationOAuthFlowStore::new(Arc::new(
            PgOAuthFlowRepository::new(db.pool().clone()),
        ))),
        b"integration-oauth-boundary-secret",
        "https://api.example.com".into(),
    );
    assert!(matches!(
        unconfigured
            .start(user_id, IntegrationOAuthProvider::Notion, None)
            .await,
        Err(IntegrationOAuthError::ProviderNotConfigured(
            IntegrationOAuthProvider::Notion
        ))
    ));

    for error in [
        IntegrationOAuthError::InvalidState,
        IntegrationOAuthError::ProviderMismatch,
        IntegrationOAuthError::InvalidCredentials,
        IntegrationOAuthError::Exchange("provider failed".into()),
        IntegrationOAuthError::Configuration("bad config".into()),
    ] {
        let projected = integration_oauth_error_to_app_error(error);
        assert!(!projected.to_string().is_empty());
    }
}
