use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_application::repos::search_reindex::{
    FullSearchReindexAdmission, SearchReindexCursor, SearchReindexRepository,
};
use ind_domain::{JobOutbox, JobOutboxId, SearchReindexAllJob, job_types};
use sqlx::PgPool;

pub struct PgSearchReindexRepository {
    pool: PgPool,
}

const FULL_REINDEX_ADMISSION_LOCK: &str = "search.reindex_all:admission";
const MAX_VERSION_REINDEX_TERMINAL_RUNS: i64 = 3;

impl PgSearchReindexRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SearchReindexRepository for PgSearchReindexRepository {
    async fn enqueue_full_reindex(
        &self,
        page_size: u32,
        target_version: Option<i32>,
        available_at: DateTime<Utc>,
    ) -> Result<FullSearchReindexAdmission, AppError> {
        let dedupe_key = target_version.map_or_else(
            || "search.reindex_all:manual".to_string(),
            |version| format!("search.reindex_all:version:{version}"),
        );
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|error| AppError::Repository(Box::new(error)))?;
        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
            FULL_REINDEX_ADMISSION_LOCK,
        )
        .execute(&mut *tx)
        .await
        .map_err(|error| AppError::Repository(Box::new(error)))?;

        if let Some(target_version) = target_version {
            let current_version = sqlx::query_scalar!(
                "SELECT current_version FROM search_index_state WHERE singleton = true FOR UPDATE",
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|error| AppError::Repository(Box::new(error)))?;
            if current_version >= target_version {
                tx.commit()
                    .await
                    .map_err(|error| AppError::Repository(Box::new(error)))?;
                return Ok(FullSearchReindexAdmission {
                    queued: false,
                    outbox: None,
                });
            }
            sqlx::query!(
                r#"
                UPDATE search_index_state
                SET target_version = $1,
                    cursor_created_at = CASE WHEN target_version = $1 THEN cursor_created_at END,
                    cursor_document_id = CASE WHEN target_version = $1 THEN cursor_document_id END,
                    updated_at = now()
                WHERE singleton = true
                "#,
                target_version,
            )
            .execute(&mut *tx)
            .await
            .map_err(|error| AppError::Repository(Box::new(error)))?;
        }

        let existing = sqlx::query_as!(
            JobOutboxRow,
            r#"
            SELECT id, job_type, payload, dedupe_key, available_at, dispatched_at, created_at
            FROM job_outbox
            WHERE job_type = $1
            ORDER BY (dedupe_key = $2) DESC, created_at DESC
            LIMIT 1
            FOR UPDATE
            "#,
            job_types::SEARCH_REINDEX_ALL,
            dedupe_key,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|error| AppError::Repository(Box::new(error)))?;

        if let Some(existing) = existing.as_ref()
            && existing.dispatched_at.is_none()
        {
            let outbox = if target_version.is_some() {
                let payload = serde_json::to_value(SearchReindexAllJob {
                    page_size: Some(page_size),
                    target_version,
                })
                .map_err(|error| AppError::Repository(Box::new(error)))?;
                sqlx::query_as!(
                    JobOutboxRow,
                    r#"
                    UPDATE job_outbox
                    SET payload = $2, dedupe_key = $3, available_at = LEAST(available_at, $4)
                    WHERE id = $1
                    RETURNING id, job_type, payload, dedupe_key, available_at, dispatched_at, created_at
                    "#,
                    existing.id,
                    payload,
                    dedupe_key,
                    available_at,
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|error| AppError::Repository(Box::new(error)))?
                .into_domain()
            } else {
                existing.clone().into_domain()
            };
            tx.commit()
                .await
                .map_err(|error| AppError::Repository(Box::new(error)))?;
            return Ok(FullSearchReindexAdmission {
                queued: false,
                outbox: Some(outbox),
            });
        }

        let active = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM apalis.jobs
                WHERE metadata->>'job_type' = $1
                  AND (
                    status IN ('Pending', 'Queued', 'Running')
                    OR (status = 'Failed' AND attempts < max_attempts)
                  )
            ) AS "exists!"
            "#,
            job_types::SEARCH_REINDEX_ALL,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|error| AppError::Repository(Box::new(error)))?;
        if active {
            tx.commit()
                .await
                .map_err(|error| AppError::Repository(Box::new(error)))?;
            return Ok(FullSearchReindexAdmission {
                queued: false,
                outbox: existing.map(JobOutboxRow::into_domain),
            });
        }

        if target_version.is_some() {
            let terminal_runs = sqlx::query_scalar!(
                r#"
                SELECT count(*)
                FROM apalis.jobs
                WHERE metadata->>'dedupe_key' = $1
                  AND metadata->>'job_type' = $2
                  AND status = 'Failed'
                  AND attempts >= max_attempts
                "#,
                dedupe_key,
                job_types::SEARCH_REINDEX_ALL,
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|error| AppError::Repository(Box::new(error)))?
            .unwrap_or(0);
            if terminal_runs >= MAX_VERSION_REINDEX_TERMINAL_RUNS {
                tx.commit()
                    .await
                    .map_err(|error| AppError::Repository(Box::new(error)))?;
                return Ok(FullSearchReindexAdmission {
                    queued: false,
                    outbox: existing.map(JobOutboxRow::into_domain),
                });
            }
        }

        let payload = serde_json::to_value(SearchReindexAllJob {
            page_size: Some(page_size),
            target_version,
        })
        .map_err(|error| AppError::Repository(Box::new(error)))?;
        let row = if let Some(existing) = existing {
            sqlx::query_as!(
                JobOutboxRow,
                r#"
                UPDATE job_outbox
                SET job_type = $2, payload = $3, dedupe_key = $4,
                    available_at = $5, dispatched_at = NULL
                WHERE id = $1
                RETURNING id, job_type, payload, dedupe_key, available_at, dispatched_at, created_at
                "#,
                existing.id,
                job_types::SEARCH_REINDEX_ALL,
                payload,
                dedupe_key,
                available_at,
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|error| AppError::Repository(Box::new(error)))?
        } else {
            sqlx::query_as!(
                JobOutboxRow,
                r#"
                INSERT INTO job_outbox (
                    id, job_type, payload, dedupe_key, available_at, created_at
                )
                VALUES ($1, $2, $3, $4, $5, now())
                RETURNING id, job_type, payload, dedupe_key, available_at, dispatched_at, created_at
                "#,
                JobOutboxId::new().into_uuid(),
                job_types::SEARCH_REINDEX_ALL,
                payload,
                dedupe_key,
                available_at,
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|error| AppError::Repository(Box::new(error)))?
        };
        tx.commit()
            .await
            .map_err(|error| AppError::Repository(Box::new(error)))?;

        Ok(FullSearchReindexAdmission {
            queued: true,
            outbox: Some(row.into_domain()),
        })
    }

    async fn complete_version(&self, version: i32) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE search_index_state
            SET current_version = GREATEST(current_version, $1),
                target_version = NULL,
                cursor_created_at = NULL,
                cursor_document_id = NULL,
                updated_at = now()
            WHERE singleton = true
            "#,
            version,
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|error| AppError::Repository(Box::new(error)))
    }

    async fn load_version_cursor(
        &self,
        version: i32,
    ) -> Result<Option<SearchReindexCursor>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT cursor_created_at, cursor_document_id
            FROM search_index_state
            WHERE singleton = true AND target_version = $1
            "#,
            version,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| AppError::Repository(Box::new(error)))?;

        match row.map(|row| (row.cursor_created_at, row.cursor_document_id)) {
            Some((Some(created_at), Some(document_id))) => Ok(Some(SearchReindexCursor {
                created_at,
                document_id,
            })),
            _ => Ok(None),
        }
    }

    async fn checkpoint_version_cursor(
        &self,
        version: i32,
        cursor: SearchReindexCursor,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE search_index_state
            SET cursor_created_at = $2, cursor_document_id = $3, updated_at = now()
            WHERE singleton = true AND target_version = $1 AND current_version < $1
            "#,
            version,
            cursor.created_at,
            cursor.document_id,
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|error| AppError::Repository(Box::new(error)))
    }
}

#[derive(Clone)]
struct JobOutboxRow {
    id: uuid::Uuid,
    job_type: String,
    payload: serde_json::Value,
    dedupe_key: Option<String>,
    available_at: DateTime<Utc>,
    dispatched_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl JobOutboxRow {
    fn into_domain(self) -> JobOutbox {
        JobOutbox {
            id: JobOutboxId::from(self.id),
            job_type: self.job_type,
            payload: self.payload,
            dedupe_key: self.dedupe_key,
            available_at: self.available_at,
            dispatched_at: self.dispatched_at,
            created_at: self.created_at,
        }
    }
}
