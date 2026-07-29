use chrono::{DateTime, Utc};

use super::event::MutationSideEffects;
use crate::error::AppError;
use ind_domain::{AiRun, AiRunId};

#[async_trait::async_trait]
pub trait AiRunRepository: Send + Sync {
    async fn create(&self, run: &AiRun) -> Result<AiRun, AppError>;

    async fn mark_completed(
        &self,
        run_id: AiRunId,
        input_tokens: Option<i32>,
        output_tokens: Option<i32>,
        completed_at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn mark_failed(
        &self,
        run_id: AiRunId,
        error_message: String,
        effects: MutationSideEffects,
        completed_at: DateTime<Utc>,
    ) -> Result<(), AppError>;
}
