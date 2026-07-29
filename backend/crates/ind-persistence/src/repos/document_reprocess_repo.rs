use std::time::Duration;

use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_application::repos::document_reprocess::{
    CompleteUploadReprocess, DocumentReprocessAdmission, DocumentReprocessRepository,
};
use ind_application::repos::lifecycle_outbox::{
    document_ai_processing_outbox, search_reindex_document_outbox,
};
use ind_domain::{DomainError, JobOutboxId, ReprocessDocumentJob, job_types};
use sqlx::PgPool;

use super::document_upload_repo::{set_document_upload_metadata_tx, upsert_staged_asset_tx};
use super::write_helpers::apply_outbox_tx;

pub struct PgDocumentReprocessRepository {
    pool: PgPool,
}

impl PgDocumentReprocessRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DocumentReprocessRepository for PgDocumentReprocessRepository {
    async fn admit(
        &self,
        job: ReprocessDocumentJob,
        requested_at: DateTime<Utc>,
        cooldown: Duration,
    ) -> Result<DocumentReprocessAdmission, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        let dedupe_key = format!("{}:{}", job_types::DOCUMENT_REPROCESS, job.document_id);
        let exact_processing_keys = [
            dedupe_key.clone(),
            format!("{}:{}", job_types::FEED_PREPARE_DOCUMENT, job.document_id),
            format!("{}:{}", job_types::DOCUMENT_YOUTUBE_INGEST, job.document_id),
        ];
        let attach_processing_pattern = format!(
            "{}:{}:%",
            job_types::DOCUMENT_ATTACH_PROVIDED_CONTENT,
            job.document_id
        );

        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
            dedupe_key,
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        let existing = sqlx::query_as!(
            ReprocessOutboxRow,
            "SELECT id, dispatched_at FROM job_outbox WHERE dedupe_key = $1 FOR UPDATE",
            dedupe_key,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        if existing
            .as_ref()
            .is_some_and(|row| row.dispatched_at.is_none())
        {
            tx.commit()
                .await
                .map_err(|err| AppError::Repository(Box::new(err)))?;
            return Ok(DocumentReprocessAdmission {
                queued: false,
                retry_after_seconds: None,
            });
        }

        let has_pending_processing = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM job_outbox
                WHERE dispatched_at IS NULL
                  AND (
                    dedupe_key = ANY($1)
                    OR dedupe_key LIKE $2
                  )
            ) AS "exists!"
            "#,
            &exact_processing_keys[..],
            attach_processing_pattern,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;
        if has_pending_processing {
            tx.commit()
                .await
                .map_err(|err| AppError::Repository(Box::new(err)))?;
            return Ok(DocumentReprocessAdmission {
                queued: false,
                retry_after_seconds: None,
            });
        }

        let has_active_job = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM apalis.jobs
                WHERE (
                    metadata->>'dedupe_key' = ANY($1)
                    OR metadata->>'dedupe_key' LIKE $2
                  )
                  AND (
                    status IN ('Pending', 'Queued', 'Running')
                    OR (status = 'Failed' AND attempts < max_attempts)
                  )
            ) AS "exists!"
            "#,
            &exact_processing_keys[..],
            attach_processing_pattern,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;
        if has_active_job {
            tx.commit()
                .await
                .map_err(|err| AppError::Repository(Box::new(err)))?;
            return Ok(DocumentReprocessAdmission {
                queued: false,
                retry_after_seconds: None,
            });
        }

        if let Some(dispatched_at) = existing.as_ref().and_then(|row| row.dispatched_at) {
            let cooldown = chrono::Duration::from_std(cooldown).map_err(|err| {
                AppError::Domain(DomainError::InvariantViolation {
                    message: format!("invalid document reprocess cooldown: {err}"),
                })
            })?;
            let ready_at = dispatched_at + cooldown;
            if ready_at > requested_at {
                let remaining_millis = (ready_at - requested_at).num_milliseconds();
                let retry_after_seconds = remaining_millis
                    .saturating_add(999)
                    .checked_div(1_000)
                    .and_then(|seconds| seconds.try_into().ok());
                tx.commit()
                    .await
                    .map_err(|err| AppError::Repository(Box::new(err)))?;
                return Ok(DocumentReprocessAdmission {
                    queued: false,
                    retry_after_seconds,
                });
            }
        }

        let payload =
            serde_json::to_value(&job).map_err(|err| AppError::Repository(Box::new(err)))?;
        if let Some(existing) = existing {
            sqlx::query!(
                "UPDATE job_outbox \
                 SET job_type = $2, payload = $3, available_at = $4, dispatched_at = NULL \
                 WHERE id = $1",
                existing.id,
                job_types::DOCUMENT_REPROCESS,
                payload,
                requested_at,
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        } else {
            sqlx::query!(
                "INSERT INTO job_outbox \
                    (id, job_type, payload, dedupe_key, available_at, created_at) \
                 VALUES ($1, $2, $3, $4, $5, $5)",
                JobOutboxId::new().into_uuid(),
                job_types::DOCUMENT_REPROCESS,
                payload,
                dedupe_key,
                requested_at,
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        }

        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        Ok(DocumentReprocessAdmission {
            queued: true,
            retry_after_seconds: None,
        })
    }

    async fn complete_upload(&self, request: CompleteUploadReprocess) -> Result<(), AppError> {
        if request
            .assets
            .iter()
            .any(|asset| asset.asset_kind == ind_domain::ArchiveAssetKind::OriginalUpload)
        {
            return Err(AppError::Domain(DomainError::InvariantViolation {
                message: "document reprocess must not replace the preserved original upload".into(),
            }));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        let owned = sqlx::query_scalar!(
            "SELECT EXISTS (SELECT 1 FROM documents WHERE id = $1 AND user_id = $2) AS \"exists!\"",
            request.document_id.into_uuid(),
            request.user_id.into_uuid(),
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;
        if !owned {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "Document",
                id: request.document_id.to_string(),
            }));
        }

        sqlx::query!(
            "DELETE FROM archive_assets \
             WHERE document_id = $1 AND asset_kind = 'extracted_text' AND status <> 'completed'",
            request.document_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        for asset in request.assets {
            upsert_staged_asset_tx(&mut tx, request.document_id, asset).await?;
        }
        if request.word_count.is_some() || request.reading_time_minutes.is_some() {
            set_document_upload_metadata_tx(
                &mut tx,
                request.user_id,
                request.document_id,
                request.word_count,
                request.reading_time_minutes,
                None,
            )
            .await?;
        }

        let now = Utc::now();
        let mut outbox = vec![search_reindex_document_outbox(request.document_id, now)];
        outbox.extend(document_ai_processing_outbox(request.document_id, now));
        apply_outbox_tx(&mut tx, &outbox).await?;

        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        Ok(())
    }
}

struct ReprocessOutboxRow {
    id: uuid::Uuid,
    dispatched_at: Option<DateTime<Utc>>,
}
