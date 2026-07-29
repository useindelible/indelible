use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use ind_application::error::AppError;
use ind_application::recovery_keys::recovery_key_for;
use ind_application::repos::dead_letter::{
    DeadLetterReplay, DeadLetterRepository, DeadLetterStats,
};
use ind_domain::{DeadLetterJob, DeadLetterJobId, DomainError, JobOutbox, JobOutboxId};

pub struct PgDeadLetterRepository {
    pool: PgPool,
}

impl PgDeadLetterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn repo_error(error: sqlx::Error) -> AppError {
    AppError::Repository(Box::new(error))
}

fn not_found(id: DeadLetterJobId) -> AppError {
    AppError::Domain(DomainError::NotFound {
        entity: "DeadLetterJob",
        id: id.to_string(),
    })
}

#[async_trait]
impl DeadLetterRepository for PgDeadLetterRepository {
    async fn insert(&self, job: DeadLetterJob) -> Result<DeadLetterJob, AppError> {
        sqlx::query!(
            "INSERT INTO dead_letter_jobs (id, original_job_type, original_payload, original_dedupe_key, failure_reason_code, error_message, attempts, failed_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            job.id.into_uuid(),
            &job.original_job_type,
            &job.original_payload,
            job.original_dedupe_key.as_deref(),
            job.failure_reason_code.as_deref(),
            &job.error_message,
            job.attempts,
            job.failed_at,
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;

        Ok(job)
    }

    async fn get(&self, id: DeadLetterJobId) -> Result<DeadLetterJob, AppError> {
        let row = sqlx::query_as!(
            DlqRow,
            "SELECT id, original_job_type, original_payload, original_dedupe_key, failure_reason_code, error_message, attempts, failed_at, replayed_at, replay_outbox_id \
             FROM dead_letter_jobs \
             WHERE id = $1",
            id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(repo_error)?
        .ok_or_else(|| not_found(id))?;

        Ok(row.into_domain())
    }

    async fn list(&self, limit: i64) -> Result<Vec<DeadLetterJob>, AppError> {
        let rows = sqlx::query_as!(
            DlqRow,
            "SELECT id, original_job_type, original_payload, original_dedupe_key, failure_reason_code, error_message, attempts, failed_at, replayed_at, replay_outbox_id \
             FROM dead_letter_jobs \
             ORDER BY failed_at DESC, id DESC \
             LIMIT $1",
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(repo_error)?;

        Ok(rows.into_iter().map(DlqRow::into_domain).collect())
    }

    async fn replay(
        &self,
        id: DeadLetterJobId,
        available_at: DateTime<Utc>,
    ) -> Result<DeadLetterReplay, AppError> {
        let mut tx = self.pool.begin().await.map_err(repo_error)?;
        let row = sqlx::query_as!(
            DlqRow,
            r#"
            SELECT id, original_job_type, original_payload, original_dedupe_key,
                   failure_reason_code, error_message, attempts, failed_at,
                   replayed_at, replay_outbox_id
            FROM dead_letter_jobs
            WHERE id = $1
            FOR UPDATE
            "#,
            id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(repo_error)?
        .ok_or_else(|| not_found(id))?;
        let mut dead_letter = row.into_domain();

        if let Some(replay_outbox_id) = dead_letter.replay_outbox_id {
            let outbox = sqlx::query_as!(
                OutboxRow,
                r#"
                SELECT id, job_type, payload, dedupe_key, available_at, dispatched_at, created_at
                FROM job_outbox
                WHERE id = $1
                "#,
                replay_outbox_id.as_uuid(),
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(repo_error)?
            .into_domain();
            tx.commit().await.map_err(repo_error)?;
            return Ok(DeadLetterReplay {
                dead_letter,
                outbox,
                queued: false,
            });
        }

        let replay_dedupe_key = dead_letter
            .original_dedupe_key
            .clone()
            .unwrap_or_else(|| format!("dead-letter.replay:{id}"));
        let replayed_at = Utc::now();
        let outbox_id = JobOutboxId::new();
        let outbox = sqlx::query_as!(
            OutboxRow,
            r#"
            INSERT INTO job_outbox (
                id, job_type, payload, dedupe_key, available_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (dedupe_key) WHERE dedupe_key IS NOT NULL DO UPDATE SET
                payload = EXCLUDED.payload,
                available_at = CASE
                    WHEN job_outbox.dispatched_at IS NULL
                        THEN LEAST(job_outbox.available_at, EXCLUDED.available_at)
                    ELSE EXCLUDED.available_at
                END,
                dispatched_at = NULL
            RETURNING id, job_type, payload, dedupe_key, available_at, dispatched_at, created_at
            "#,
            outbox_id.as_uuid(),
            &dead_letter.original_job_type,
            &dead_letter.original_payload,
            replay_dedupe_key,
            available_at,
            replayed_at,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(repo_error)?
        .into_domain();

        let recovery_key = recovery_key_for(
            &dead_letter.original_job_type,
            &dead_letter.original_payload,
            dead_letter.original_dedupe_key.as_deref(),
        );
        sqlx::query!(
            r#"
            UPDATE background_job_recoveries
            SET status = 'resolved',
                next_retry_at = NULL,
                lease_owner = NULL,
                lease_expires_at = NULL,
                resolved_at = $2,
                updated_at = $2
            WHERE recovery_key = $1 AND status = 'terminal'
            "#,
            recovery_key,
            replayed_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(repo_error)?;

        sqlx::query!(
            r#"
            UPDATE dead_letter_jobs
            SET replayed_at = $2, replay_outbox_id = $3
            WHERE id = $1 AND replayed_at IS NULL
            "#,
            id.into_uuid(),
            replayed_at,
            outbox.id.as_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(repo_error)?;
        tx.commit().await.map_err(repo_error)?;

        dead_letter.replayed_at = Some(replayed_at);
        dead_letter.replay_outbox_id = Some(outbox.id);
        Ok(DeadLetterReplay {
            dead_letter,
            outbox,
            queued: true,
        })
    }

    async fn stats(&self) -> Result<DeadLetterStats, AppError> {
        let row = sqlx::query_as!(
            DlqStatsRow,
            "SELECT \
                COUNT(*) FILTER (WHERE replayed_at IS NULL)::BIGINT AS \"unresolved!\", \
                COUNT(*) FILTER (WHERE replayed_at IS NOT NULL)::BIGINT AS \"replayed!\", \
                COUNT(DISTINCT original_job_type) FILTER (WHERE replayed_at IS NULL)::BIGINT AS \"distinct_unresolved_job_types!\", \
                MIN(failed_at) FILTER (WHERE replayed_at IS NULL) AS oldest_unresolved_failed_at, \
                MAX(failed_at) FILTER (WHERE replayed_at IS NULL) AS newest_unresolved_failed_at \
             FROM dead_letter_jobs",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(repo_error)?;

        Ok(DeadLetterStats {
            unresolved: row.unresolved,
            replayed: row.replayed,
            distinct_unresolved_job_types: row.distinct_unresolved_job_types,
            oldest_unresolved_failed_at: row.oldest_unresolved_failed_at,
            newest_unresolved_failed_at: row.newest_unresolved_failed_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct DlqRow {
    id: uuid::Uuid,
    original_job_type: String,
    original_payload: serde_json::Value,
    original_dedupe_key: Option<String>,
    failure_reason_code: Option<String>,
    error_message: String,
    attempts: i32,
    failed_at: DateTime<Utc>,
    replayed_at: Option<DateTime<Utc>>,
    replay_outbox_id: Option<uuid::Uuid>,
}

impl DlqRow {
    fn into_domain(self) -> DeadLetterJob {
        DeadLetterJob {
            id: DeadLetterJobId::from_uuid(self.id),
            original_job_type: self.original_job_type,
            original_payload: self.original_payload,
            original_dedupe_key: self.original_dedupe_key,
            failure_reason_code: self.failure_reason_code,
            error_message: self.error_message,
            attempts: self.attempts,
            failed_at: self.failed_at,
            replayed_at: self.replayed_at,
            replay_outbox_id: self.replay_outbox_id.map(JobOutboxId::from_uuid),
        }
    }
}

struct DlqStatsRow {
    unresolved: i64,
    replayed: i64,
    distinct_unresolved_job_types: i64,
    oldest_unresolved_failed_at: Option<DateTime<Utc>>,
    newest_unresolved_failed_at: Option<DateTime<Utc>>,
}

struct OutboxRow {
    id: uuid::Uuid,
    job_type: String,
    payload: serde_json::Value,
    dedupe_key: Option<String>,
    available_at: DateTime<Utc>,
    dispatched_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl OutboxRow {
    fn into_domain(self) -> JobOutbox {
        JobOutbox {
            id: JobOutboxId::from_uuid(self.id),
            job_type: self.job_type,
            payload: self.payload,
            dedupe_key: self.dedupe_key,
            available_at: self.available_at,
            dispatched_at: self.dispatched_at,
            created_at: self.created_at,
        }
    }
}
