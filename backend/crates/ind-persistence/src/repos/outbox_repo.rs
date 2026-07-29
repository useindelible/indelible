use chrono::{DateTime, Utc};
use sqlx::PgPool;

use ind_application::error::AppError;
use ind_application::repos::outbox::JobOutboxRepository;
use ind_domain::{JobOutbox, JobOutboxId};

pub struct PgJobOutboxRepository {
    pool: PgPool,
}

impl PgJobOutboxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl JobOutboxRepository for PgJobOutboxRepository {
    async fn enqueue(
        &self,
        job_type: &str,
        payload: serde_json::Value,
        dedupe_key: Option<String>,
        available_at: DateTime<Utc>,
    ) -> Result<JobOutbox, AppError> {
        let now = Utc::now();

        let row = match dedupe_key {
            Some(dedupe_key) => {
                let id = JobOutboxId::new();
                sqlx::query_as!(
                    JobOutboxRow,
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
                    RETURNING id, job_type, payload, dedupe_key, available_at, dispatched_at, created_at
                    "#,
                    id.as_uuid(),
                    job_type,
                    payload,
                    dedupe_key,
                    available_at,
                    now,
                )
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::Repository(Box::new(e)))?
            }
            None => {
                let id = JobOutboxId::new();
                sqlx::query_as!(
                    JobOutboxRow,
                    r#"
                    INSERT INTO job_outbox (id, job_type, payload, dedupe_key, available_at, created_at)
                    VALUES ($1, $2, $3, NULL, $4, $5)
                    RETURNING id, job_type, payload, dedupe_key, available_at, dispatched_at, created_at
                    "#,
                    id.as_uuid(),
                    job_type,
                    payload,
                    available_at,
                    now,
                )
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::Repository(Box::new(e)))?
            }
        };

        Ok(row.into_domain())
    }
}

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
