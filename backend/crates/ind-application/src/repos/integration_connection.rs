use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::error::AppError;
use ind_domain::{
    DocumentId, IntegrationConnection, IntegrationConnectionId, IntegrationProvider,
    LibraryEntryId, NotionExportItem, UserId,
};

/// A saved Library entry eligible for Notion export (TASK-236). Carries both the cursor key
/// (`library_entry_id`) and the capability key (`document_id`) the per-document export job needs.
#[derive(Debug, Clone)]
pub struct NotionExportCandidate {
    pub library_entry_id: LibraryEntryId,
    pub document_id: DocumentId,
    pub saved_at: DateTime<Utc>,
}

/// Keyset cursor for `list_notion_export_candidates`, ordered by `(saved_at, library_entry_id)`.
#[derive(Debug, Clone, Copy)]
pub struct NotionExportCursor {
    pub saved_at: DateTime<Utc>,
    pub library_entry_id: LibraryEntryId,
}

#[derive(Debug, Clone)]
pub struct NotionExportItemsPage {
    pub items: Vec<NotionExportItem>,
    pub total_count: i64,
    pub filtered_count: i64,
}

#[async_trait::async_trait]
pub trait IntegrationConnectionRepository: Send + Sync {
    async fn create(
        &self,
        connection: IntegrationConnection,
    ) -> Result<IntegrationConnection, AppError>;

    async fn upsert_by_user_provider(
        &self,
        user_id: UserId,
        provider: IntegrationProvider,
        config: serde_json::Value,
        status: &str,
    ) -> Result<IntegrationConnection, AppError>;

    async fn find_by_id(
        &self,
        user_id: UserId,
        id: IntegrationConnectionId,
    ) -> Result<Option<IntegrationConnection>, AppError>;

    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<IntegrationConnection>, AppError>;

    async fn list_active_export_capable(
        &self,
        user_id: UserId,
    ) -> Result<Vec<IntegrationConnection>, AppError>;

    async fn list_active_notion_auto_export(&self) -> Result<Vec<IntegrationConnection>, AppError>;

    async fn set_status(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        status: &str,
    ) -> Result<(), AppError>;

    async fn set_last_sync_at(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn set_last_error(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        error: Option<String>,
    ) -> Result<(), AppError>;

    async fn update_config(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        config: serde_json::Value,
    ) -> Result<(), AppError>;

    /// Optimistic-locking variant of `update_config`. Compares the
    /// caller-provided `expected_version` against the current row;
    /// returns `Ok(new_version)` on success or `Err(DomainError::Conflict)`
    /// when the version no longer matches (i.e. someone else's PATCH won
    /// the race). Settings handlers use this so two concurrent PATCHes
    /// targeting different fields can't silently overwrite each other.
    async fn update_config_with_version(
        &self,
        id: IntegrationConnectionId,
        user_id: UserId,
        expected_version: i64,
        config: serde_json::Value,
    ) -> Result<i64, AppError>;

    async fn delete(&self, id: IntegrationConnectionId, user_id: UserId) -> Result<(), AppError>;

    /// Returns counts of integration jobs in the outbox that have not yet
    /// dispatched, grouped by connection id. Backed by a single subquery
    /// against `job_outbox` filtered to integration-namespaced job types.
    /// Connections with no queued jobs are absent from the map.
    async fn count_pending_jobs_per_connection(
        &self,
        user_id: UserId,
    ) -> Result<HashMap<IntegrationConnectionId, u32>, AppError>;

    async fn list_notion_export_items(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        query: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<NotionExportItemsPage, AppError>;

    async fn find_notion_export_item(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
    ) -> Result<Option<NotionExportItem>, AppError>;

    /// Keyset-paginated saved Library entries to export to Notion (TASK-236 AC#4). When
    /// `selected_only` is true only entries with a `selected` selection row are returned; the
    /// `library_entries JOIN documents` enumeration is itself the saved-content filter.
    async fn list_notion_export_candidates(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        selected_only: bool,
        after: Option<NotionExportCursor>,
        limit: i64,
    ) -> Result<Vec<NotionExportCandidate>, AppError>;

    /// Atomically apply a batch of selection updates. The (library_entry_id, selected)
    /// pairs are written in a single transaction; if any pair fails the
    /// connection-ownership / item-ownership check the entire batch is
    /// rolled back so callers never observe a partial PATCH.
    async fn set_notion_export_item_selections_batch(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        selections: &[(LibraryEntryId, bool)],
    ) -> Result<(), AppError>;

    async fn acquire_notion_managed_target_lock(
        &self,
        connection_id: IntegrationConnectionId,
    ) -> Result<Box<dyn IntegrationConnectionLock>, AppError>;
}

pub trait IntegrationConnectionLock: Send {}
