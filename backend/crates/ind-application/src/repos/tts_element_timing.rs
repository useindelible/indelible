use crate::error::AppError;
use ind_domain::{TtsChunkRecordId, TtsElementTiming};

#[async_trait::async_trait]
pub trait TtsElementTimingRepository: Send + Sync {
    async fn insert_batch(&self, timings: &[TtsElementTiming]) -> Result<(), AppError>;
    async fn get_by_element(
        &self,
        chunk_record_id: TtsChunkRecordId,
        element_index: i32,
    ) -> Result<Option<TtsElementTiming>, AppError>;
}
