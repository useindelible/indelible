use std::sync::Arc;

use crate::config::ServerConfig;
use crate::services::repositories::Repositories;
use ind_ai::{AiHttpClientConfig, MilaChatService, ReqwestAiProviderClient};
use ind_application::repos::mila_config::DefaultingMilaConfigRepository;
use ind_application::storage::ObjectStorage;
use ind_ingest::AssetBackedPreparedContentProvider;
use ind_persistence::repos::{
    PgAiPromptPresetRepository, PgContentVectorRepository, PgDocumentAssetRepository,
    PgDocumentLifecycle, PgDocumentRepository, PgEmbeddingBackfillRepository,
    PgFeedDeliveryRepository, PgFeedRepository, PgJobOutboxRepository, PgMilaSessionRepository,
};
use secrecy::ExposeSecret;

pub(super) fn build_mila_ops(
    config: &ServerConfig,
    pool: &sqlx::PgPool,
    storage: Option<Arc<dyn ObjectStorage>>,
    outbox_repo: Arc<PgJobOutboxRepository>,
    repos: &Repositories,
) -> anyhow::Result<Option<Arc<ind_ai::MilaOperationsService>>> {
    let mila_repo: Arc<dyn ind_application::repos::mila_config::MilaConfigRepository> =
        repos.mila_config.clone();
    let defaulting_mila_repo = Arc::new(DefaultingMilaConfigRepository::new(
        mila_repo.clone(),
        config.mila.clone(),
    ))
        as Arc<dyn ind_application::repos::mila_config::MilaConfigRepository>;
    let mila_service =
        ind_application::MilaConfigService::new(mila_repo.clone(), config.mila.clone());
    let ai_client = Arc::new(ReqwestAiProviderClient::new(
        AiHttpClientConfig::default(),
        config.egress_policy(),
    )?);
    let credential_cipher = config
        .auth
        .credential_key
        .as_ref()
        .map(|key| ind_auth::CredentialCipher::from_base64(key.expose_secret()).map(Arc::new))
        .transpose()?;
    let mila_document_repo = Arc::new(PgDocumentRepository::new(pool.clone()))
        as Arc<dyn ind_application::repos::document::DocumentRepository>;
    let mila_collection_repo: Arc<dyn ind_application::repos::collection::CollectionRepository> =
        repos.collection.clone();
    let mila_session_repo = Arc::new(PgMilaSessionRepository::new(pool.clone()))
        as Arc<dyn ind_application::repos::mila_session::MilaSessionRepository>;
    let mila_prompt_preset_repo = Arc::new(PgAiPromptPresetRepository::new(pool.clone()))
        as Arc<dyn ind_application::repos::ai_preset::AiPromptPresetRepository>;
    let content_vector_repo = Arc::new(PgContentVectorRepository::new(pool.clone()))
        as Arc<dyn ind_application::repos::content_vector::ContentVectorRepository>;
    let content_provider = Arc::new(AssetBackedPreparedContentProvider::new(
        mila_document_repo.clone(),
        Arc::new(PgDocumentAssetRepository::new(pool.clone())),
        defaulting_mila_repo.clone(),
        storage,
    ))
        as Arc<dyn ind_application::repos::prepared_content::PreparedContentProvider>;
    let chat_service = Arc::new(
        MilaChatService::new(
            mila_document_repo.clone(),
            content_provider,
            defaulting_mila_repo,
            mila_prompt_preset_repo.clone(),
            content_vector_repo.clone(),
            mila_session_repo.clone(),
            ai_client.clone(),
        )
        .with_credential_cipher(credential_cipher.clone()),
    );
    let embedding_backfill_repo = Arc::new(PgEmbeddingBackfillRepository::new(pool.clone()))
        as Arc<dyn ind_application::repos::embedding_backfill::EmbeddingBackfillRepository>;
    let lifecycle = Arc::new(PgDocumentLifecycle::new(pool.clone()))
        as Arc<dyn ind_application::repos::document_lifecycle::DocumentLifecycle>;
    let feed_delivery_repo = Arc::new(PgFeedDeliveryRepository::new(pool.clone()))
        as Arc<dyn ind_application::repos::feed_delivery::FeedDeliveryRepository>;
    let feed_repo = Arc::new(PgFeedRepository::new(pool.clone()))
        as Arc<dyn ind_application::repos::feed::FeedRepository>;
    let mila_session_service = Arc::new(ind_application::MilaSessionService::new(
        lifecycle,
        mila_document_repo.clone(),
        feed_delivery_repo,
        feed_repo,
    ));
    Ok(Some(Arc::new(ind_ai::MilaOperationsService::new(
        ind_ai::MilaOperationsDeps {
            service: mila_service,
            ai_client,
            collection_repo: mila_collection_repo,
            content_vector_repo,
            ai_preset_repo: mila_prompt_preset_repo,
            mila_session_repo,
            mila_session_service,
            chat_service,
            document_repo: mila_document_repo,
            outbox_repo,
            embedding_backfill_repo,
            credential_cipher,
        },
    ))))
}
