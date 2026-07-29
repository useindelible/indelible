use ind_application::AppError;
use sqlx::PgPool;

#[derive(Clone)]
pub struct TestWorkerHarness {
    pool: PgPool,
}

impl TestWorkerHarness {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn pending_job_count_by_type(&self, job_type: &str) -> Result<i64, AppError> {
        sqlx::query_scalar(
            "SELECT count(*) FROM job_outbox \
             WHERE dispatched_at IS NULL AND job_type = $1",
        )
        .bind(job_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))
    }
}
