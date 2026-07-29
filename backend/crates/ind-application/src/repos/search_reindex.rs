use chrono::{DateTime, Utc};
use ind_domain::JobOutbox;
use uuid::Uuid;

use crate::AppError;

#[derive(Debug, Clone)]
pub struct FullSearchReindexAdmission {
    pub queued: bool,
    pub outbox: Option<JobOutbox>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchReindexCursor {
    pub created_at: DateTime<Utc>,
    pub document_id: Uuid,
}

#[async_trait::async_trait]
pub trait SearchReindexRepository: Send + Sync {
    async fn enqueue_full_reindex(
        &self,
        page_size: u32,
        target_version: Option<i32>,
        available_at: DateTime<Utc>,
    ) -> Result<FullSearchReindexAdmission, AppError>;

    async fn complete_version(&self, version: i32) -> Result<(), AppError>;

    async fn load_version_cursor(
        &self,
        version: i32,
    ) -> Result<Option<SearchReindexCursor>, AppError>;

    async fn checkpoint_version_cursor(
        &self,
        version: i32,
        cursor: SearchReindexCursor,
    ) -> Result<(), AppError>;
}
