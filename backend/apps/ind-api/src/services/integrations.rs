use std::sync::Arc;

use crate::config::ServerConfig;
use crate::services::repositories::Repositories;
use ind_application::export_summary::{ExportSummaryProvider, StoredExportSummaryProvider};
use ind_application::ports::{ExportOperations, ImportOperations, WebhookOperations};
use ind_application::repos::export_cursor::ExportCursorRepository;
use ind_application::repos::import_job::ImportJobRepository;
use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_application::repos::integration_oauth_token::IntegrationOAuthTokenRepository;
use ind_application::repos::mila_config::{DefaultingMilaConfigRepository, MilaConfigRepository};
use ind_application::repos::oauth_flow::OAuthFlowRepository;
use ind_application::repos::prepared_content::PreparedContentProvider;
use ind_application::storage::ObjectStorage;
use ind_http_api::IntegrationOperations;
use ind_ingest::AssetBackedPreparedContentProvider;
use ind_persistence::repos::{
    PgDocumentAssetRepository, PgDocumentRepository, PgImportJobRepository,
    PgIntegrationOAuthTokenRepository, PgJobOutboxRepository, PgObsidianExportRepository,
    PgObsidianPreviewRepository, PgWebhookRepository,
};
use secrecy::ExposeSecret;

pub(super) struct IntegrationServices {
    pub integration_ops: Option<Arc<dyn IntegrationOperations>>,
    pub import_ops: Option<Arc<dyn ImportOperations>>,
    pub export_ops: Option<Arc<dyn ExportOperations>>,
    pub webhook_ops: Option<Arc<dyn WebhookOperations>>,
    pub export_summary_provider: Arc<dyn ExportSummaryProvider>,
}

pub(super) fn build_integration_services(
    config: &ServerConfig,
    pool: &sqlx::PgPool,
    storage: Option<&Arc<dyn ObjectStorage>>,
    outbox_repo: Arc<PgJobOutboxRepository>,
    integration_connection_repo: Arc<dyn IntegrationConnectionRepository>,
    oauth_flow_repo: Arc<dyn OAuthFlowRepository>,
    repos: &Repositories,
) -> anyhow::Result<IntegrationServices> {
    let integration_oauth_token_repo: Arc<dyn IntegrationOAuthTokenRepository> =
        Arc::new(PgIntegrationOAuthTokenRepository::new(pool.clone()));
    let integration_export_cursor_repo: Arc<dyn ExportCursorRepository> =
        repos.export_cursor.clone();
    let import_job_repo: Arc<dyn ImportJobRepository> =
        Arc::new(PgImportJobRepository::new(pool.clone()));

    let credential_cipher: Option<Arc<ind_auth::CredentialCipher>> = match config
        .auth
        .credential_key
        .as_ref()
    {
        Some(key) => match ind_auth::CredentialCipher::from_base64(key.expose_secret()) {
            Ok(cipher) => Some(Arc::new(cipher)),
            Err(e) => {
                // M.9: fail closed in production rather than silently disabling
                // integration token encryption.
                if config.is_production() {
                    anyhow::bail!("auth.credential_key is set but invalid: {e}");
                }
                tracing::warn!(error = %e, "auth.credential_key is set but invalid; integration OAuth token storage disabled");
                None
            }
        },
        None => None,
    };

    let integration_flow_store: Arc<dyn ind_auth::integration_oauth::IntegrationOAuthFlowStore> =
        Arc::new(ind_auth::RepositoryIntegrationOAuthFlowStore::new(
            oauth_flow_repo,
        ));

    let mut integration_oauth_adapters: Vec<
        Arc<dyn ind_auth::integration_oauth::IntegrationOAuthProviderAdapter>,
    > = Vec::new();
    if let (Some(client_id), Some(client_secret), Some(redirect_url)) = (
        config.integrations.notion.client_id.clone(),
        config
            .integrations
            .notion
            .client_secret
            .as_ref()
            .map(|s| s.expose_secret().to_owned()),
        config.integrations.notion.redirect_url.clone(),
    ) {
        let expected_callback = format!(
            "{}/api/v1/integrations/notion/callback",
            config.server.base_url.trim_end_matches('/')
        );
        if redirect_url != expected_callback {
            tracing::warn!(
                configured_redirect_url = %redirect_url,
                expected_callback = %expected_callback,
                "Notion OAuth redirect URL differs from server base URL; using configured redirect URL"
            );
        }
        let notion_adapter = Arc::new(ind_auth::NotionOAuthAdapter::new(
            client_id,
            client_secret,
            "https://api.notion.com".into(),
            redirect_url,
        ));
        integration_oauth_adapters.push(notion_adapter);
        tracing::info!("Notion OAuth integration enabled");
    }

    let integration_oauth_service =
        Arc::new(ind_auth::integration_oauth::IntegrationOAuthService::new(
            integration_oauth_adapters,
            integration_flow_store,
            config.auth.csrf_secret.expose_secret().as_bytes(),
            config.server.base_url.clone(),
        ));

    let export_summary_provider: Arc<dyn ExportSummaryProvider> =
        Arc::new(StoredExportSummaryProvider::new(repos.ai_output.clone()));

    let prepared_content_provider: Arc<dyn PreparedContentProvider> = {
        let mila_repo: Arc<dyn MilaConfigRepository> = repos.mila_config.clone();
        let defaulting_mila_repo = Arc::new(DefaultingMilaConfigRepository::new(
            mila_repo,
            config.mila.clone(),
        )) as Arc<dyn MilaConfigRepository>;
        Arc::new(AssetBackedPreparedContentProvider::new(
            Arc::new(PgDocumentRepository::new(pool.clone())),
            Arc::new(PgDocumentAssetRepository::new(pool.clone())),
            defaulting_mila_repo,
            storage.cloned(),
        ))
    };

    if credential_cipher.is_none() {
        tracing::info!(
            "integration OAuth callbacks will fail until auth.credential_key is set; \
             Obsidian PAT minting and integration listing remain available"
        );
    }

    let integration_ops: Option<Arc<dyn IntegrationOperations>> = Some(Arc::new(
        ind_integrations::IntegrationOperationsService::new(
            integration_connection_repo.clone(),
            integration_oauth_token_repo,
            integration_export_cursor_repo,
            outbox_repo.clone(),
            export_summary_provider.clone(),
            prepared_content_provider,
            Arc::new(PgObsidianPreviewRepository::new(pool.clone())),
            integration_oauth_service,
            credential_cipher.clone(),
        ),
    )
        as Arc<dyn IntegrationOperations>);

    let import_ops: Option<Arc<dyn ImportOperations>> = storage.map(|s| {
        Arc::new(ind_application::services::import::ImportService::new(
            import_job_repo,
            s.clone(),
            outbox_repo.clone(),
        )) as Arc<dyn ImportOperations>
    });
    if import_ops.is_none() {
        tracing::info!("import operations disabled - S3 storage is required for artifact uploads");
    }

    let export_ops: Option<Arc<dyn ExportOperations>> =
        Some(Arc::new(ind_integrations::ExportOperationsService::new(
            integration_connection_repo,
            outbox_repo.clone(),
            repos.export_cursor.clone(),
            Arc::new(PgObsidianExportRepository::new(pool.clone())),
        )) as Arc<dyn ExportOperations>);

    #[expect(
        clippy::expect_used,
        reason = "webhook guarded client builds from a valid static egress policy; construction is infallible"
    )]
    let webhook_http = ind_integrations::webhook_delivery::build_webhook_http_client(
        config.webhook_egress_policy(),
    )
    .expect("webhook guarded client builds");
    let webhook_ops: Option<Arc<dyn WebhookOperations>> = Some(Arc::new(
        ind_integrations::webhook_delivery::WebhookDeliveryService::new(
            Arc::new(PgWebhookRepository::new(pool.clone())),
            credential_cipher,
            webhook_http,
        ),
    ) as Arc<dyn WebhookOperations>);

    Ok(IntegrationServices {
        integration_ops,
        import_ops,
        export_ops,
        webhook_ops,
        export_summary_provider,
    })
}
