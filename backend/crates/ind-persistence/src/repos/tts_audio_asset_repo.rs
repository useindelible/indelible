use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::tts_audio_asset::TtsAudioAssetRepository;
use ind_domain::{TtsAudioAsset, TtsAudioAssetId, TtsChunkRecordId, UserId};

pub struct PgTtsAudioAssetRepository {
    pool: PgPool,
}

impl PgTtsAudioAssetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct TtsAudioAssetRow {
    id: Uuid,
    user_id: Uuid,
    chunk_record_id: Uuid,
    s3_key: String,
    content_type: String,
    size_bytes: i64,
    created_at: DateTime<Utc>,
}

impl From<TtsAudioAssetRow> for TtsAudioAsset {
    fn from(row: TtsAudioAssetRow) -> Self {
        Self {
            id: TtsAudioAssetId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            chunk_record_id: TtsChunkRecordId::from_uuid(row.chunk_record_id),
            s3_key: row.s3_key,
            content_type: row.content_type,
            size_bytes: row.size_bytes,
            created_at: row.created_at,
        }
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("tts_audio_asset", "asset already exists for chunk", err)
}

#[async_trait::async_trait]
impl TtsAudioAssetRepository for PgTtsAudioAssetRepository {
    async fn insert(&self, asset: &TtsAudioAsset) -> Result<TtsAudioAsset, AppError> {
        let row = sqlx::query_as!(
            TtsAudioAssetRow,
            r#"
            INSERT INTO tts_audio_assets (
                id, user_id, chunk_record_id, s3_key, content_type, size_bytes, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id, user_id, chunk_record_id, s3_key, content_type, size_bytes, created_at
            "#,
            asset.id.into_uuid(),
            asset.user_id.into_uuid(),
            asset.chunk_record_id.into_uuid(),
            asset.s3_key,
            asset.content_type,
            asset.size_bytes,
            asset.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.into())
    }

    async fn get_by_chunk_record(
        &self,
        user_id: UserId,
        chunk_record_id: TtsChunkRecordId,
    ) -> Result<Option<TtsAudioAsset>, AppError> {
        let row = sqlx::query_as!(
            TtsAudioAssetRow,
            r#"
            SELECT
                id, user_id, chunk_record_id, s3_key, content_type, size_bytes, created_at
            FROM tts_audio_assets
            WHERE chunk_record_id = $1 AND user_id = $2
            "#,
            chunk_record_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(Into::into))
    }

    async fn delete_by_chunk_record(
        &self,
        user_id: UserId,
        chunk_record_id: TtsChunkRecordId,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM tts_audio_assets
            WHERE user_id = $1 AND chunk_record_id = $2
            "#,
            user_id.into_uuid(),
            chunk_record_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(())
    }

    async fn filter_existing_s3_keys(&self, keys: &[String]) -> Result<Vec<String>, AppError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        sqlx::query_scalar!(
            r#"
            SELECT s3_key
            FROM tts_audio_assets
            WHERE s3_key = ANY($1)
            ORDER BY s3_key
            "#,
            keys,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)
    }
}
