use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceTaskLease {
    pub task_name: String,
    pub continuation_cursor: Option<String>,
}

#[async_trait]
pub trait MaintenanceTaskRepository: Send + Sync {
    async fn try_acquire(
        &self,
        task_name: &str,
        lease_owner: &str,
        now: DateTime<Utc>,
        lease_expires_at: DateTime<Utc>,
    ) -> Result<Option<MaintenanceTaskLease>, AppError>;

    async fn complete(
        &self,
        task_name: &str,
        lease_owner: &str,
        next_run_at: DateTime<Utc>,
        continuation_cursor: Option<&str>,
        completed_at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn fail(
        &self,
        task_name: &str,
        lease_owner: &str,
        next_run_at: DateTime<Utc>,
        error: &str,
        failed_at: DateTime<Utc>,
    ) -> Result<(), AppError>;
}
