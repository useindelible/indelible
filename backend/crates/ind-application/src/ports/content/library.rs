use super::*;
use crate::repos::Cursor;
use crate::repos::document_lifecycle::SaveToLibraryOutcome;
use crate::repos::library::LibraryScopeCounts;
use bytes::Bytes;
use ind_domain::{
    ArchiveAssetStatus, DocumentType, FeedDeliveryId, LibraryEntry, LibraryEntryId,
    LibraryEntryWithDocument,
};

/// Input for a manual/URL/API save into the Library. The document is materialized-or-found
/// from the canonicalized URL.
pub struct SaveUrlRequest {
    pub url: String,
    pub title: Option<String>,
    pub item_type: Option<DocumentType>,
}

#[derive(Debug, Clone)]
pub struct LibraryUrlCheckResult {
    pub entry: LibraryEntry,
    pub document: ind_domain::Document,
}

/// Raw HTTP upload input after the route has read the multipart body.
pub struct UploadFileRequest {
    pub filename: String,
    pub content_type: String,
    pub data: Bytes,
    pub title_override: Option<String>,
    pub max_bytes: usize,
    pub asset_base_url: String,
}

pub struct UploadFileProcessRequest {
    pub filename: String,
    pub content_type: String,
    pub data: Bytes,
    pub title_override: Option<String>,
    pub max_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct ProcessedUploadAsset {
    /// `Some(kind)` creates/updates an `archive_assets` row. `None` uploads a companion object
    /// only, such as EPUB chapter HTML derived from the TOC asset prefix.
    pub asset_kind: Option<ArchiveAssetKind>,
    pub filename: String,
    pub content_type: String,
    pub bytes: Bytes,
    pub status: ArchiveAssetStatus,
    pub failed_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProcessedUpload {
    pub document_type: DocumentType,
    pub original_extension: &'static str,
    pub title: String,
    pub author: Option<String>,
    pub word_count: Option<i32>,
    pub reading_time_minutes: Option<i32>,
    pub assets: Vec<ProcessedUploadAsset>,
}

/// Driven port implemented by `ind-ingest`: validates, extracts metadata, generates EPUB
/// TOC/chapter assets, extracts PDF text metrics, and returns staged bytes for the application
/// service to upload.
#[async_trait::async_trait]
pub trait FileUploadProcessor: Send + Sync {
    async fn process_upload(
        &self,
        request: UploadFileProcessRequest,
    ) -> Result<ProcessedUpload, AppError>;
}

pub trait LibraryUploadOperations: Send + Sync {
    fn upload_file(
        &self,
        user_id: UserId,
        req: UploadFileRequest,
    ) -> BoxFuture<'_, Result<SaveToLibraryOutcome, AppError>>;
}

/// HTTP-facing port for the document-model Library surface. Reads always go through
/// `library_entries JOIN documents`; saves go through the atomic `save_to_library` lifecycle.
/// See docs/document-feed-library-architecture.md (library_entries; Query Surfaces -> Library).
pub trait LibraryOperations: Send + Sync {
    fn save_url(
        &self,
        user_id: UserId,
        req: SaveUrlRequest,
    ) -> BoxFuture<'_, Result<SaveToLibraryOutcome, AppError>>;

    fn save_from_delivery(
        &self,
        user_id: UserId,
        delivery_id: FeedDeliveryId,
    ) -> BoxFuture<'_, Result<SaveToLibraryOutcome, AppError>>;

    fn list(
        &self,
        user_id: UserId,
        triage: Option<TriageState>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> BoxFuture<'_, Result<Page<LibraryEntryWithDocument>, AppError>>;

    fn get(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<Option<LibraryEntryWithDocument>, AppError>>;

    fn check_url(
        &self,
        user_id: UserId,
        canonical_url: &str,
    ) -> BoxFuture<'_, Result<Option<LibraryUrlCheckResult>, AppError>>;

    fn set_triage(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
        state: TriageState,
    ) -> BoxFuture<'_, Result<LibraryEntry, AppError>>;

    fn toggle_favorite(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<LibraryEntry, AppError>>;

    fn toggle_shortlist(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<LibraryEntry, AppError>>;

    fn delete(&self, user_id: UserId, id: LibraryEntryId) -> BoxFuture<'_, Result<(), AppError>>;

    fn restore(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<LibraryEntry, AppError>>;

    fn purge(&self, user_id: UserId, id: LibraryEntryId) -> BoxFuture<'_, Result<(), AppError>>;

    fn empty_trash(&self, user_id: UserId) -> BoxFuture<'_, Result<u64, AppError>>;

    fn list_trashed(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> BoxFuture<'_, Result<Page<LibraryEntryWithDocument>, AppError>>;

    fn count(&self, user_id: UserId) -> BoxFuture<'_, Result<i64, AppError>>;

    fn count_trashed(&self, user_id: UserId) -> BoxFuture<'_, Result<i64, AppError>>;

    fn scope_counts(
        &self,
        user_id: UserId,
        triage: Option<TriageState>,
    ) -> BoxFuture<'_, Result<LibraryScopeCounts, AppError>>;
}
