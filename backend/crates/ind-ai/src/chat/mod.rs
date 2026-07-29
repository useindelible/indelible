mod context;
mod fusion;
mod prompting;
mod retrieval;
mod streaming;
mod turn;

use std::sync::Arc;

use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::ai_preset::AiPromptPresetRepository;
use ind_application::repos::content_vector::ContentVectorRepository;
use ind_application::repos::content_vector::{
    CollectionDocumentFtsQuery, CollectionDocumentVectorQuery, CrossDocumentFtsQuery,
    CrossDocumentVectorQuery, SingleDocumentFtsQuery, SingleDocumentVectorQuery,
};
use ind_application::repos::document::DocumentRepository;
use ind_application::repos::mila_config::MilaConfigRepository;
use ind_application::repos::mila_session::MilaSessionRepository;
use ind_application::repos::prepared_content::PreparedContentProvider;
use ind_auth::CredentialCipher;
use ind_domain::{
    AiPromptAction, AiPromptPreset, Document, DocumentId, DomainError, MilaConfig, MilaMessage,
    MilaSession, MilaSessionId, SearchHit, UserId,
};

use crate::content::map_ai_error;
use crate::{AiProviderClient, AiProviderConfig, ChatCompletionRequest};

use prompting::{validate_chat_request, validate_question_for_context};
use streaming::{ChatStreamState, wrap_stream};

pub type MilaChatStream = futures::stream::BoxStream<'static, Result<MilaChatDelta, AppError>>;

#[derive(Debug, Clone)]
pub struct MilaChatRequest {
    pub user_id: UserId,
    pub session_id: MilaSessionId,
    pub question: String,
    pub highlight_text: Option<String>,
    pub highlight_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaChatDelta {
    pub content: String,
    pub retrieval_degraded: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedChatTurn {
    pub assistant_message: String,
    pub source_chunks: Vec<Uuid>,
}

pub struct MilaChatService {
    document_repo: Arc<dyn ChatDocuments>,
    content_provider: Arc<dyn ChatContent>,
    mila_config_repo: Arc<dyn ChatConfig>,
    ai_preset_repo: Arc<dyn ChatPresets>,
    content_vector_repo: Arc<dyn ChatRetrieval>,
    mila_session_repo: Arc<dyn ChatSessions>,
    ai_client: Arc<dyn AiProviderClient>,
    credential_cipher: Option<Arc<CredentialCipher>>,
}

#[async_trait::async_trait]
trait ChatDocuments: Send + Sync {
    async fn find_by_id(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<Document>, AppError>;
}

#[async_trait::async_trait]
trait ChatContent: Send + Sync {
    async fn load_for_document(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<ind_domain::PreparedItemContent>, AppError>;
    async fn load_readable_text_for_document(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<String>, AppError>;
}

#[async_trait::async_trait]
trait ChatConfig: Send + Sync {
    async fn get_by_user(&self, user_id: UserId) -> Result<Option<MilaConfig>, AppError>;
}

#[async_trait::async_trait]
trait ChatPresets: Send + Sync {
    async fn find_default_for_action(
        &self,
        user_id: UserId,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError>;
    async fn find_system_preset_for_action(
        &self,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError>;
}

#[async_trait::async_trait]
trait ChatRetrieval: Send + Sync {
    async fn search_single_document(
        &self,
        query: &SingleDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError>;
    async fn search_cross_document(
        &self,
        query: &CrossDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError>;
    async fn search_collection_document(
        &self,
        query: &CollectionDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError>;
    async fn fts_single_document(
        &self,
        query: &SingleDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError>;
    async fn fts_cross_document(
        &self,
        query: &CrossDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError>;
    async fn fts_collection_document(
        &self,
        query: &CollectionDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError>;
}

#[async_trait::async_trait]
pub(super) trait ChatSessions: Send + Sync {
    async fn find_session_for_user(
        &self,
        session_id: MilaSessionId,
        user_id: UserId,
    ) -> Result<Option<MilaSession>, AppError>;
    async fn insert_message(
        &self,
        user_id: UserId,
        message: &MilaMessage,
    ) -> Result<MilaMessage, AppError>;
    async fn list_messages(
        &self,
        session_id: MilaSessionId,
        user_id: UserId,
    ) -> Result<Vec<MilaMessage>, AppError>;
    async fn touch_session(
        &self,
        session_id: MilaSessionId,
        user_id: UserId,
        last_active: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError>;
}

struct DocumentAdapter(Arc<dyn DocumentRepository>);
struct ContentAdapter(Arc<dyn PreparedContentProvider>);
struct ConfigAdapter(Arc<dyn MilaConfigRepository>);
struct PresetAdapter(Arc<dyn AiPromptPresetRepository>);
struct RetrievalAdapter(Arc<dyn ContentVectorRepository>);
struct SessionAdapter(Arc<dyn MilaSessionRepository>);

#[async_trait::async_trait]
impl ChatDocuments for DocumentAdapter {
    async fn find_by_id(&self, user: UserId, id: DocumentId) -> Result<Option<Document>, AppError> {
        self.0.find_by_id(user, id).await
    }
}

#[async_trait::async_trait]
impl ChatContent for ContentAdapter {
    async fn load_for_document(
        &self,
        id: DocumentId,
    ) -> Result<Option<ind_domain::PreparedItemContent>, AppError> {
        self.0.load_for_document(id).await
    }
    async fn load_readable_text_for_document(
        &self,
        id: DocumentId,
    ) -> Result<Option<String>, AppError> {
        self.0.load_readable_text_for_document(id).await
    }
}

#[async_trait::async_trait]
impl ChatConfig for ConfigAdapter {
    async fn get_by_user(&self, user: UserId) -> Result<Option<MilaConfig>, AppError> {
        self.0.get_by_user(user).await
    }
}

#[async_trait::async_trait]
impl ChatPresets for PresetAdapter {
    async fn find_default_for_action(
        &self,
        user: UserId,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError> {
        self.0.find_default_for_action(user, action).await
    }
    async fn find_system_preset_for_action(
        &self,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError> {
        self.0.find_system_preset_for_action(action).await
    }
}

#[async_trait::async_trait]
impl ChatRetrieval for RetrievalAdapter {
    async fn search_single_document(
        &self,
        q: &SingleDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.0.search_single_document(q).await
    }
    async fn search_cross_document(
        &self,
        q: &CrossDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.0.search_cross_document(q).await
    }
    async fn search_collection_document(
        &self,
        q: &CollectionDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.0.search_collection_document(q).await
    }
    async fn fts_single_document(
        &self,
        q: &SingleDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.0.fts_single_document(q).await
    }
    async fn fts_cross_document(
        &self,
        q: &CrossDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.0.fts_cross_document(q).await
    }
    async fn fts_collection_document(
        &self,
        q: &CollectionDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.0.fts_collection_document(q).await
    }
}

#[async_trait::async_trait]
impl ChatSessions for SessionAdapter {
    async fn find_session_for_user(
        &self,
        id: MilaSessionId,
        user: UserId,
    ) -> Result<Option<MilaSession>, AppError> {
        self.0.find_session_for_user(id, user).await
    }
    async fn insert_message(
        &self,
        user: UserId,
        message: &MilaMessage,
    ) -> Result<MilaMessage, AppError> {
        self.0.insert_message(user, message).await
    }
    async fn list_messages(
        &self,
        id: MilaSessionId,
        user: UserId,
    ) -> Result<Vec<MilaMessage>, AppError> {
        self.0.list_messages(id, user).await
    }
    async fn touch_session(
        &self,
        id: MilaSessionId,
        user: UserId,
        at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError> {
        self.0.touch_session(id, user, at).await
    }
}

pub(super) struct PreparedChatTurn {
    provider: AiProviderConfig,
    completion_request: ChatCompletionRequest,
    user_message: MilaMessage,
    source_chunk_ids: Vec<Uuid>,
    source_label_count: usize,
    retrieval_degraded: Option<String>,
    session_id: MilaSessionId,
    user_id: UserId,
}

impl MilaChatService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        document_repo: Arc<dyn DocumentRepository>,
        content_provider: Arc<dyn PreparedContentProvider>,
        mila_config_repo: Arc<dyn MilaConfigRepository>,
        ai_preset_repo: Arc<dyn AiPromptPresetRepository>,
        content_vector_repo: Arc<dyn ContentVectorRepository>,
        mila_session_repo: Arc<dyn MilaSessionRepository>,
        ai_client: Arc<dyn AiProviderClient>,
    ) -> Self {
        Self {
            document_repo: Arc::new(DocumentAdapter(document_repo)),
            content_provider: Arc::new(ContentAdapter(content_provider)),
            mila_config_repo: Arc::new(ConfigAdapter(mila_config_repo)),
            ai_preset_repo: Arc::new(PresetAdapter(ai_preset_repo)),
            content_vector_repo: Arc::new(RetrievalAdapter(content_vector_repo)),
            mila_session_repo: Arc::new(SessionAdapter(mila_session_repo)),
            ai_client,
            credential_cipher: None,
        }
    }

    #[cfg(test)]
    fn with_ports(
        document_repo: Arc<dyn ChatDocuments>,
        content_provider: Arc<dyn ChatContent>,
        mila_config_repo: Arc<dyn ChatConfig>,
        ai_preset_repo: Arc<dyn ChatPresets>,
        content_vector_repo: Arc<dyn ChatRetrieval>,
        mila_session_repo: Arc<dyn ChatSessions>,
        ai_client: Arc<dyn AiProviderClient>,
    ) -> Self {
        Self {
            document_repo,
            content_provider,
            mila_config_repo,
            ai_preset_repo,
            content_vector_repo,
            mila_session_repo,
            ai_client,
            credential_cipher: None,
        }
    }

    pub fn with_credential_cipher(mut self, cipher: Option<Arc<CredentialCipher>>) -> Self {
        self.credential_cipher = cipher;
        self
    }

    async fn resolve_chat_system_prompt(&self, user_id: UserId) -> Result<String, AppError> {
        if let Some(preset) = self
            .ai_preset_repo
            .find_default_for_action(user_id, AiPromptAction::Chat)
            .await?
            && !preset.system_prompt.trim().is_empty()
        {
            return Ok(preset.system_prompt);
        }
        let system_preset = self
            .ai_preset_repo
            .find_system_preset_for_action(AiPromptAction::Chat)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::InvariantViolation {
                    message: "no system preset found for chat action".into(),
                })
            })?;
        Ok(system_preset.system_prompt)
    }

    pub async fn stream_chat(&self, request: MilaChatRequest) -> Result<MilaChatStream, AppError> {
        validate_chat_request(&request)?;

        let session = self
            .mila_session_repo
            .find_session_for_user(request.session_id, request.user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "mila_session",
                    id: request.session_id.to_string(),
                })
            })?;

        let config = self
            .mila_config_repo
            .get_by_user(request.user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "mila_config",
                    id: request.user_id.to_string(),
                })
            })?;

        if !config.enabled {
            return Err(AppError::Domain(DomainError::InvalidState {
                entity: "mila_config",
                current: "disabled".into(),
                expected: "enabled".into(),
            }));
        }

        validate_question_for_context(&request.question, config.model_context_window)?;

        let history = self
            .mila_session_repo
            .list_messages(session.id, request.user_id)
            .await?;
        let prepared = self
            .prepare_turn(&session, &config, &history, &request)
            .await?;

        // Provider handshake happens before persistence: an unavailable provider must fail the
        // turn without recording the question, so a client Retry cannot duplicate it. The rarer
        // inverse loss (handshake succeeds, insert fails, one wasted generation start) is cheaper.
        let upstream = self
            .ai_client
            .chat_completion_stream(&prepared.provider, prepared.completion_request)
            .await
            .map_err(map_ai_error)?;

        self.mila_session_repo
            .insert_message(request.user_id, &prepared.user_message)
            .await?;
        self.mila_session_repo
            .touch_session(
                prepared.session_id,
                prepared.user_id,
                prepared.user_message.created_at,
            )
            .await?;

        Ok(wrap_stream(ChatStreamState {
            upstream,
            session_repo: Arc::clone(&self.mila_session_repo),
            user_id: prepared.user_id,
            session_id: prepared.session_id,
            assistant_text: String::new(),
            source_chunk_ids: prepared.source_chunk_ids,
            source_label_count: prepared.source_label_count,
            pending_warning: prepared.retrieval_degraded,
            finished: false,
        }))
    }
}

#[cfg(test)]
use fusion::reciprocal_rank_fusion;

#[cfg(test)]
mod tests;
