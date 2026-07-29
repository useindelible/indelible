use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::maintenance::{MaintenanceTaskLease, MaintenanceTaskRepository};
use ind_domain::DomainError;

pub struct PgMaintenanceTaskRepository {
    pool: PgPool,
}

impl PgMaintenanceTaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn repo_error(error: sqlx::Error) -> AppError {
    AppError::Repository(Box::new(error))
}

fn lost_lease(task_name: &str) -> AppError {
    AppError::Domain(DomainError::InvariantViolation {
        message: format!("maintenance task `{task_name}` lease is no longer owned"),
    })
}

#[async_trait]
impl MaintenanceTaskRepository for PgMaintenanceTaskRepository {
    async fn try_acquire(
        &self,
        task_name: &str,
        lease_owner: &str,
        now: DateTime<Utc>,
        lease_expires_at: DateTime<Utc>,
    ) -> Result<Option<MaintenanceTaskLease>, AppError> {
        let mut tx = self.pool.begin().await.map_err(repo_error)?;
        sqlx::query!(
            r#"
            INSERT INTO maintenance_tasks (task_name, next_run_at, updated_at)
            VALUES ($1, $2, $2)
            ON CONFLICT (task_name) DO NOTHING
            "#,
            task_name,
            now,
        )
        .execute(&mut *tx)
        .await
        .map_err(repo_error)?;

        let lease = sqlx::query_as!(
            MaintenanceTaskLease,
            r#"
            UPDATE maintenance_tasks
            SET lease_owner = $2,
                lease_expires_at = $4,
                last_started_at = $3,
                updated_at = $3
            WHERE task_name = $1
              AND next_run_at <= $3
              AND (lease_expires_at IS NULL OR lease_expires_at <= $3)
            RETURNING task_name, continuation_cursor
            "#,
            task_name,
            lease_owner,
            now,
            lease_expires_at,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(repo_error)?;
        tx.commit().await.map_err(repo_error)?;
        Ok(lease)
    }

    async fn complete(
        &self,
        task_name: &str,
        lease_owner: &str,
        next_run_at: DateTime<Utc>,
        continuation_cursor: Option<&str>,
        completed_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE maintenance_tasks
            SET next_run_at = $3,
                continuation_cursor = $4,
                lease_owner = NULL,
                lease_expires_at = NULL,
                last_completed_at = $5,
                last_error = NULL,
                updated_at = $5
            WHERE task_name = $1 AND lease_owner = $2
            "#,
            task_name,
            lease_owner,
            next_run_at,
            continuation_cursor,
            completed_at,
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;
        if result.rows_affected() != 1 {
            return Err(lost_lease(task_name));
        }
        Ok(())
    }

    async fn fail(
        &self,
        task_name: &str,
        lease_owner: &str,
        next_run_at: DateTime<Utc>,
        error: &str,
        failed_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE maintenance_tasks
            SET next_run_at = $3,
                continuation_cursor = NULL,
                lease_owner = NULL,
                lease_expires_at = NULL,
                last_error = $4,
                updated_at = $5
            WHERE task_name = $1 AND lease_owner = $2
            "#,
            task_name,
            lease_owner,
            next_run_at,
            error,
            failed_at,
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;
        if result.rows_affected() != 1 {
            return Err(lost_lease(task_name));
        }
        Ok(())
    }
}
