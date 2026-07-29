use chrono::{DateTime, Utc};

use crate::error::AppError;
use ind_domain::UserId;

#[derive(Debug, Clone)]
pub struct UsageCheck {
    pub quota_name: String,
    pub current_value: i64,
    pub limit_value: i64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait UsageCounterRepository: Send + Sync {
    /// Atomically increment a rolling window counter by an arbitrary amount.
    /// Amount-based accounting is used for TTS quotas (characters, seconds, cost units).
    async fn increment_window_by(
        &self,
        user_id: UserId,
        quota_name: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit_value: i64,
        amount: i64,
    ) -> Result<UsageCheck, AppError>;

    /// Atomically reserve quota when the resulting value stays within the
    /// configured limit. Returns `Ok(None)` when the reservation would exceed
    /// the limit and no counter mutation should be committed.
    async fn try_increment_window_by(
        &self,
        user_id: UserId,
        quota_name: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit_value: i64,
        amount: i64,
    ) -> Result<Option<UsageCheck>, AppError> {
        let usage = self
            .increment_window_by(
                user_id,
                quota_name,
                period_start,
                period_end,
                limit_value,
                amount,
            )
            .await?;
        Ok((usage.current_value <= usage.limit_value).then_some(usage))
    }

    /// Convenience wrapper that increments by 1.
    async fn increment_window(
        &self,
        user_id: UserId,
        quota_name: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit_value: i64,
    ) -> Result<UsageCheck, AppError> {
        self.increment_window_by(
            user_id,
            quota_name,
            period_start,
            period_end,
            limit_value,
            1,
        )
        .await
    }
}
