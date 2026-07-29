use ind_application::error::AppError;
use ind_application::repos::embedding_backfill::EmbeddingBackfillRepository;
use ind_domain::{
    DocumentId, EmbedDocumentJob, JobOutboxId, MilaPlatformDefaults, UserId, job_types,
};
use sqlx::PgPool;

pub struct PgEmbeddingBackfillRepository {
    pool: PgPool,
}

impl PgEmbeddingBackfillRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn reconcile_platform_defaults(
        &self,
        defaults: &MilaPlatformDefaults,
    ) -> Result<i64, AppError> {
        let mut queued = 0_i64;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        let users = sqlx::query!(
            r#"
            SELECT DISTINCT cv.user_id AS "user_id!"
            FROM content_vectors cv
            WHERE NOT EXISTS (
                    SELECT 1
                    FROM mila_config mc
                    WHERE mc.user_id = cv.user_id
              )
              AND (
                    cv.embedding_model <> $1
                    OR cv.embedding_dim <> $2
              )
            "#,
            defaults.embedding_model,
            defaults.embedding_dim,
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        for user in users {
            let user_id = UserId::from_uuid(user.user_id);
            let docs = eligible_document_rows_tx(&mut tx, user_id).await?;
            queued += i64::try_from(docs.len()).map_err(|_| {
                AppError::Repository(Box::new(std::io::Error::other(
                    "queued document count overflow",
                )))
            })?;
            enqueue_embed_jobs_tx(&mut tx, docs).await?;
        }

        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(queued)
    }
}

async fn eligible_document_rows_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: UserId,
) -> Result<Vec<DocumentId>, AppError> {
    let rows = sqlx::query!(
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
        user_id.into_uuid(),
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(|err| AppError::Repository(Box::new(err)))?;

    Ok(rows
        .into_iter()
        .map(|row| DocumentId::from_uuid(row.id))
        .collect())
}

async fn enqueue_embed_jobs_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document_ids: Vec<DocumentId>,
) -> Result<(), AppError> {
    let now = chrono::Utc::now();
    for document_id in document_ids {
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
        .execute(&mut **tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;
    }
    Ok(())
}

fn push_repair_id(repair_ids: &mut Vec<DocumentId>, limit: usize, document_id: DocumentId) {
    if repair_ids.len() >= limit || repair_ids.contains(&document_id) {
        return;
    }
    repair_ids.push(document_id);
}

#[async_trait::async_trait]
impl EmbeddingBackfillRepository for PgEmbeddingBackfillRepository {
    async fn readable_html_document_ids_missing_vectors(
        &self,
        limit: i64,
    ) -> Result<Vec<DocumentId>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT d.id
            FROM documents d
            WHERE d.user_id IS NOT NULL
              AND EXISTS (
                    SELECT 1
                    FROM archive_assets aa
                    WHERE aa.document_id = d.id
                      AND aa.asset_kind = 'readable_html'
                      AND aa.status = 'completed'
                      AND aa.content_type = 'text/html'
                      AND aa.s3_key <> ''
              )
              AND NOT EXISTS (
                    SELECT 1
                    FROM content_vectors cv
                    WHERE cv.document_id = d.id
              )
              AND NOT EXISTS (
                    SELECT 1
                    FROM job_outbox jo
                    WHERE jo.dedupe_key = ('document.ai.embed:doc_' || d.id::text)
                      AND (
                            jo.dispatched_at IS NULL
                            OR jo.dispatched_at >= now() - interval '15 minutes'
                      )
              )
              AND NOT EXISTS (
                    SELECT 1
                    FROM job_outbox jo
                    JOIN apalis.jobs active_job
                      ON active_job.metadata->>'outbox_id' = jo.id::text
                    WHERE jo.dedupe_key = ('document.ai.embed:doc_' || d.id::text)
                      AND (
                            active_job.status IN ('Pending', 'Queued', 'Running')
                            OR (
                                active_job.status = 'Failed'
                                AND active_job.attempts < active_job.max_attempts
                            )
                      )
              )
              AND NOT EXISTS (
                    SELECT 1
                    FROM background_job_recoveries recovery
                    WHERE recovery.job_type = 'document.ai.embed'
                      AND recovery.status <> 'resolved'
                      AND recovery.payload->>'document_id' = ('doc_' || d.id::text)
              )
              AND NOT EXISTS (
                    SELECT 1
                    FROM dead_letter_jobs dead_letter
                    WHERE dead_letter.original_job_type = 'document.ai.embed'
                      AND dead_letter.replayed_at IS NULL
                      AND dead_letter.original_payload->>'document_id' = ('doc_' || d.id::text)
              )
            ORDER BY d.created_at DESC, d.id DESC
            LIMIT $1
            "#,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(rows
            .into_iter()
            .map(|row| DocumentId::from_uuid(row.id))
            .collect())
    }

    async fn epub_pdf_document_ids_missing_vectors(
        &self,
        limit: i64,
    ) -> Result<Vec<DocumentId>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT d.id
            FROM documents d
            JOIN archive_assets aa ON aa.document_id = d.id
            WHERE d.user_id IS NOT NULL
              AND aa.status = 'completed'
              AND aa.s3_key <> ''
              AND (
                  (aa.asset_kind = 'epub' AND aa.content_type = 'application/json')
                  OR (aa.asset_kind = 'original_upload' AND aa.content_type = 'application/pdf')
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM content_vectors cv
                  WHERE cv.document_id = d.id
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM job_outbox jo
                  WHERE jo.dedupe_key = ('document.ai.embed:doc_' || d.id::text)
                    AND (
                          jo.dispatched_at IS NULL
                          OR jo.dispatched_at >= now() - interval '15 minutes'
                    )
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM job_outbox jo
                  JOIN apalis.jobs active_job
                    ON active_job.metadata->>'outbox_id' = jo.id::text
                  WHERE jo.dedupe_key = ('document.ai.embed:doc_' || d.id::text)
                    AND (
                          active_job.status IN ('Pending', 'Queued', 'Running')
                          OR (
                              active_job.status = 'Failed'
                              AND active_job.attempts < active_job.max_attempts
                          )
                    )
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM background_job_recoveries recovery
                  WHERE recovery.job_type = 'document.ai.embed'
                    AND recovery.status <> 'resolved'
                    AND recovery.payload->>'document_id' = ('doc_' || d.id::text)
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM dead_letter_jobs dead_letter
                  WHERE dead_letter.original_job_type = 'document.ai.embed'
                    AND dead_letter.replayed_at IS NULL
                    AND dead_letter.original_payload->>'document_id' = ('doc_' || d.id::text)
              )
            ORDER BY d.id
            LIMIT $1
            "#,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(rows
            .into_iter()
            .map(|row| DocumentId::from_uuid(row.id))
            .collect())
    }

    async fn enqueue_missing_vector_repairs(&self, limit: i64) -> Result<i64, AppError> {
        if limit <= 0 {
            return Ok(0);
        }

        let limit = usize::try_from(limit).unwrap_or(usize::MAX);
        let mut repair_ids = Vec::new();
        for document_id in self
            .readable_html_document_ids_missing_vectors(i64::try_from(limit).unwrap_or(i64::MAX))
            .await?
        {
            push_repair_id(&mut repair_ids, limit, document_id);
            if repair_ids.len() >= limit {
                break;
            }
        }

        if repair_ids.len() < limit {
            let remaining = i64::try_from(limit - repair_ids.len()).unwrap_or(i64::MAX);
            for document_id in self
                .epub_pdf_document_ids_missing_vectors(remaining)
                .await?
            {
                push_repair_id(&mut repair_ids, limit, document_id);
                if repair_ids.len() >= limit {
                    break;
                }
            }
        }

        let queued = i64::try_from(repair_ids.len()).map_err(|_| {
            AppError::Repository(Box::new(std::io::Error::other(
                "queued document count overflow",
            )))
        })?;
        if repair_ids.is_empty() {
            return Ok(0);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        enqueue_embed_jobs_tx(&mut tx, repair_ids).await?;
        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(queued)
    }

    async fn eligible_document_ids_for_backfill(
        &self,
        user_id: UserId,
        embedding_model: &str,
        embedding_dim: i32,
    ) -> Result<Vec<DocumentId>, AppError> {
        let rows = sqlx::query!(
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
              AND NOT EXISTS (
                    SELECT 1
                    FROM content_vectors cv
                    WHERE cv.document_id = d.id
                      AND cv.embedding_model = $2
                      AND cv.embedding_dim = $3
              )
            ORDER BY le.saved_at DESC, le.id DESC
            "#,
            user_id.into_uuid(),
            embedding_model,
            embedding_dim,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(rows
            .into_iter()
            .map(|row| DocumentId::from_uuid(row.id))
            .collect())
    }

    async fn eligible_document_ids_for_full_reindex(
        &self,
        user_id: UserId,
    ) -> Result<Vec<DocumentId>, AppError> {
        let rows = sqlx::query!(
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
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(rows
            .into_iter()
            .map(|row| DocumentId::from_uuid(row.id))
            .collect())
    }

    async fn count_eligible_items(&self, user_id: UserId) -> Result<i64, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT d.id) AS "count!"
            FROM documents d
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
            "#,
            user_id.into_uuid(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(row.count)
    }

    async fn count_indexed_items(
        &self,
        user_id: UserId,
        embedding_model: &str,
        embedding_dim: i32,
    ) -> Result<i64, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT cv.document_id) AS "count!"
            FROM content_vectors cv
            WHERE cv.user_id = $1
              AND cv.embedding_model = $2
              AND cv.embedding_dim = $3
            "#,
            user_id.into_uuid(),
            embedding_model,
            embedding_dim,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(row.count)
    }

    async fn count_stale_items(
        &self,
        user_id: UserId,
        embedding_model: &str,
        embedding_dim: i32,
    ) -> Result<i64, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT cv.document_id) AS "count!"
            FROM content_vectors cv
            WHERE cv.user_id = $1
              AND (
                    cv.embedding_model <> $2
                    OR cv.embedding_dim <> $3
              )
            "#,
            user_id.into_uuid(),
            embedding_model,
            embedding_dim,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(row.count)
    }

    async fn has_pending_outbox(&self, user_id: UserId) -> Result<bool, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM job_outbox jo
                JOIN documents d ON (jo.payload ->> 'document_id') = ('doc_' || d.id::text)
                WHERE jo.job_type = 'document.ai.embed'
                  AND jo.dispatched_at IS NULL
                  AND d.user_id = $1
            ) AS "has_pending!"
            "#,
            user_id.into_uuid(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(row.has_pending)
    }
}
