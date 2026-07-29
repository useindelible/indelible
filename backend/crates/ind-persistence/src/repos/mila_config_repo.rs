use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::mila_config::{MilaConfigReindexOutcome, MilaConfigRepository};
use ind_domain::{DocumentId, EmbedDocumentJob, JobOutboxId, MilaConfig, UserId, job_types};

pub struct PgMilaConfigRepository {
    pool: PgPool,
}

impl PgMilaConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug)]
struct MilaConfigRow {
    user_id: Uuid,
    chat_api_base: String,
    chat_api_key_enc: Option<Vec<u8>>,
    chat_model: String,
    embedding_api_base: String,
    embedding_api_key_enc: Option<Vec<u8>>,
    embedding_model: String,
    embedding_dim: i32,
    byo_enabled: bool,
    model_context_window: i32,
    chat_context_pct: i32,
    chunk_size: i32,
    chunk_overlap: i32,
    top_k: i32,
    cross_item_top_k: i32,
    cross_item_max_per_item: i32,
    enabled: bool,
    supports_structured_output: bool,
    supports_reasoning_effort: bool,
    chat_cipher_version: i16,
    embedding_cipher_version: i16,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<MilaConfigRow> for MilaConfig {
    fn from(row: MilaConfigRow) -> Self {
        Self {
            user_id: UserId::from_uuid(row.user_id),
            chat_api_base: row.chat_api_base,
            chat_api_key_enc: row.chat_api_key_enc,
            chat_model: row.chat_model,
            embedding_api_base: row.embedding_api_base,
            embedding_api_key_enc: row.embedding_api_key_enc,
            embedding_model: row.embedding_model,
            embedding_dim: row.embedding_dim,
            byo_enabled: row.byo_enabled,
            model_context_window: row.model_context_window,
            chat_context_pct: row.chat_context_pct,
            chunk_size: row.chunk_size,
            chunk_overlap: row.chunk_overlap,
            top_k: row.top_k,
            cross_item_top_k: row.cross_item_top_k,
            cross_item_max_per_item: row.cross_item_max_per_item,
            enabled: row.enabled,
            supports_structured_output: row.supports_structured_output,
            supports_reasoning_effort: row.supports_reasoning_effort,
            chat_cipher_version: row.chat_cipher_version,
            embedding_cipher_version: row.embedding_cipher_version,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("mila_config", "mila config already exists", err)
}

#[async_trait::async_trait]
impl MilaConfigRepository for PgMilaConfigRepository {
    async fn get_by_user(&self, user_id: UserId) -> Result<Option<MilaConfig>, AppError> {
        let row = sqlx::query_as!(
            MilaConfigRow,
            r#"
            SELECT
                user_id,
                chat_api_base,
                chat_api_key_enc,
                chat_model,
                embedding_api_base,
                embedding_api_key_enc,
                embedding_model,
                embedding_dim,
                byo_enabled,
                model_context_window,
                chat_context_pct,
                chunk_size,
                chunk_overlap,
                top_k,
                cross_item_top_k,
                cross_item_max_per_item,
                enabled,
                supports_structured_output,
                supports_reasoning_effort,
                chat_cipher_version,
                embedding_cipher_version,
                created_at,
                updated_at
            FROM mila_config
            WHERE user_id = $1
            "#,
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn upsert(&self, config: &MilaConfig) -> Result<MilaConfig, AppError> {
        let row = sqlx::query_as!(
            MilaConfigRow,
            r#"
            INSERT INTO mila_config (
                user_id,
                chat_api_base,
                chat_api_key_enc,
                chat_cipher_version,
                chat_model,
                embedding_api_base,
                embedding_api_key_enc,
                embedding_cipher_version,
                embedding_model,
                embedding_dim,
                model_context_window,
                chat_context_pct,
                chunk_size,
                chunk_overlap,
                top_k,
                cross_item_top_k,
                cross_item_max_per_item,
                enabled,
                supports_structured_output,
                supports_reasoning_effort,
                created_at,
                updated_at,
                byo_enabled
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9,
                $10, $11, $12, $13, $14, $15, $16, $17,
                $18, $19, $20, $21, $22, $23
            )
            ON CONFLICT (user_id) DO UPDATE SET
                chat_api_base = EXCLUDED.chat_api_base,
                chat_api_key_enc = EXCLUDED.chat_api_key_enc,
                chat_cipher_version = EXCLUDED.chat_cipher_version,
                chat_model = EXCLUDED.chat_model,
                embedding_api_base = EXCLUDED.embedding_api_base,
                embedding_api_key_enc = EXCLUDED.embedding_api_key_enc,
                embedding_cipher_version = EXCLUDED.embedding_cipher_version,
                embedding_model = EXCLUDED.embedding_model,
                embedding_dim = EXCLUDED.embedding_dim,
                model_context_window = EXCLUDED.model_context_window,
                chat_context_pct = EXCLUDED.chat_context_pct,
                chunk_size = EXCLUDED.chunk_size,
                chunk_overlap = EXCLUDED.chunk_overlap,
                top_k = EXCLUDED.top_k,
                cross_item_top_k = EXCLUDED.cross_item_top_k,
                cross_item_max_per_item = EXCLUDED.cross_item_max_per_item,
                enabled = EXCLUDED.enabled,
                supports_structured_output = EXCLUDED.supports_structured_output,
                supports_reasoning_effort = EXCLUDED.supports_reasoning_effort,
                byo_enabled = EXCLUDED.byo_enabled,
                updated_at = EXCLUDED.updated_at
            RETURNING
                user_id,
                chat_api_base,
                chat_api_key_enc,
                chat_cipher_version,
                chat_model,
                embedding_api_base,
                embedding_api_key_enc,
                embedding_cipher_version,
                embedding_model,
                embedding_dim,
                model_context_window,
                chat_context_pct,
                chunk_size,
                chunk_overlap,
                top_k,
                cross_item_top_k,
                cross_item_max_per_item,
                enabled,
                supports_structured_output,
                supports_reasoning_effort,
                byo_enabled,
                created_at,
                updated_at
            "#,
            config.user_id.into_uuid(),
            config.chat_api_base,
            config.chat_api_key_enc.as_deref(),
            config.chat_cipher_version,
            config.chat_model,
            config.embedding_api_base,
            config.embedding_api_key_enc.as_deref(),
            config.embedding_cipher_version,
            config.embedding_model,
            config.embedding_dim,
            config.model_context_window,
            config.chat_context_pct,
            config.chunk_size,
            config.chunk_overlap,
            config.top_k,
            config.cross_item_top_k,
            config.cross_item_max_per_item,
            config.enabled,
            config.supports_structured_output,
            config.supports_reasoning_effort,
            config.created_at,
            config.updated_at,
            config.byo_enabled,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn reindex_config(
        &self,
        config: &MilaConfig,
    ) -> Result<MilaConfigReindexOutcome, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        let row = sqlx::query_as!(
            MilaConfigRow,
            r#"
            INSERT INTO mila_config (
                user_id,
                chat_api_base,
                chat_api_key_enc,
                chat_cipher_version,
                chat_model,
                embedding_api_base,
                embedding_api_key_enc,
                embedding_cipher_version,
                embedding_model,
                embedding_dim,
                model_context_window,
                chat_context_pct,
                chunk_size,
                chunk_overlap,
                top_k,
                cross_item_top_k,
                cross_item_max_per_item,
                enabled,
                supports_structured_output,
                supports_reasoning_effort,
                created_at,
                updated_at,
                byo_enabled
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9,
                $10, $11, $12, $13, $14, $15, $16, $17,
                $18, $19, $20, $21, $22, $23
            )
            ON CONFLICT (user_id) DO UPDATE SET
                chat_api_base = EXCLUDED.chat_api_base,
                chat_api_key_enc = EXCLUDED.chat_api_key_enc,
                chat_cipher_version = EXCLUDED.chat_cipher_version,
                chat_model = EXCLUDED.chat_model,
                embedding_api_base = EXCLUDED.embedding_api_base,
                embedding_api_key_enc = EXCLUDED.embedding_api_key_enc,
                embedding_cipher_version = EXCLUDED.embedding_cipher_version,
                embedding_model = EXCLUDED.embedding_model,
                embedding_dim = EXCLUDED.embedding_dim,
                model_context_window = EXCLUDED.model_context_window,
                chat_context_pct = EXCLUDED.chat_context_pct,
                chunk_size = EXCLUDED.chunk_size,
                chunk_overlap = EXCLUDED.chunk_overlap,
                top_k = EXCLUDED.top_k,
                cross_item_top_k = EXCLUDED.cross_item_top_k,
                cross_item_max_per_item = EXCLUDED.cross_item_max_per_item,
                enabled = EXCLUDED.enabled,
                supports_structured_output = EXCLUDED.supports_structured_output,
                supports_reasoning_effort = EXCLUDED.supports_reasoning_effort,
                byo_enabled = EXCLUDED.byo_enabled,
                updated_at = EXCLUDED.updated_at
            RETURNING
                user_id,
                chat_api_base,
                chat_api_key_enc,
                chat_cipher_version,
                chat_model,
                embedding_api_base,
                embedding_api_key_enc,
                embedding_cipher_version,
                embedding_model,
                embedding_dim,
                model_context_window,
                chat_context_pct,
                chunk_size,
                chunk_overlap,
                top_k,
                cross_item_top_k,
                cross_item_max_per_item,
                enabled,
                supports_structured_output,
                supports_reasoning_effort,
                byo_enabled,
                created_at,
                updated_at
            "#,
            config.user_id.into_uuid(),
            config.chat_api_base,
            config.chat_api_key_enc.as_deref(),
            config.chat_cipher_version,
            config.chat_model,
            config.embedding_api_base,
            config.embedding_api_key_enc.as_deref(),
            config.embedding_cipher_version,
            config.embedding_model,
            config.embedding_dim,
            config.model_context_window,
            config.chat_context_pct,
            config.chunk_size,
            config.chunk_overlap,
            config.top_k,
            config.cross_item_top_k,
            config.cross_item_max_per_item,
            config.enabled,
            config.supports_structured_output,
            config.supports_reasoning_effort,
            config.created_at,
            config.updated_at,
            config.byo_enabled,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        let document_rows = sqlx::query!(
            r#"
            SELECT d.id
            FROM documents d
            JOIN library_entries le
              ON le.document_id = d.id
             AND le.user_id = d.user_id
             AND le.deleted_at IS NULL
            WHERE d.user_id = $1
              AND EXISTS (
                    SELECT 1
                    FROM archive_assets aa
                    WHERE aa.document_id = d.id
                      AND aa.status = 'completed'
                      AND aa.s3_key <> ''
                      AND (
                            (aa.asset_kind = 'readable_html' AND aa.content_type = 'text/html')
                            OR (aa.asset_kind = 'epub' AND aa.content_type = 'application/json')
                            OR (aa.asset_kind = 'original_upload' AND aa.content_type = 'application/pdf')
                      )
              )
            ORDER BY le.saved_at DESC, le.id DESC
            "#,
            config.user_id.into_uuid(),
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        let now = Utc::now();
        for document_row in &document_rows {
            let document_id = DocumentId::from_uuid(document_row.id);
            let payload = serde_json::to_value(EmbedDocumentJob { document_id })
                .map_err(|err| AppError::Repository(Box::new(err)))?;
            let dedupe_key = format!("{}:{document_id}", job_types::DOCUMENT_AI_EMBED);
            let outbox_id = JobOutboxId::new();
            sqlx::query!(
                r#"
                INSERT INTO job_outbox (id, job_type, payload, dedupe_key, available_at, created_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (dedupe_key) WHERE dedupe_key IS NOT NULL DO UPDATE
                    SET payload = EXCLUDED.payload,
                        available_at = CASE
                            WHEN job_outbox.dispatched_at IS NULL
                                THEN LEAST(job_outbox.available_at, EXCLUDED.available_at)
                            ELSE EXCLUDED.available_at
                        END,
                        dispatched_at = NULL
                "#,
                outbox_id.as_uuid(),
                job_types::DOCUMENT_AI_EMBED,
                payload,
                dedupe_key,
                now,
                now,
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        }

        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(MilaConfigReindexOutcome {
            config: row.into(),
            queued_documents: i64::try_from(document_rows.len()).map_err(|_| {
                AppError::Repository(Box::new(std::io::Error::other(
                    "queued document count overflow",
                )))
            })?,
        })
    }

    async fn has_content_vectors(&self, user_id: UserId) -> Result<bool, AppError> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM content_vectors cv
                WHERE cv.user_id = $1
                LIMIT 1
            )
            "#,
            user_id.into_uuid(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(exists.unwrap_or(false))
    }
}
