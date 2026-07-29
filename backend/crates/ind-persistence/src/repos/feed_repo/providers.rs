use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_domain::FeedProviderInstance;
use uuid::Uuid;

use super::PgFeedRepository;
use super::types::*;

struct ProviderInstanceRow {
    id: Uuid,
    provider_type: String,
    base_url: String,
    priority: i32,
    enabled: bool,
    last_success_at: Option<DateTime<Utc>>,
    last_failure_at: Option<DateTime<Utc>>,
    consecutive_failures: i32,
}

impl From<ProviderInstanceRow> for FeedProviderInstance {
    fn from(row: ProviderInstanceRow) -> Self {
        FeedProviderInstance {
            id: row.id,
            provider_type: row.provider_type,
            base_url: row.base_url,
            priority: row.priority,
            enabled: row.enabled,
            last_success_at: row.last_success_at,
            last_failure_at: row.last_failure_at,
            consecutive_failures: row.consecutive_failures,
        }
    }
}

impl PgFeedRepository {
    pub(super) async fn list_provider_instances_impl(
        &self,
        provider_type: &str,
    ) -> Result<Vec<FeedProviderInstance>, AppError> {
        let rows = sqlx::query_as!(
            ProviderInstanceRow,
            "SELECT id, provider_type, base_url, priority, enabled, \
                    last_success_at, last_failure_at, consecutive_failures \
             FROM feed_provider_instances \
             WHERE provider_type = $1 AND enabled = true \
             ORDER BY priority ASC, consecutive_failures ASC",
            provider_type,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_provider_error)?;

        Ok(rows.into_iter().map(FeedProviderInstance::from).collect())
    }

    pub(super) async fn list_all_enabled_provider_instances_impl(
        &self,
    ) -> Result<Vec<FeedProviderInstance>, AppError> {
        let rows = sqlx::query_as!(
            ProviderInstanceRow,
            "SELECT id, provider_type, base_url, priority, enabled, \
                    last_success_at, last_failure_at, consecutive_failures \
             FROM feed_provider_instances \
             WHERE enabled = true \
             ORDER BY priority ASC, consecutive_failures ASC, provider_type ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_provider_error)?;

        Ok(rows.into_iter().map(FeedProviderInstance::from).collect())
    }

    pub(super) async fn record_provider_instance_success_impl(
        &self,
        id: uuid::Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE feed_provider_instances \
             SET last_success_at = now(), consecutive_failures = 0, updated_at = now() \
             WHERE id = $1",
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(map_provider_error)?;

        Ok(())
    }

    pub(super) async fn record_provider_instance_failure_impl(
        &self,
        id: uuid::Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE feed_provider_instances \
             SET last_failure_at = now(), consecutive_failures = consecutive_failures + 1, \
                 updated_at = now() \
             WHERE id = $1",
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(map_provider_error)?;

        Ok(())
    }
}
