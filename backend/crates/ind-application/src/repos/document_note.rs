use crate::error::AppError;
use crate::repos::lifecycle_outbox::OutboxEntry;
use ind_domain::{DocumentId, DocumentNote, UserId};

#[async_trait::async_trait]
pub trait DocumentNoteRepository: Send + Sync {
    /// The single note for a document. One row per `(user_id, document_id)`.
    async fn find_by_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<DocumentNote>, AppError>;
    /// Upsert the document note and commit any `outbox` rows atomically with the note.
    async fn upsert_for_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        body: &str,
        outbox: Vec<OutboxEntry>,
    ) -> Result<DocumentNote, AppError>;
    async fn delete_for_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<(), AppError>;
}
