use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ind_domain::{DeadLetterJob, JobOutbox};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadLetterStats {
    pub unresolved: i64,
    pub replayed: i64,
    pub distinct_unresolved_job_types: i64,
    pub oldest_unresolved_failed_at: Option<DateTime<Utc>>,
    pub newest_unresolved_failed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct DeadLetterReplay {
    pub dead_letter: DeadLetterJob,
    pub outbox: JobOutbox,
    pub queued: bool,
}

#[async_trait]
pub trait DeadLetterRepository: Send + Sync {
    async fn insert(&self, job: DeadLetterJob) -> Result<DeadLetterJob, AppError>;
    async fn get(&self, id: ind_domain::DeadLetterJobId) -> Result<DeadLetterJob, AppError>;
    async fn list(&self, limit: i64) -> Result<Vec<DeadLetterJob>, AppError>;
    async fn replay(
        &self,
        id: ind_domain::DeadLetterJobId,
        available_at: DateTime<Utc>,
    ) -> Result<DeadLetterReplay, AppError>;
    async fn stats(&self) -> Result<DeadLetterStats, AppError>;
}
