use apalis::prelude::Task;
use apalis_postgres::sink::push_tasks;
use apalis_postgres::{Config, PgContext};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};

use ind_application::error::AppError;
use ind_domain::{
    DomainError, GenericJobEnvelope, JobOutboxId, job_priority_for, retry_policy_for,
};

const ACTIVE_DEDUPE_DEFER_SECONDS: i64 = 30;

#[derive(Debug)]
pub struct OutboxHandoffStats {
    pub claimed: usize,
    pub relayed: usize,
    pub deduped: usize,
}

pub struct PgOutboxHandoff {
    pool: PgPool,
    config: Config,
}

impl PgOutboxHandoff {
    pub fn new(pool: PgPool, config: Config) -> Self {
        Self { pool, config }
    }

    /// Atomically claims pending outbox rows and inserts them into the Apalis
    /// Postgres job table, then stamps `dispatched_at`.
    ///
    /// Uses `FOR UPDATE SKIP LOCKED` so multiple relay instances are safe.
    pub async fn handoff_batch(&self, batch_size: i64) -> Result<OutboxHandoffStats, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        let rows = sqlx::query_as!(
            OutboxRow,
            r#"
            SELECT id, job_type, payload, dedupe_key, available_at
            FROM job_outbox
            WHERE dispatched_at IS NULL
              AND available_at <= now()
            ORDER BY available_at, created_at
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
            batch_size,
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        let claimed = rows.len();
        let mut relayed = 0;
        let mut deduped = 0;
        let mut tasks = Vec::with_capacity(rows.len());
        let mut relayed_row_ids = Vec::with_capacity(rows.len());

        for row in &rows {
            if let Some(dedupe_key) = row.dedupe_key.as_deref()
                && has_active_apalis_dedupe(&mut tx, dedupe_key, &row.job_type).await?
            {
                defer_active_deduped_outbox_row(&mut tx, row.id).await?;
                deduped += 1;
                continue;
            }

            let envelope = GenericJobEnvelope {
                outbox_id: JobOutboxId::from(row.id),
                job_type: row.job_type.clone(),
                payload: row.payload.clone(),
                dedupe_key: row.dedupe_key.clone(),
            };

            let envelope_bytes =
                serde_json::to_vec(&envelope).map_err(|e| AppError::Repository(Box::new(e)))?;

            let mut meta = serde_json::Map::new();
            meta.insert("outbox_id".into(), serde_json::json!(row.id));
            meta.insert("job_type".into(), serde_json::json!(row.job_type));
            if let Some(ref dedupe_key) = row.dedupe_key {
                meta.insert("dedupe_key".into(), serde_json::json!(dedupe_key));
            }

            let policy = retry_policy_for(&row.job_type);
            let mut task = Task::new_with_ctx(
                envelope_bytes,
                PgContext::new()
                    .with_max_attempts(policy.max_attempts)
                    .with_priority(job_priority_for(&row.job_type))
                    .with_meta(meta),
            );
            task.parts.run_at = row.available_at.timestamp().max(0) as u64;
            tasks.push(task);
            relayed_row_ids.push(row.id);
            relayed += 1;
        }

        if !tasks.is_empty() {
            push_tasks(&mut *tx, self.config.clone(), tasks)
                .await
                .map_err(|e| AppError::Repository(Box::new(e)))?;
        }

        if !relayed_row_ids.is_empty() {
            let updated = sqlx::query!(
                "UPDATE job_outbox SET dispatched_at = now() WHERE id = ANY($1)",
                &relayed_row_ids,
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;
            if updated.rows_affected() != relayed_row_ids.len() as u64 {
                return Err(AppError::Domain(DomainError::InvariantViolation {
                    message: "outbox handoff dispatch batch updated an unexpected row count".into(),
                }));
            }
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(OutboxHandoffStats {
            claimed,
            relayed,
            deduped,
        })
    }
}

async fn has_active_apalis_dedupe(
    tx: &mut Transaction<'_, Postgres>,
    dedupe_key: &str,
    job_type: &str,
) -> Result<bool, AppError> {
    sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM apalis.jobs
            WHERE metadata->>'dedupe_key' = $1
              AND metadata->>'job_type' = $2
              AND (
                  status IN ('Pending', 'Queued', 'Running')
                  OR (status = 'Failed' AND attempts < max_attempts)
              )
        ) AS "exists!"
        "#,
        dedupe_key,
        job_type,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| AppError::Repository(Box::new(e)))
}

async fn defer_active_deduped_outbox_row(
    tx: &mut Transaction<'_, Postgres>,
    row_id: uuid::Uuid,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        UPDATE job_outbox
        SET available_at = now() + ($2::BIGINT * INTERVAL '1 second')
        WHERE id = $1
        "#,
        row_id,
        ACTIVE_DEDUPE_DEFER_SECONDS,
    )
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(|e| AppError::Repository(Box::new(e)))
}

#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: uuid::Uuid,
    job_type: String,
    payload: serde_json::Value,
    dedupe_key: Option<String>,
    available_at: DateTime<Utc>,
}

#[cfg(test)]
#[path = "outbox_handoff_tests.rs"]
mod tests;
