use chrono::{DateTime, Utc};

use crate::error::AppError;
use ind_domain::{
    DocumentId, ExportCursor, HighlightId, IntegrationConnectionId, JobOutbox, LibraryEntryId,
    UserId,
};

#[async_trait::async_trait]
pub trait ExportCursorRepository: Send + Sync {
    async fn upsert(
        &self,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
    ) -> Result<ExportCursor, AppError>;

    async fn mark_attempted(
        &self,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
        at: DateTime<Utc>,
        error: Option<String>,
    ) -> Result<(), AppError>;

    async fn mark_synced(
        &self,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
        at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn list_stale(
        &self,
        connection_id: IntegrationConnectionId,
        older_than: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<ExportCursor>, AppError>;

    async fn mark_remote_page_resolved(
        &self,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
        remote_page_id: &str,
        at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn mark_highlight_chunk_synced(
        &self,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
        last_highlight_created_at: DateTime<Utc>,
        last_highlight_id: HighlightId,
        at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn reset_document_export_and_enqueue_notion(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
        document_id: DocumentId,
        replaced_page_id: Option<String>,
    ) -> Result<JobOutbox, AppError>;

    async fn record_generated_path(
        &self,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
        new_path: String,
        new_full_document_path: String,
    ) -> Result<bool, AppError>;
}
