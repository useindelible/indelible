use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::tts_chunk::TtsChunkRepository;
use ind_domain::{
    AudioFormat, DocumentId, TtsChunk, TtsChunkRecordId, TtsChunkStatus, TtsProvider,
    TtsVoicePersonaId, UserId,
};

pub struct PgTtsChunkRepository {
    pool: PgPool,
}

impl PgTtsChunkRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// NUMERIC(4,2) columns (pitch) are cast to float8 in SELECT and written
// through $N::numeric(4,2) in INSERT so sqlx can bind them with the `f64` type
// without requiring the bigdecimal feature flag.
struct TtsChunkRow {
    id: Uuid,
    user_id: Uuid,
    document_id: Uuid,
    chunk_id: String,
    cache_key: String,
    voice_persona_id: Option<Uuid>,
    provider: String,
    provider_model: Option<String>,
    provider_voice_id: Option<String>,
    pitch: Option<f64>,
    audio_format: String,
    sample_rate: i32,
    pronunciation_version: i32,
    chunking_version: i32,
    normalized_text_hash: String,
    start_element_index: i32,
    end_element_index: i32,
    duration_seconds: Option<f64>,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<TtsChunkRow> for TtsChunk {
    type Error = AppError;

    fn try_from(row: TtsChunkRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: TtsChunkRecordId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            document_id: DocumentId::from_uuid(row.document_id),
            chunk_id: row.chunk_id,
            cache_key: row.cache_key,
            voice_persona_id: row.voice_persona_id.map(TtsVoicePersonaId::from_uuid),
            provider: TtsProvider::parse(&row.provider).ok_or_else(|| {
                AppError::Repository(format!("unknown tts provider: {}", row.provider).into())
            })?,
            provider_model: row.provider_model,
            provider_voice_id: row.provider_voice_id,
            pitch: row.pitch.unwrap_or(1.0),
            audio_format: AudioFormat::parse(&row.audio_format).ok_or_else(|| {
                AppError::Repository(format!("unknown audio format: {}", row.audio_format).into())
            })?,
            sample_rate: row.sample_rate,
            pronunciation_version: row.pronunciation_version,
            chunking_version: row.chunking_version,
            normalized_text_hash: row.normalized_text_hash,
            start_element_index: row.start_element_index,
            end_element_index: row.end_element_index,
            duration_seconds: row.duration_seconds,
            status: TtsChunkStatus::parse(&row.status).ok_or_else(|| {
                AppError::Repository(format!("unknown chunk status: {}", row.status).into())
            })?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("tts_chunk", "chunk cache_key already exists", err)
}

#[async_trait::async_trait]
impl TtsChunkRepository for PgTtsChunkRepository {
    async fn get_by_cache_key(
        &self,
        user_id: UserId,
        cache_key: &str,
    ) -> Result<Option<TtsChunk>, AppError> {
        let row = sqlx::query_as!(
            TtsChunkRow,
            r#"
            SELECT
                id,
                user_id,
                document_id,
                chunk_id,
                cache_key,
                voice_persona_id,
                provider,
                provider_model,
                provider_voice_id,
                pitch::float8 AS pitch,
                audio_format,
                sample_rate,
                pronunciation_version,
                chunking_version,
                normalized_text_hash,
                start_element_index,
                end_element_index,
                duration_seconds,
                status,
                created_at,
                updated_at
            FROM tts_chunks
            WHERE user_id = $1 AND cache_key = $2
            "#,
            user_id.into_uuid(),
            cache_key,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(TtsChunk::try_from).transpose()
    }

    async fn get(
        &self,
        user_id: UserId,
        id: TtsChunkRecordId,
    ) -> Result<Option<TtsChunk>, AppError> {
        let row = sqlx::query_as!(
            TtsChunkRow,
            r#"
            SELECT
                id,
                user_id,
                document_id,
                chunk_id,
                cache_key,
                voice_persona_id,
                provider,
                provider_model,
                provider_voice_id,
                pitch::float8 AS pitch,
                audio_format,
                sample_rate,
                pronunciation_version,
                chunking_version,
                normalized_text_hash,
                start_element_index,
                end_element_index,
                duration_seconds,
                status,
                created_at,
                updated_at
            FROM tts_chunks
            WHERE id = $1 AND user_id = $2
            "#,
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(TtsChunk::try_from).transpose()
    }

    async fn insert(&self, chunk: &TtsChunk) -> Result<TtsChunk, AppError> {
        let row = sqlx::query_as!(
            TtsChunkRow,
            r#"
            INSERT INTO tts_chunks (
                id,
                user_id,
                document_id,
                chunk_id,
                cache_key,
                voice_persona_id,
                provider,
                provider_model,
                provider_voice_id,
                pitch,
                audio_format,
                sample_rate,
                pronunciation_version,
                chunking_version,
                normalized_text_hash,
                start_element_index,
                end_element_index,
                duration_seconds,
                status,
                created_at,
                updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9,
                $10::float8::numeric(4,2),
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21
            )
            RETURNING
                id,
                user_id,
                document_id,
                chunk_id,
                cache_key,
                voice_persona_id,
                provider,
                provider_model,
                provider_voice_id,
                pitch::float8 AS pitch,
                audio_format,
                sample_rate,
                pronunciation_version,
                chunking_version,
                normalized_text_hash,
                start_element_index,
                end_element_index,
                duration_seconds,
                status,
                created_at,
                updated_at
            "#,
            chunk.id.into_uuid(),
            chunk.user_id.into_uuid(),
            chunk.document_id.into_uuid(),
            chunk.chunk_id,
            chunk.cache_key,
            chunk.voice_persona_id.map(|id| id.into_uuid()),
            chunk.provider.as_str(),
            chunk.provider_model.as_deref(),
            chunk.provider_voice_id.as_deref(),
            chunk.pitch,
            chunk.audio_format.as_str(),
            chunk.sample_rate,
            chunk.pronunciation_version,
            chunk.chunking_version,
            chunk.normalized_text_hash,
            chunk.start_element_index,
            chunk.end_element_index,
            chunk.duration_seconds,
            chunk.status.as_str(),
            chunk.created_at,
            chunk.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        TtsChunk::try_from(row)
    }

    async fn mark_ready(
        &self,
        user_id: UserId,
        id: TtsChunkRecordId,
        duration_seconds: Option<f64>,
        updated_at: DateTime<Utc>,
    ) -> Result<TtsChunk, AppError> {
        let row = sqlx::query_as!(
            TtsChunkRow,
            r#"
            UPDATE tts_chunks
            SET
                status = 'ready',
                duration_seconds = $3,
                updated_at = $4
            WHERE id = $1 AND user_id = $2
            RETURNING
                id,
                user_id,
                document_id,
                chunk_id,
                cache_key,
                voice_persona_id,
                provider,
                provider_model,
                provider_voice_id,
                pitch::float8 AS pitch,
                audio_format,
                sample_rate,
                pronunciation_version,
                chunking_version,
                normalized_text_hash,
                start_element_index,
                end_element_index,
                duration_seconds,
                status,
                created_at,
                updated_at
            "#,
            id.into_uuid(),
            user_id.into_uuid(),
            duration_seconds,
            updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        TtsChunk::try_from(row)
    }

    async fn delete(&self, user_id: UserId, id: TtsChunkRecordId) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM tts_chunks
            WHERE id = $1 AND user_id = $2
            "#,
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(())
    }
}
