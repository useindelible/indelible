use crate::error::AppError;
use ind_domain::{TtsAudioAsset, TtsChunkRecordId, UserId};

#[async_trait::async_trait]
pub trait TtsAudioAssetRepository: Send + Sync {
    async fn insert(&self, asset: &TtsAudioAsset) -> Result<TtsAudioAsset, AppError>;
    async fn get_by_chunk_record(
        &self,
        user_id: UserId,
        chunk_record_id: TtsChunkRecordId,
    ) -> Result<Option<TtsAudioAsset>, AppError>;

    async fn delete_by_chunk_record(
        &self,
        user_id: UserId,
        chunk_record_id: TtsChunkRecordId,
    ) -> Result<(), AppError>;

    async fn filter_existing_s3_keys(&self, keys: &[String]) -> Result<Vec<String>, AppError> {
        let _ = keys;
        Err(AppError::Repository(
            "tts audio asset key filtering is not supported by this repository".into(),
        ))
    }
}
