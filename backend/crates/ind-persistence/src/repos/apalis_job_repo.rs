use std::time::Duration;

use ind_application::AppError;
use ind_application::repos::apalis_job::ApalisJobRepository;
use sqlx::PgPool;

pub struct PgApalisJobRepository {
    pool: PgPool,
}

impl PgApalisJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ApalisJobRepository for PgApalisJobRepository {
    async fn reschedule_locked_job(
        &self,
        task_id: &str,
        lock_by: &str,
        delay: Duration,
    ) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "UPDATE apalis.jobs \
             SET run_at = now() + make_interval(secs => $1::double precision) \
             WHERE id = $2 AND lock_by = $3",
            delay.as_secs_f64(),
            task_id,
            lock_by,
        )
        .execute(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(result.rows_affected())
    }
}
