use crate::error::AppError;
use ind_domain::{DocumentId, TtsSession, TtsSessionChunk, TtsSessionId, UserId};

#[async_trait::async_trait]
pub trait TtsSessionRepository: Send + Sync {
    async fn insert(&self, session: &TtsSession) -> Result<TtsSession, AppError>;
    async fn insert_session_chunks(
        &self,
        session_id: TtsSessionId,
        chunks: &[TtsSessionChunk],
    ) -> Result<(), AppError>;
    async fn resolve_chunk(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        session_id: TtsSessionId,
        chunk_id: &str,
    ) -> Result<Option<TtsSessionChunk>, AppError>;
}
