use ind_application::error::AppError;
use ind_application::repos::embedding_backfill::{
    EffectiveEmbeddingTarget, EmbeddingBackfillRepository,
};
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
) -> Result<Vec<(DocumentId, JobOutboxId)>, AppError> {
    let now = chrono::Utc::now();
    let mut queued = Vec::with_capacity(document_ids.len());
    for document_id in document_ids {
        let payload = serde_json::to_value(EmbedDocumentJob { document_id })
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        let dedupe_key = format!("{}:{document_id}", job_types::DOCUMENT_AI_EMBED);
        let outbox_id = JobOutboxId::new();
        let persisted_id = sqlx::query_scalar!(
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
            RETURNING id
            "#,
            outbox_id.as_uuid(),
            job_types::DOCUMENT_AI_EMBED,
            payload,
            dedupe_key,
            now,
            now,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;
        queued.push((document_id, JobOutboxId::from_uuid(persisted_id)));
    }
    Ok(queued)
}

async fn target_document_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: UserId,
    target: &EffectiveEmbeddingTarget,
    limit: i64,
    respect_failure_suppression: bool,
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
          AND (
                NOT $5
                OR (
                    NOT EXISTS (
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
                )
          )
        ORDER BY (
                    NOT $5
                    AND (
                        EXISTS (
                            SELECT 1
                            FROM background_job_recoveries recovery
                            WHERE recovery.job_type = 'document.ai.embed'
                              AND recovery.status <> 'resolved'
                              AND recovery.payload->>'document_id' = ('doc_' || d.id::text)
                        )
                        OR EXISTS (
                            SELECT 1
                            FROM dead_letter_jobs dead_letter
                            WHERE dead_letter.original_job_type = 'document.ai.embed'
                              AND dead_letter.replayed_at IS NULL
                              AND dead_letter.original_payload->>'document_id' = ('doc_' || d.id::text)
                        )
                    )
                 ) DESC,
                 le.saved_at DESC,
                 le.id DESC
        LIMIT $4
        "#,
        user_id.into_uuid(),
        target.embedding_model,
        target.embedding_dim,
        limit,
        respect_failure_suppression,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(|err| AppError::Repository(Box::new(err)))?;

    Ok(rows
        .into_iter()
        .map(|row| DocumentId::from_uuid(row.id))
        .collect())
}

async fn lock_unresolved_embedding_dead_letters_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document_ids: &[DocumentId],
) -> Result<(), AppError> {
    if document_ids.is_empty() {
        return Ok(());
    }
    let dedupe_keys = document_ids
        .iter()
        .map(|document_id| format!("{}:{document_id}", job_types::DOCUMENT_AI_EMBED))
        .collect::<Vec<_>>();
    sqlx::query_scalar!(
        r#"
        SELECT id
        FROM dead_letter_jobs
        WHERE original_job_type = 'document.ai.embed'
          AND original_dedupe_key = ANY($1)
          AND replayed_at IS NULL
        ORDER BY id
        FOR UPDATE
        "#,
        &dedupe_keys,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(|err| AppError::Repository(Box::new(err)))?;
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

    async fn enqueue_target_vector_repairs(
        &self,
        defaults: &MilaPlatformDefaults,
        limit: i64,
    ) -> Result<i64, AppError> {
        if limit <= 0 {
            return Ok(0);
        }

        let rows = sqlx::query!(
            r#"
            SELECT d.id
            FROM documents d
            JOIN library_entries le
              ON le.document_id = d.id
             AND le.user_id = d.user_id
             AND le.deleted_at IS NULL
            LEFT JOIN mila_config mc ON mc.user_id = d.user_id
            WHERE d.user_id IS NOT NULL
              AND COALESCE(mc.enabled, $3)
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
                      AND cv.embedding_model = CASE
                            WHEN mc.byo_enabled THEN mc.embedding_model
                            ELSE $1
                          END
                      AND cv.embedding_dim = CASE
                            WHEN mc.byo_enabled THEN mc.embedding_dim
                            ELSE $2
                          END
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
            ORDER BY le.saved_at DESC, le.id DESC
            LIMIT $4
            "#,
            defaults.embedding_model,
            defaults.embedding_dim,
            defaults.enabled,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        let document_ids = rows
            .into_iter()
            .map(|row| DocumentId::from_uuid(row.id))
            .collect::<Vec<_>>();
        let queued = i64::try_from(document_ids.len()).map_err(|_| {
            AppError::Repository(Box::new(std::io::Error::other(
                "queued document count overflow",
            )))
        })?;
        if document_ids.is_empty() {
            return Ok(0);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        enqueue_embed_jobs_tx(&mut tx, document_ids).await?;
        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        Ok(queued)
    }

    async fn enqueue_user_vector_repairs(
        &self,
        user_id: UserId,
        target: &EffectiveEmbeddingTarget,
        limit: i64,
    ) -> Result<i64, AppError> {
        if limit <= 0 {
            return Ok(0);
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        let document_ids = target_document_ids(&mut tx, user_id, target, limit, true).await?;
        let queued = i64::try_from(document_ids.len()).map_err(|_| {
            AppError::Repository(Box::new(std::io::Error::other(
                "queued document count overflow",
            )))
        })?;
        enqueue_embed_jobs_tx(&mut tx, document_ids).await?;
        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        Ok(queued)
    }

    async fn retry_user_vector_repairs(
        &self,
        user_id: UserId,
        target: &EffectiveEmbeddingTarget,
        limit: i64,
    ) -> Result<i64, AppError> {
        if limit <= 0 {
            return Ok(0);
        }
        let now = chrono::Utc::now();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        let document_ids = target_document_ids(&mut tx, user_id, target, limit, false).await?;
        let queued = i64::try_from(document_ids.len()).map_err(|_| {
            AppError::Repository(Box::new(std::io::Error::other(
                "queued document count overflow",
            )))
        })?;
        lock_unresolved_embedding_dead_letters_tx(&mut tx, &document_ids).await?;
        let outbox_rows = enqueue_embed_jobs_tx(&mut tx, document_ids).await?;
        for (document_id, outbox_id) in outbox_rows {
            let dedupe_key = format!("{}:{document_id}", job_types::DOCUMENT_AI_EMBED);
            let recovery_key = format!("dedupe:{dedupe_key}");
            sqlx::query!(
                r#"
                UPDATE background_job_recoveries
                SET status = 'resolved',
                    next_retry_at = NULL,
                    lease_owner = NULL,
                    lease_expires_at = NULL,
                    resolved_at = $2,
                    updated_at = $2
                WHERE recovery_key = $1
                  AND status <> 'resolved'
                "#,
                recovery_key,
                now,
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

            sqlx::query!(
                r#"
                UPDATE dead_letter_jobs
                SET replayed_at = $3,
                    replay_outbox_id = $4
                WHERE original_job_type = $1
                  AND original_dedupe_key = $2
                  AND replayed_at IS NULL
                "#,
                job_types::DOCUMENT_AI_EMBED,
                dedupe_key,
                now,
                outbox_id.as_uuid(),
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        }
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
            SELECT COUNT(DISTINCT d.id) AS "count!"
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
              AND EXISTS (
                    SELECT 1
                    FROM content_vectors cv
                    WHERE cv.document_id = d.id
                      AND cv.embedding_model = $2
                      AND cv.embedding_dim = $3
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

    async fn count_stale_items(
        &self,
        user_id: UserId,
        embedding_model: &str,
        embedding_dim: i32,
    ) -> Result<i64, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT d.id) AS "count!"
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
              AND EXISTS (
                    SELECT 1
                    FROM content_vectors cv
                    WHERE cv.document_id = d.id
              )
              AND NOT EXISTS (
                    SELECT 1
                    FROM content_vectors cv
                    WHERE cv.document_id = d.id
                      AND cv.embedding_model = $2
                      AND cv.embedding_dim = $3
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

    async fn has_active_embedding_work(&self, user_id: UserId) -> Result<bool, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM job_outbox jo
                JOIN documents d ON (jo.payload ->> 'document_id') = ('doc_' || d.id::text)
                WHERE jo.job_type = 'document.ai.embed'
                  AND d.user_id = $1
                  AND (
                        jo.dispatched_at IS NULL
                        OR EXISTS (
                            SELECT 1
                            FROM apalis.jobs active_job
                            WHERE active_job.metadata->>'outbox_id' = jo.id::text
                              AND (
                                    active_job.status IN ('Pending', 'Queued', 'Running')
                                    OR (
                                        active_job.status = 'Failed'
                                        AND active_job.attempts < active_job.max_attempts
                                    )
                              )
                        )
                  )
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
