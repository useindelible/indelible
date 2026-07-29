use crate::error::AppError;
use ind_domain::{DocumentId, UserDocumentState, UserId};

/// Read access to per-user reader state.
/// See docs/document-feed-library-architecture.md (user_document_state).
///
/// This task exposes the concept read-only so Feed/Reader surfaces can read state.
/// Writes include progress, opened timestamps, and scroll/reader position fields.
#[async_trait::async_trait]
pub trait UserDocumentStateRepository: Send + Sync {
    async fn find(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<UserDocumentState>, AppError>;

    /// Targeted reading-progress upsert: `progress_percent`, `max_progress_percent` (GREATEST),
    /// `chapter_locator`/`chapter_offset`, and `last_read_at = now()`. No `library_entries` row
    /// is required (architecture doc, "User reads a prepared-but-unsaved document"). No whole-row
    /// read-modify-write.
    async fn record_progress(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        progress_percent: i32,
        chapter_locator: Option<String>,
        chapter_offset: Option<i32>,
    ) -> Result<UserDocumentState, AppError>;

    /// Records that the user opened the document: sets `first_opened_at` once (kept on conflict)
    /// and advances `last_opened_at` to the latest open. Idempotent per open.
    async fn record_document_opened(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<(), AppError>;
}
