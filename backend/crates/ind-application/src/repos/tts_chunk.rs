use crate::error::AppError;
use chrono::{DateTime, Utc};
use ind_domain::{TtsChunk, TtsChunkRecordId, UserId};

#[async_trait::async_trait]
pub trait TtsChunkRepository: Send + Sync {
    async fn get_by_cache_key(
        &self,
        user_id: UserId,
        cache_key: &str,
    ) -> Result<Option<TtsChunk>, AppError>;

    async fn get(
        &self,
        user_id: UserId,
        id: TtsChunkRecordId,
    ) -> Result<Option<TtsChunk>, AppError>;

    async fn insert(&self, chunk: &TtsChunk) -> Result<TtsChunk, AppError>;

    async fn mark_ready(
        &self,
        user_id: UserId,
        id: TtsChunkRecordId,
        duration_seconds: Option<f64>,
        updated_at: DateTime<Utc>,
    ) -> Result<TtsChunk, AppError>;

    /// Delete a chunk row by id. Used as a cleanup step when the post-insert
    /// flow (billing, quota) fails and the row must be rolled back to keep
    /// retries idempotent. Returns `Ok(())` whether or not the row existed —
    /// callers use this as a best-effort cleanup, not a correctness gate.
    async fn delete(&self, user_id: UserId, id: TtsChunkRecordId) -> Result<(), AppError>;
}
