use chrono::{DateTime, Utc};

use crate::error::AppError;
use ind_domain::JobOutbox;

#[async_trait::async_trait]
pub trait JobOutboxRepository: Send + Sync {
    async fn enqueue(
        &self,
        job_type: &str,
        payload: serde_json::Value,
        dedupe_key: Option<String>,
        available_at: DateTime<Utc>,
    ) -> Result<JobOutbox, AppError>;
}
