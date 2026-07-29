use std::collections::HashSet;
use std::sync::Arc;

use futures::StreamExt;
use futures::future::BoxFuture;
use ind_application::AppError;
use ind_application::outputs::mila::*;
use ind_application::ports::*;
use ind_application::repos::ai_preset::{AiPromptPresetRepository, UpdateAiPromptPresetInput};
use ind_application::repos::collection::CollectionRepository;
use ind_application::repos::content_vector::ContentVectorRepository;
use ind_application::repos::embedding_backfill::EmbeddingBackfillRepository;
use ind_application::repos::lifecycle_outbox::document_ai_processing_outbox;
use ind_application::repos::mila_session::MilaSessionRepository;
use ind_application::repos::outbox::JobOutboxRepository;
use ind_domain::*;

use crate::{AiProviderClient, MilaChatRequest, MilaChatService};

// -- Mila ports --

pub struct MilaOperationsService {
    service: ind_application::MilaConfigService,
    ai_client: Arc<dyn AiProviderClient>,
    collection_repo: Arc<dyn CollectionRepository>,
    content_vector_repo: Arc<dyn ContentVectorRepository>,
    ai_preset_repo: Arc<dyn AiPromptPresetRepository>,
    mila_session_repo: Arc<dyn MilaSessionRepository>,
    mila_session_service: Arc<ind_application::MilaSessionService>,
    chat_service: Arc<MilaChatService>,
    outbox_repo: Arc<dyn JobOutboxRepository>,
    embedding_backfill_repo: Arc<dyn EmbeddingBackfillRepository>,
    credential_cipher: Option<Arc<ind_auth::CredentialCipher>>,
}

pub struct MilaOperationsDeps {
    pub service: ind_application::MilaConfigService,
    pub ai_client: Arc<dyn AiProviderClient>,
    pub collection_repo: Arc<dyn CollectionRepository>,
    pub content_vector_repo: Arc<dyn ContentVectorRepository>,
    pub ai_preset_repo: Arc<dyn AiPromptPresetRepository>,
    pub mila_session_repo: Arc<dyn MilaSessionRepository>,
    pub mila_session_service: Arc<ind_application::MilaSessionService>,
    pub chat_service: Arc<MilaChatService>,
    pub outbox_repo: Arc<dyn JobOutboxRepository>,
    pub embedding_backfill_repo: Arc<dyn EmbeddingBackfillRepository>,
    pub credential_cipher: Option<Arc<ind_auth::CredentialCipher>>,
}

impl MilaOperationsService {
    pub fn new(deps: MilaOperationsDeps) -> Self {
        let MilaOperationsDeps {
            service,
            ai_client,
            collection_repo,
            content_vector_repo,
            ai_preset_repo,
            mila_session_repo,
            mila_session_service,
            chat_service,
            outbox_repo,
            embedding_backfill_repo,
            credential_cipher,
        } = deps;

        Self {
            service,
            ai_client,
            collection_repo,
            content_vector_repo,
            ai_preset_repo,
            mila_session_repo,
            mila_session_service,
            chat_service,
            outbox_repo,
            embedding_backfill_repo,
            credential_cipher,
        }
    }
}

mod chat;
mod config;
mod helpers;
mod presets;
mod sessions;
