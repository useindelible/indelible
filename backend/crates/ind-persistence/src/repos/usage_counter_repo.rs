use chrono::{DateTime, Utc};
use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::usage_counter::{UsageCheck, UsageCounterRepository};
use ind_domain::{UsageCounterId, UserId};

pub struct PgUsageCounterRepository {
    pool: PgPool,
}

impl PgUsageCounterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct UsageCheckRow {
    quota_name: String,
    current_value: i64,
    limit_value: i64,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
}

#[async_trait::async_trait]
impl UsageCounterRepository for PgUsageCounterRepository {
    async fn increment_window_by(
        &self,
        user_id: UserId,
        quota_name: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit_value: i64,
        amount: i64,
    ) -> Result<UsageCheck, AppError> {
        let id = UsageCounterId::new();
        let row = sqlx::query_as!(
            UsageCheckRow,
            r#"
            INSERT INTO usage_counters (
                id,
                user_id,
                quota_name,
                period_start,
                period_end,
                current_value,
                limit_value
            )
            VALUES ($1, $2, $3, $4, $5, $7, $6)
            ON CONFLICT (user_id, quota_name, period_start) DO UPDATE SET
                current_value = usage_counters.current_value + $7,
                period_end = EXCLUDED.period_end,
                limit_value = EXCLUDED.limit_value
            RETURNING
                quota_name,
                current_value,
                limit_value,
                period_start,
                period_end
            "#,
            id.as_uuid(),
            user_id.into_uuid(),
            quota_name,
            period_start,
            period_end,
            limit_value,
            amount,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(UsageCheck {
            quota_name: row.quota_name,
            current_value: row.current_value,
            limit_value: row.limit_value,
            period_start: row.period_start,
            period_end: row.period_end,
        })
    }

    async fn try_increment_window_by(
        &self,
        user_id: UserId,
        quota_name: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit_value: i64,
        amount: i64,
    ) -> Result<Option<UsageCheck>, AppError> {
        let id = UsageCounterId::new();
        let row = sqlx::query_as!(
            UsageCheckRow,
            r#"
            INSERT INTO usage_counters (
                id,
                user_id,
                quota_name,
                period_start,
                period_end,
                current_value,
                limit_value
            )
            SELECT $1, $2, $3, $4, $5, $7::bigint, $6::bigint
            WHERE $7::bigint <= $6::bigint
            ON CONFLICT (user_id, quota_name, period_start) DO UPDATE SET
                current_value = usage_counters.current_value + EXCLUDED.current_value,
                period_end = EXCLUDED.period_end,
                limit_value = EXCLUDED.limit_value
            WHERE usage_counters.current_value + EXCLUDED.current_value <= EXCLUDED.limit_value
            RETURNING
                quota_name,
                current_value,
                limit_value,
                period_start,
                period_end
            "#,
            id.as_uuid(),
            user_id.into_uuid(),
            quota_name,
            period_start,
            period_end,
            limit_value,
            amount,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(row.map(|row| UsageCheck {
            quota_name: row.quota_name,
            current_value: row.current_value,
            limit_value: row.limit_value,
            period_start: row.period_start,
            period_end: row.period_end,
        }))
    }
}
