use chrono::{DateTime, Utc};

use crate::error::AppError;
use crate::repos::lifecycle_outbox::OutboxEntry;
use ind_domain::{DocumentId, DocumentNote, UserId};

/// How a note write that named the version it was based on ended.
#[derive(Debug)]
pub enum NoteWrite {
    Written(DocumentNote),
    /// The stored note is newer than the base; nothing was written and this is what is stored.
    Conflict(DocumentNote),
}

#[async_trait::async_trait]
pub trait DocumentNoteRepository: Send + Sync {
    /// The single note for a document. One row per `(user_id, document_id)`.
    async fn find_by_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<DocumentNote>, AppError>;
    /// Upsert the document note and commit any `outbox` rows atomically with the note.
    /// With a `base`, the write applies only while the stored `updated_at` is not newer than it,
    /// compared at millisecond precision because clients keep milliseconds; a refused write
    /// commits nothing, the outbox rows included.
    async fn upsert_for_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        body: &str,
        base: Option<DateTime<Utc>>,
        outbox: Vec<OutboxEntry>,
    ) -> Result<NoteWrite, AppError>;
    async fn delete_for_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<(), AppError>;
}
