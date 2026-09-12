use crate::error::AppError;
use ind_domain::{
    DocumentId, EventOrigin, NewReadingEvent, ReadingPosition, UserDocumentState, UserId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AppendOutcome {
    pub accepted: usize,
    pub replayed: usize,
}

/// Per-user reader state: the current-position projection of `reading_events`.
#[async_trait::async_trait]
pub trait UserDocumentStateRepository: Send + Sync {
    async fn find(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<UserDocumentState>, AppError>;

    /// Single write from a caller with no device-side counter: appends one event for `origin`
    /// using the server sequence, then projects it.
    async fn record_progress(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        progress_percent: i32,
        position: Option<ReadingPosition>,
        origin: EventOrigin,
    ) -> Result<UserDocumentState, AppError>;

    /// Inserts the events and projects the newly inserted ones onto `user_document_state` in
    /// one transaction. Exact duplicates by id count as `replayed`; a reused id or
    /// `(origin, origin_seq)` with different content is a `DomainError::Conflict`.
    async fn append_reading_events(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        events: &[NewReadingEvent],
    ) -> Result<AppendOutcome, AppError>;

    /// Records that the user opened the document: sets `first_opened_at` once (kept on conflict)
    /// and advances `last_opened_at` to the latest open. Idempotent per open.
    async fn record_document_opened(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<(), AppError>;
}
