use super::*;
use ind_domain::{
    Document, DocumentAsset, DocumentId, EventOrigin, LibraryEntryId, NewReadingEvent,
    ReadingPosition, UserDocumentState,
};

use crate::repos::user_document_state::AppendOutcome;

/// Reader read-model for a materialized document. Distinguishes a prepared-but-unsaved
/// document (`library_entry_id` is `None`) from a saved Library entry (`Some`).
pub struct DocumentReaderView {
    pub document: Document,
    pub state: Option<UserDocumentState>,
    pub library_entry_id: Option<LibraryEntryId>,
    pub assets: Vec<DocumentAsset>,
}

pub struct CreateHighlightRequest {
    /// Caller-supplied id. `None` lets the server mint one, which is never a replay.
    pub requested_id: Option<HighlightId>,
    pub color: String,
    pub text_content: String,
    pub locator: Option<HighlightLocator>,
    pub source_locator: Option<HighlightSourceLocator>,
}

/// Outcome of a highlight create. `created` is false when a caller-supplied id replayed an
/// identical highlight, which the HTTP layer answers with 200 instead of 201.
#[derive(Debug, Clone)]
pub struct HighlightCreation {
    pub highlight: Highlight,
    pub created: bool,
}

pub struct DocumentReprocessOutput {
    pub queued: bool,
    pub job_type: String,
    pub retry_after_seconds: Option<u64>,
}

/// HTTP-facing port for the document reader and its authored capabilities. Highlights and notes
/// require the document to have a completed readable asset (canonical rendered content); progress
/// writes `user_document_state` without requiring a Library entry.
pub trait DocumentReaderOperations: Send + Sync {
    fn get_reader(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<DocumentReaderView, AppError>>;

    /// Look up a completed asset by kind. URL projection for clients happens in
    /// the HTTP layer, which owns `asset_serving_mode`; this port hands back
    /// only the asset record.
    fn get_completed_asset(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        kind: ArchiveAssetKind,
    ) -> BoxFuture<'_, Result<DocumentAsset, AppError>>;

    fn reprocess_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<DocumentReprocessOutput, AppError>>;

    fn create_highlight(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        request: CreateHighlightRequest,
    ) -> BoxFuture<'_, Result<HighlightCreation, AppError>>;

    fn list_highlights(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<Vec<HighlightWithNote>, AppError>>;

    fn get_note(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<Option<DocumentNote>, AppError>>;

    fn upsert_note(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        body: String,
    ) -> BoxFuture<'_, Result<DocumentNote, AppError>>;

    fn update_progress(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        progress_percent: i32,
        position: Option<ReadingPosition>,
        origin: EventOrigin,
    ) -> BoxFuture<'_, Result<UserDocumentState, AppError>>;

    fn append_reading_events(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        events: Vec<NewReadingEvent>,
    ) -> BoxFuture<'_, Result<AppendOutcome, AppError>>;
}
