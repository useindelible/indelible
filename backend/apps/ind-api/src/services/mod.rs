use std::sync::Arc;

mod app_config;
mod email;
mod integrations;
mod mila;
pub(crate) mod repositories;
mod storage;
mod tts;

use crate::config::ServerConfig;
use ind_application::ports::{
    CollectionOperations, EntityOperations, FeedOperations, HighlightOperations, HomeOperations,
    MilaChatPort, MilaConfigPort, MilaPromptPresetPort, MilaSessionPort, SearchOperations,
    SmartListOperations, TagOperations,
};
use ind_auth::oauth::{OAuthConfigInput, build_oauth_config};
use ind_http_api::AppState;
use ind_http_api::middleware::rate_limit::RateLimitConfig;
use ind_persistence::repos::{
    PgApiTokenRepository, PgAuthorizationCodeRepository, PgBillingRepository,
    PgEmailVerificationRepository, PgEntityRepository, PgEventRepository, PgFeedRepository,
    PgHighlightRepository, PgHomeRepository, PgIntegrationConnectionRepository,
    PgJobOutboxRepository, PgNotificationPreferencesRepository, PgOAuthFlowRepository,
    PgOAuthIdentityRepository, PgPasswordResetRepository, PgSearchRepository,
    PgUserPreferencesRepository,
};
use ind_search::{SearchEngine, SearchRateLimitDefaults, SearchRateLimiter};
use secrecy::ExposeSecret;

pub struct ServiceBundle {
    pub state: AppState,
    pub rate_limit_config: RateLimitConfig,
    pub realtime_listener_handle: tokio::task::JoinHandle<()>,
}

#[derive(Default)]
pub struct ServiceOverrides {
    pub storage: Option<Arc<dyn ind_application::storage::ObjectStorage>>,
}

pub async fn build(config: &ServerConfig, pool: sqlx::PgPool) -> anyhow::Result<ServiceBundle> {
    build_with_overrides(config, pool, ServiceOverrides::default()).await
}

pub async fn build_with_overrides(
    config: &ServerConfig,
    pool: sqlx::PgPool,
    overrides: ServiceOverrides,
) -> anyhow::Result<ServiceBundle> {
    let jwt_secret = config.auth.jwt_secret.expose_secret().as_bytes().to_vec();
    let repos = repositories::Repositories::new(&pool);

    // Shared SSRF guard for user-supplied URLs (article ingest, feeds). The
    // policy is strict in hosted prod; self-host may opt into private targets.
    let url_guard: Arc<dyn ind_application::ports::OutboundUrlGuard> =
        Arc::new(ind_ingest::EgressUrlGuard::new(config.egress_policy()));

    let email_verification_repo = PgEmailVerificationRepository::new(pool.clone());
    let password_reset_repo = PgPasswordResetRepository::new(pool.clone());
    let api_token_repo = PgApiTokenRepository::new(pool.clone());

    let auth_service = Arc::new(ind_auth::AuthOperationsService(ind_auth::AuthService::new(
        repos.user.clone(),
        repos.refresh_token.clone(),
        Arc::new(email_verification_repo),
        Arc::new(password_reset_repo),
        jwt_secret.clone(),
        config.auth.allow_signups,
    )));

    let api_token_service = Arc::new(ind_auth::ApiTokenOperationsService(
        ind_auth::ApiTokenService::new(api_token_repo),
    ));

    let settings_service = Arc::new(ind_application::SettingsService::new(
        repos.user.clone(),
        Arc::new(PgUserPreferencesRepository::new(pool.clone())),
        Arc::new(PgNotificationPreferencesRepository::new(pool.clone())),
    ));

    let user_lookup = Arc::new(ind_auth::UserLookupService::new(repos.user.clone()));
    let outbox_repo = Arc::new(PgJobOutboxRepository::new(pool.clone()));
    let event_repo: Arc<dyn ind_application::repos::event::EventRepository> =
        Arc::new(PgEventRepository::new(pool.clone()));
    let oauth_flow_repo: Arc<dyn ind_application::repos::oauth_flow::OAuthFlowRepository> =
        Arc::new(PgOAuthFlowRepository::new(pool.clone()));
    let integration_connection_repo: Arc<
        dyn ind_application::repos::integration_connection::IntegrationConnectionRepository,
    > = Arc::new(PgIntegrationConnectionRepository::new(pool.clone()));

    let library_ops: Arc<dyn ind_application::ports::LibraryOperations> =
        Arc::new(ind_application::LibraryService::new(
            Arc::new(ind_persistence::repos::PgDocumentLifecycle::new(
                pool.clone(),
            )),
            Arc::new(ind_persistence::repos::PgLibraryRepository::new(
                pool.clone(),
            )),
            Arc::new(ind_persistence::repos::PgFeedDeliveryRepository::new(
                pool.clone(),
            )),
            Arc::new(ind_persistence::repos::PgFeedRepository::new(pool.clone())),
            url_guard.clone(),
        ));

    let oauth_config = build_oauth_config(OAuthConfigInput {
        google_client_id: config.oauth.google_client_id.as_deref(),
        google_client_secret: config
            .oauth
            .google_client_secret
            .as_ref()
            .map(|s| s.expose_secret()),
        apple_client_id: config.oauth.apple_client_id.as_deref(),
        apple_team_id: config.oauth.apple_team_id.as_deref(),
        apple_key_id: config.oauth.apple_key_id.as_deref(),
        apple_private_key_pem: config
            .oauth
            .apple_private_key_pem
            .as_ref()
            .map(|s| s.expose_secret()),
        oidc_enabled: config.oauth.oidc_enabled,
        oidc_issuer_url: config.oauth.oidc_issuer_url.as_deref(),
        oidc_client_id: config.oauth.oidc_client_id.as_deref(),
        oidc_client_secret: config
            .oauth
            .oidc_client_secret
            .as_ref()
            .map(|s| s.expose_secret()),
        oidc_provider_name: &config.oauth.oidc_provider_name,
        oidc_scopes: &config.oauth.oidc_scopes,
        oidc_auto_create_users: config.oauth.oidc_auto_create_users,
        base_url: &config.server.base_url,
    });

    let oauth_service = oauth_config.as_ref().map(|oc| {
        let oauth_identity_repo = PgOAuthIdentityRepository::new(pool.clone());
        let svc = ind_auth::OAuthOperationsService(ind_auth::oauth::OAuthService::new(
            oc.clone(),
            repos.user.clone(),
            Arc::new(oauth_identity_repo),
            config.auth.allow_signups,
        ));
        Arc::new(svc) as Arc<dyn ind_http_api::OAuthOperations>
    });

    // Extension PKCE auth
    let ext_code_repo = PgAuthorizationCodeRepository::new(pool.clone());
    let ext_refresh_service = Arc::new(ind_auth::RefreshTokenService::new(
        repos.refresh_token.clone(),
        jwt_secret.clone(),
    ));
    let allowed_redirect_uris = vec![
        format!("{}/extension/auth/callback", config.server.base_url),
        "com.useindelible.app:/oauth/callback".to_string(),
    ];
    let extension_auth_service = Arc::new(ind_auth::ExtensionAuthOperationsService(
        ind_auth::AuthorizationCodeService::new(
            Arc::new(ext_code_repo),
            ext_refresh_service,
            allowed_redirect_uris,
        ),
    ));

    let storage = match overrides.storage {
        Some(storage) => Some(storage),
        None => storage::build_storage_services(config).await?.storage,
    };
    let extension_save_ops =
        storage::build_extension_save_ops(&pool, storage.as_ref(), url_guard.clone());
    let library_upload_ops: Option<Arc<dyn ind_application::ports::LibraryUploadOperations>> =
        storage.as_ref().map(|storage| {
            Arc::new(ind_application::LibraryUploadService::new(
                Arc::new(ind_ingest::DocumentFileUploadProcessor),
                storage.clone(),
                Arc::new(ind_persistence::repos::PgDocumentUploadRepository::new(
                    pool.clone(),
                )),
                Arc::new(ind_persistence::repos::PgDocumentRepository::new(
                    pool.clone(),
                )),
                Arc::new(ind_persistence::repos::PgDocumentAssetRepository::new(
                    pool.clone(),
                )),
            )) as Arc<dyn ind_application::ports::LibraryUploadOperations>
        });

    let feed_ops: Option<Arc<dyn FeedOperations>> = {
        let feed_repo = Arc::new(PgFeedRepository::new(pool.clone()));
        let http_fetcher = Arc::new(ind_ingest::ReqwestHttpFetcher::with_policy(
            config.egress_policy(),
        )?);
        let feed_parser = Arc::new(ind_ingest::FeedRsFeedParser::new());
        let opml_parser = Arc::new(ind_ingest::QuickXmlOpmlParser::new());
        let feed_service = ind_application::FeedService::new(
            feed_repo,
            outbox_repo.clone(),
            http_fetcher,
            feed_parser,
            opml_parser,
        );
        Some(Arc::new(feed_service))
    };

    let feed_delivery_ops: Option<Arc<dyn ind_application::ports::FeedDeliveryOperations>> = Some(
        Arc::new(ind_application::FeedDeliveryService::new(Arc::new(
            ind_persistence::repos::PgFeedDeliveryRepository::new(pool.clone()),
        ))),
    );

    let highlight_ops: Option<Arc<dyn HighlightOperations>> = {
        let hl_repo = Arc::new(PgHighlightRepository::new(pool.clone()))
            as Arc<dyn ind_application::repos::highlight::HighlightRepository>;
        let tag_repo: Arc<dyn ind_application::repos::tag::TagRepository> = repos.tag.clone();
        Some(Arc::new(ind_application::HighlightService::new(
            hl_repo, tag_repo,
        )))
    };

    let search_ops: Option<Arc<dyn SearchOperations>> = {
        let defaults = SearchRateLimitDefaults::default();
        let repo = Arc::new(PgSearchRepository::new(pool.clone()));
        let engine = SearchEngine::new(repo.clone(), repo.clone(), defaults);
        let rate_limiter = SearchRateLimiter::new(
            Arc::new(PgBillingRepository::new(pool.clone())),
            repos.usage_counter.clone(),
            defaults,
        );
        Some(Arc::new(ind_search::SearchService::new(
            engine,
            rate_limiter,
        )))
    };

    let mila_adapter =
        mila::build_mila_ops(config, &pool, storage.clone(), outbox_repo.clone(), &repos)?;
    let mut mila_config_ops: Option<Arc<dyn MilaConfigPort>> = None;
    let mut mila_prompt_preset_ops: Option<Arc<dyn MilaPromptPresetPort>> = None;
    let mut mila_session_ops: Option<Arc<dyn MilaSessionPort>> = None;
    let mut mila_chat_ops: Option<Arc<dyn MilaChatPort>> = None;
    if let Some(mila) = mila_adapter {
        mila_config_ops = Some(mila.clone());
        mila_prompt_preset_ops = Some(mila.clone());
        mila_session_ops = Some(mila.clone());
        mila_chat_ops = Some(mila);
    }

    let entity_ops: Option<Arc<dyn EntityOperations>> = {
        let entity_repo = Arc::new(PgEntityRepository::new(pool.clone()))
            as Arc<dyn ind_application::repos::entity::EntityRepository>;
        Some(Arc::new(ind_application::EntityOperationsService::new(
            entity_repo,
            outbox_repo.clone(),
        )))
    };

    let home_ops: Option<Arc<dyn HomeOperations>> = {
        let home_repo = Arc::new(PgHomeRepository::new(pool.clone()));
        Some(Arc::new(ind_application::HomeService::new(home_repo)))
    };

    let collection_ops: Option<Arc<dyn CollectionOperations>> = Some(Arc::new(
        ind_application::CollectionService::new(repos.collection.clone()),
    ));

    let tag_ops: Option<Arc<dyn TagOperations>> = Some(Arc::new(ind_application::TagService::new(
        repos.tag.clone(),
    )));

    let smart_list_ops: Option<Arc<dyn SmartListOperations>> = {
        let sl_repo = Arc::new(ind_persistence::repos::PgSmartListRepository::new(
            pool.clone(),
        ));
        Some(Arc::new(ind_application::SmartListService::new(sl_repo)))
    };

    let email_services = email::build_email_services(config, &pool, &repos);
    let email_ingest_ops = email_services.ingest_ops;
    let email_ingest_provider = email_services.ingest_provider;
    let email_sender_ops = email_services.sender_ops;
    let email_alias_ops = email_services.alias_ops;

    let tts_ops = tts::build_tts_ops(config, &pool, storage.as_ref(), &repos);

    let integration_services = integrations::build_integration_services(
        config,
        &pool,
        storage.as_ref(),
        outbox_repo.clone(),
        integration_connection_repo.clone(),
        oauth_flow_repo.clone(),
        &repos,
    )?;
    let integration_ops = integration_services.integration_ops;
    let import_ops = integration_services.import_ops;
    let export_ops = integration_services.export_ops;
    let webhook_ops = integration_services.webhook_ops;
    let export_summary_provider = integration_services.export_summary_provider;

    let rate_limit_config = config.rate_limit_config();
    let app_config = app_config::build_app_config(config, rate_limit_config.clone())?;

    let realtime_hub = ind_http_api::realtime::RealtimeHub::new();
    let realtime_listener_handle =
        ind_http_api::realtime::spawn_pg_listener(pool.clone(), realtime_hub.clone());

    let feed_preparation_ops: Option<Arc<dyn ind_application::ports::FeedPreparationOperations>> =
        Some(Arc::new(ind_application::FeedPreparationService::new(
            Arc::new(ind_persistence::repos::PgDocumentLifecycle::new(
                pool.clone(),
            )),
            Arc::new(ind_persistence::repos::PgFeedDeliveryRepository::new(
                pool.clone(),
            )),
            Arc::new(ind_persistence::repos::PgFeedRepository::new(pool.clone())),
            app_config.feed_prefetch,
        )));

    // The reader serves content from object storage, so it is only available when storage is
    // configured (503 otherwise via require_document_reader_ops).
    let document_reader_ops: Option<Arc<dyn ind_application::ports::DocumentReaderOperations>> =
        storage.clone().map(|_| {
            Arc::new(ind_application::DocumentReaderService::new(
                Arc::new(ind_persistence::repos::PgDocumentRepository::new(
                    pool.clone(),
                )),
                Arc::new(ind_persistence::repos::PgUserDocumentStateRepository::new(
                    pool.clone(),
                )),
                Arc::new(ind_persistence::repos::PgLibraryRepository::new(
                    pool.clone(),
                )),
                Arc::new(ind_persistence::repos::PgHighlightRepository::new(
                    pool.clone(),
                )),
                Arc::new(ind_persistence::repos::PgDocumentNoteRepository::new(
                    pool.clone(),
                )),
                Arc::new(ind_persistence::repos::PgDocumentAssetRepository::new(
                    pool.clone(),
                )),
                Arc::new(ind_persistence::repos::PgDocumentReprocessRepository::new(
                    pool.clone(),
                )),
            )) as Arc<dyn ind_application::ports::DocumentReaderOperations>
        });

    let article_toc_ops: Option<Arc<dyn ind_application::ports::ArticleTocOperations>> =
        storage.clone().map(|storage| {
            Arc::new(
                ind_application::handlers::article_toc::ArticleTocReadService::new(
                    Arc::new(ind_persistence::repos::PgDocumentRepository::new(
                        pool.clone(),
                    )),
                    Arc::new(ind_persistence::repos::PgDocumentAssetRepository::new(
                        pool.clone(),
                    )),
                    storage,
                    Arc::new(ind_persistence::repos::PgJobOutboxRepository::new(
                        pool.clone(),
                    )),
                ),
            ) as Arc<dyn ind_application::ports::ArticleTocOperations>
        });

    let state = AppState {
        db: pool,
        config: app_config,
        jwt_secret,
        user_rate_limiter: ind_http_api::middleware::rate_limit::UserRateLimiter::new(
            rate_limit_config.user_api,
        ),
        trusted_proxies: ind_http_api::middleware::ip_extract::TrustedProxies::default(),
        token_validator: api_token_service.clone(),
        user_lookup,
        auth_service: auth_service.clone(),
        oauth_service,
        oauth_config,
        oauth_flow_repo: Some(oauth_flow_repo),
        account_ops: auth_service.clone(),
        onboarding_ops: auth_service,
        api_token_ops: api_token_service,
        webhook_ops,
        settings_ops: settings_service,
        library_ops: Some(library_ops),
        library_upload_ops,
        extension_auth_ops: Some(extension_auth_service),
        storage,
        extension_save_ops,
        feed_ops,
        feed_delivery_ops,
        feed_preparation_ops,
        highlight_ops,
        document_reader_ops,
        article_toc_ops,
        home_ops,
        search_ops,
        mila_config_ops,
        mila_prompt_preset_ops,
        mila_session_ops,
        mila_chat_ops,
        entity_ops,
        email_ingest_ops,
        email_ingest_provider,
        email_sender_ops,
        email_alias_ops,
        collection_ops,
        tag_ops,
        smart_list_ops,
        tts_ops,
        integration_ops,
        import_ops,
        export_ops,
        export_summary_provider: Some(export_summary_provider),
        event_repo: Some(event_repo),
        realtime_hub,
    };

    Ok(ServiceBundle {
        state,
        rate_limit_config,
        realtime_listener_handle,
    })
}
