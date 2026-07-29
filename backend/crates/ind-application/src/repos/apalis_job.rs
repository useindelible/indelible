use std::time::Duration;

use crate::error::AppError;

#[async_trait::async_trait]
pub trait ApalisJobRepository: Send + Sync {
    async fn reschedule_locked_job(
        &self,
        task_id: &str,
        lock_by: &str,
        delay: Duration,
    ) -> Result<u64, AppError>;
}
