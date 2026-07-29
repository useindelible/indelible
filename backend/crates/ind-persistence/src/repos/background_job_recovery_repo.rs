use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::background_job_recovery::{
    ActiveRecoveryFilter, BackgroundJobRecoveryRepository, DeadLetterInsert, RecoveryFailureInput,
    RecoveryReplay,
};
use ind_application::repos::lifecycle_outbox::OutboxEntry;

use super::write_helpers;
use ind_domain::{
    BackgroundJobFailureClass, BackgroundJobRecovery, BackgroundJobRecoveryId,
    BackgroundJobRecoveryStatus, BackgroundJobSubjectKind, DomainError, JobOutboxId,
};

pub struct PgBackgroundJobRecoveryRepository {
    pool: PgPool,
}

impl PgBackgroundJobRecoveryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BackgroundJobRecoveryRepository for PgBackgroundJobRecoveryRepository {
    async fn upsert_waiting_failure(
        &self,
        input: RecoveryFailureInput<'_>,
    ) -> Result<BackgroundJobRecovery, AppError> {
        let id = BackgroundJobRecoveryId::new();
        let row = sqlx::query_as!(
            RecoveryRow,
            r#"
            INSERT INTO background_job_recoveries (
                id, recovery_key, job_type, payload, dedupe_key, outbox_id,
                subject_kind, subject_id, status, failure_class,
                failure_reason_code, error_message, apalis_attempts,
                recovery_attempts, next_retry_at, lease_owner, lease_expires_at,
                first_failed_at, last_failed_at, resolved_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'waiting', $9, $10, $11,
                    $12, 0, $13, NULL, NULL, $14, $14, NULL, $14, $14)
            ON CONFLICT (recovery_key) DO UPDATE SET
                status              = 'waiting',
                payload             = EXCLUDED.payload,
                dedupe_key          = EXCLUDED.dedupe_key,
                outbox_id           = EXCLUDED.outbox_id,
                subject_kind        = EXCLUDED.subject_kind,
                subject_id          = EXCLUDED.subject_id,
                failure_class       = EXCLUDED.failure_class,
                failure_reason_code = EXCLUDED.failure_reason_code,
                error_message       = EXCLUDED.error_message,
                apalis_attempts     = GREATEST(background_job_recoveries.apalis_attempts, EXCLUDED.apalis_attempts),
                recovery_attempts   = CASE
                    WHEN background_job_recoveries.failure_class <> EXCLUDED.failure_class THEN 0
                    ELSE background_job_recoveries.recovery_attempts
                END,
                next_retry_at       = EXCLUDED.next_retry_at,
                lease_owner         = NULL,
                lease_expires_at    = NULL,
                resolved_at         = NULL,
                last_failed_at      = EXCLUDED.last_failed_at,
                updated_at          = EXCLUDED.updated_at
            RETURNING
                id, recovery_key, job_type, payload, dedupe_key, outbox_id,
                subject_kind, subject_id, status, failure_class,
                failure_reason_code, error_message, apalis_attempts,
                recovery_attempts, next_retry_at, lease_owner, lease_expires_at,
                first_failed_at, last_failed_at, resolved_at, created_at, updated_at
            "#,
            id.into_uuid(),
            input.recovery_key,
            input.job_type,
            input.payload,
            input.dedupe_key,
            input.outbox_id.map(|o| o.into_uuid()),
            input.subject_kind.map(subject_kind_to_str),
            input.subject_id,
            failure_class_to_str(input.failure_class),
            input.failure_reason_code,
            input.error_message,
            input.apalis_attempts,
            input.next_retry_at,
            input.now,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        row.try_into()
    }

    async fn upsert_terminal_failure(
        &self,
        input: RecoveryFailureInput<'_>,
        dlq_insert: DeadLetterInsert<'_>,
    ) -> Result<BackgroundJobRecovery, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        // Serialize concurrent terminal upserts for the same recovery_key so
        // two simultaneous first-time terminal failures cannot both INSERT and
        // both write a DLQ row. The advisory lock is transaction-scoped and is
        // released on commit/rollback.
        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
            input.recovery_key,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        let prior_status = sqlx::query_scalar!(
            "SELECT status FROM background_job_recoveries WHERE recovery_key = $1 FOR UPDATE",
            input.recovery_key,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        let id = BackgroundJobRecoveryId::new();
        let row = sqlx::query_as!(
            RecoveryRow,
            r#"
            INSERT INTO background_job_recoveries (
                id, recovery_key, job_type, payload, dedupe_key, outbox_id,
                subject_kind, subject_id, status, failure_class,
                failure_reason_code, error_message, apalis_attempts,
                recovery_attempts, next_retry_at, lease_owner, lease_expires_at,
                first_failed_at, last_failed_at, resolved_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'terminal', $9, $10, $11,
                    $12, 0, NULL, NULL, NULL, $13, $13, NULL, $13, $13)
            ON CONFLICT (recovery_key) DO UPDATE SET
                status              = 'terminal',
                payload             = EXCLUDED.payload,
                dedupe_key          = EXCLUDED.dedupe_key,
                outbox_id           = EXCLUDED.outbox_id,
                subject_kind        = EXCLUDED.subject_kind,
                subject_id          = EXCLUDED.subject_id,
                failure_class       = EXCLUDED.failure_class,
                failure_reason_code = EXCLUDED.failure_reason_code,
                error_message       = EXCLUDED.error_message,
                apalis_attempts     = GREATEST(background_job_recoveries.apalis_attempts, EXCLUDED.apalis_attempts),
                next_retry_at       = NULL,
                lease_owner         = NULL,
                lease_expires_at    = NULL,
                last_failed_at      = EXCLUDED.last_failed_at,
                updated_at          = EXCLUDED.updated_at
            RETURNING
                id, recovery_key, job_type, payload, dedupe_key, outbox_id,
                subject_kind, subject_id, status, failure_class,
                failure_reason_code, error_message, apalis_attempts,
                recovery_attempts, next_retry_at, lease_owner, lease_expires_at,
                first_failed_at, last_failed_at, resolved_at, created_at, updated_at
            "#,
            id.into_uuid(),
            input.recovery_key,
            input.job_type,
            input.payload,
            input.dedupe_key,
            input.outbox_id.map(|o| o.into_uuid()),
            input.subject_kind.map(subject_kind_to_str),
            input.subject_id,
            failure_class_to_str(input.failure_class),
            input.failure_reason_code,
            input.error_message,
            input.apalis_attempts,
            input.now,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        let was_already_terminal = prior_status.as_deref() == Some("terminal");
        if !was_already_terminal {
            insert_dlq(&mut tx, dlq_insert).await?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        row.try_into()
    }

    async fn mark_resolved(
        &self,
        recovery_key: &str,
        resolved_at: DateTime<Utc>,
    ) -> Result<Option<BackgroundJobRecovery>, AppError> {
        let row = sqlx::query_as!(
            RecoveryRow,
            r#"
            UPDATE background_job_recoveries
            SET status = 'resolved',
                resolved_at = $2,
                updated_at = $2
            WHERE recovery_key = $1
            RETURNING
                id, recovery_key, job_type, payload, dedupe_key, outbox_id,
                subject_kind, subject_id, status, failure_class,
                failure_reason_code, error_message, apalis_attempts,
                recovery_attempts, next_retry_at, lease_owner, lease_expires_at,
                first_failed_at, last_failed_at, resolved_at, created_at, updated_at
            "#,
            recovery_key,
            resolved_at,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        row.map(BackgroundJobRecovery::try_from).transpose()
    }

    async fn claim_due(
        &self,
        now: DateTime<Utc>,
        lease_owner: &str,
        lease_until: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<BackgroundJobRecovery>, AppError> {
        let rows = sqlx::query_as!(
            RecoveryRow,
            r#"
            UPDATE background_job_recoveries
            SET status = 'leased',
                lease_owner = $2,
                lease_expires_at = $3,
                updated_at = $1
            WHERE id IN (
                SELECT id FROM background_job_recoveries
                WHERE next_retry_at <= $1
                  AND (
                      status = 'waiting'
                      OR (status = 'leased' AND lease_expires_at IS NOT NULL AND lease_expires_at <= $1)
                  )
                ORDER BY next_retry_at
                FOR UPDATE SKIP LOCKED
                LIMIT $4
            )
            RETURNING
                id, recovery_key, job_type, payload, dedupe_key, outbox_id,
                subject_kind, subject_id, status, failure_class,
                failure_reason_code, error_message, apalis_attempts,
                recovery_attempts, next_retry_at, lease_owner, lease_expires_at,
                first_failed_at, last_failed_at, resolved_at, created_at, updated_at
            "#,
            now,
            lease_owner,
            lease_until,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        rows.into_iter()
            .map(BackgroundJobRecovery::try_from)
            .collect()
    }

    async fn replay_via_outbox(&self, replay: RecoveryReplay<'_>) -> Result<JobOutboxId, AppError> {
        let RecoveryReplay {
            id,
            job_type,
            payload,
            dedupe_key,
            lease_owner,
            next_retry_at,
            now,
        } = replay;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        let outbox_id = write_helpers::enqueue_outbox_tx(
            &mut tx,
            &OutboxEntry {
                job_type: job_type.to_string(),
                payload,
                dedupe_key: dedupe_key.map(str::to_string),
                available_at: now,
            },
        )
        .await?;

        let updated = sqlx::query!(
            r#"
            UPDATE background_job_recoveries
            SET status = 'waiting',
                outbox_id = $2,
                recovery_attempts = recovery_attempts + 1,
                lease_owner = NULL,
                lease_expires_at = NULL,
                next_retry_at = $3,
                updated_at = $4
            WHERE id = $1
              AND status = 'leased'
              AND lease_owner = $5
              AND lease_expires_at > $4
            "#,
            id.into_uuid(),
            outbox_id.into_uuid(),
            next_retry_at,
            now,
            lease_owner,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        if updated.rows_affected() != 1 {
            return Err(AppError::Domain(DomainError::InvariantViolation {
                message: format!(
                    "background_job_recovery {id} is no longer leased by {lease_owner}; skipping replay"
                ),
            }));
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;
        Ok(outbox_id)
    }

    async fn reschedule_waiting(
        &self,
        id: BackgroundJobRecoveryId,
        next_retry_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE background_job_recoveries
            SET next_retry_at = $2,
                updated_at = $3
            WHERE id = $1 AND status = 'waiting'
            "#,
            id.into_uuid(),
            next_retry_at,
            now,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;
        Ok(())
    }

    async fn mark_recovery_terminal(
        &self,
        id: BackgroundJobRecoveryId,
        reason: &str,
        error: &str,
        lease_owner: &str,
        dlq_insert: DeadLetterInsert<'_>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        let updated = sqlx::query!(
            r#"
            UPDATE background_job_recoveries
            SET status = 'terminal',
                failure_class = 'terminal',
                failure_reason_code = $2,
                error_message = $3,
                next_retry_at = NULL,
                lease_owner = NULL,
                lease_expires_at = NULL,
                last_failed_at = $4,
                updated_at = $4
            WHERE id = $1
              AND status = 'leased'
              AND lease_owner = $5
              AND lease_expires_at > $4
            "#,
            id.into_uuid(),
            reason,
            error,
            now,
            lease_owner,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        if updated.rows_affected() != 1 {
            return Err(AppError::Domain(DomainError::InvariantViolation {
                message: format!(
                    "background_job_recovery {id} is no longer leased by {lease_owner}; skipping terminalization"
                ),
            }));
        }

        // The lease fence guarantees the row was not already terminal, so the
        // DLQ write happens at most once per terminalization.
        insert_dlq(&mut tx, dlq_insert).await?;

        tx.commit()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(())
    }

    async fn defer_recovery(
        &self,
        id: BackgroundJobRecoveryId,
        next_retry_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE background_job_recoveries
            SET status = 'waiting',
                recovery_attempts = recovery_attempts + 1,
                lease_owner = NULL,
                lease_expires_at = NULL,
                next_retry_at = $2,
                updated_at = $3
            WHERE id = $1
            "#,
            id.into_uuid(),
            next_retry_at,
            now,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;
        Ok(())
    }

    async fn list_active(
        &self,
        filter: ActiveRecoveryFilter,
        limit: i64,
    ) -> Result<Vec<BackgroundJobRecovery>, AppError> {
        let status = filter.status.map(status_to_str);
        let subject_kind = filter.subject_kind.map(subject_kind_to_str);
        let rows = sqlx::query_as!(
            RecoveryRow,
            r#"
            SELECT
                id, recovery_key, job_type, payload, dedupe_key, outbox_id,
                subject_kind, subject_id, status, failure_class,
                failure_reason_code, error_message, apalis_attempts,
                recovery_attempts, next_retry_at, lease_owner, lease_expires_at,
                first_failed_at, last_failed_at, resolved_at, created_at, updated_at
            FROM background_job_recoveries
            WHERE ($1::text IS NULL OR status = $1)
              AND ($1::text IS NOT NULL OR status <> 'resolved')
              AND ($2::text IS NULL OR job_type = $2)
              AND ($3::text IS NULL OR subject_kind = $3)
            ORDER BY updated_at DESC
            LIMIT $4
            "#,
            status,
            filter.job_type.as_deref(),
            subject_kind,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        rows.into_iter()
            .map(BackgroundJobRecovery::try_from)
            .collect()
    }
}

async fn insert_dlq(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    insert: DeadLetterInsert<'_>,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        INSERT INTO dead_letter_jobs
            (id, original_job_type, original_payload, original_dedupe_key,
             failure_reason_code, error_message, attempts, failed_at)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7)
        "#,
        insert.job_type,
        insert.payload,
        insert.dedupe_key,
        insert.failure_reason_code,
        insert.error_message,
        insert.attempts,
        insert.failed_at,
    )
    .execute(&mut **tx)
    .await
    .map_err(|e| AppError::Repository(Box::new(e)))?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct RecoveryRow {
    id: Uuid,
    recovery_key: String,
    job_type: String,
    payload: serde_json::Value,
    dedupe_key: Option<String>,
    outbox_id: Option<Uuid>,
    subject_kind: Option<String>,
    subject_id: Option<String>,
    status: String,
    failure_class: String,
    failure_reason_code: String,
    error_message: String,
    apalis_attempts: i32,
    recovery_attempts: i32,
    next_retry_at: Option<DateTime<Utc>>,
    lease_owner: Option<String>,
    lease_expires_at: Option<DateTime<Utc>>,
    first_failed_at: DateTime<Utc>,
    last_failed_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<RecoveryRow> for BackgroundJobRecovery {
    type Error = AppError;

    fn try_from(row: RecoveryRow) -> Result<Self, Self::Error> {
        Ok(BackgroundJobRecovery {
            id: BackgroundJobRecoveryId::from_uuid(row.id),
            recovery_key: row.recovery_key,
            job_type: row.job_type,
            payload: row.payload,
            dedupe_key: row.dedupe_key,
            outbox_id: row.outbox_id.map(JobOutboxId::from_uuid),
            subject_kind: row
                .subject_kind
                .as_deref()
                .map(parse_subject_kind)
                .transpose()?,
            subject_id: row.subject_id,
            status: parse_status(&row.status)?,
            failure_class: parse_failure_class(&row.failure_class)?,
            failure_reason_code: row.failure_reason_code,
            error_message: row.error_message,
            apalis_attempts: row.apalis_attempts,
            recovery_attempts: row.recovery_attempts,
            next_retry_at: row.next_retry_at,
            lease_owner: row.lease_owner,
            lease_expires_at: row.lease_expires_at,
            first_failed_at: row.first_failed_at,
            last_failed_at: row.last_failed_at,
            resolved_at: row.resolved_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

fn parse_status(s: &str) -> Result<BackgroundJobRecoveryStatus, AppError> {
    match s {
        "waiting" => Ok(BackgroundJobRecoveryStatus::Waiting),
        "leased" => Ok(BackgroundJobRecoveryStatus::Leased),
        "terminal" => Ok(BackgroundJobRecoveryStatus::Terminal),
        "resolved" => Ok(BackgroundJobRecoveryStatus::Resolved),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid background_job_recovery status: {other}"),
        })),
    }
}

fn parse_failure_class(s: &str) -> Result<BackgroundJobFailureClass, AppError> {
    match s {
        "retryable" => Ok(BackgroundJobFailureClass::Retryable),
        "terminal" => Ok(BackgroundJobFailureClass::Terminal),
        "patient" => Ok(BackgroundJobFailureClass::Patient),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid background_job_recovery failure_class: {other}"),
        })),
    }
}

fn parse_subject_kind(s: &str) -> Result<BackgroundJobSubjectKind, AppError> {
    match s {
        "document" => Ok(BackgroundJobSubjectKind::Document),
        "library_entry" => Ok(BackgroundJobSubjectKind::LibraryEntry),
        "feed_delivery" => Ok(BackgroundJobSubjectKind::FeedDelivery),
        "feed_source" => Ok(BackgroundJobSubjectKind::FeedSource),
        "integration_connection" => Ok(BackgroundJobSubjectKind::IntegrationConnection),
        "import_job" => Ok(BackgroundJobSubjectKind::ImportJob),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid background_job_recovery subject_kind: {other}"),
        })),
    }
}

fn status_to_str(status: BackgroundJobRecoveryStatus) -> &'static str {
    match status {
        BackgroundJobRecoveryStatus::Waiting => "waiting",
        BackgroundJobRecoveryStatus::Leased => "leased",
        BackgroundJobRecoveryStatus::Terminal => "terminal",
        BackgroundJobRecoveryStatus::Resolved => "resolved",
    }
}

fn failure_class_to_str(class: BackgroundJobFailureClass) -> &'static str {
    match class {
        BackgroundJobFailureClass::Retryable => "retryable",
        BackgroundJobFailureClass::Terminal => "terminal",
        BackgroundJobFailureClass::Patient => "patient",
    }
}

fn subject_kind_to_str(kind: BackgroundJobSubjectKind) -> &'static str {
    match kind {
        BackgroundJobSubjectKind::Document => "document",
        BackgroundJobSubjectKind::LibraryEntry => "library_entry",
        BackgroundJobSubjectKind::FeedDelivery => "feed_delivery",
        BackgroundJobSubjectKind::FeedSource => "feed_source",
        BackgroundJobSubjectKind::IntegrationConnection => "integration_connection",
        BackgroundJobSubjectKind::ImportJob => "import_job",
    }
}
