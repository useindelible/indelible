use ind_application::AppError;
use ind_domain::{DomainError, FeedDelivery, FeedDeliveryId, FeedSubscriptionId, UserId};

use super::PgFeedDeliveryRepository;
use super::rows::{DeliveryRow, map_delivery_error};

fn not_found(id: FeedDeliveryId) -> AppError {
    AppError::Domain(DomainError::NotFound {
        entity: "FeedDelivery",
        id: id.to_string(),
    })
}

impl PgFeedDeliveryRepository {
    /// Mark a delivery seen. Idempotent: `seen_at` is set only if currently NULL so re-marking
    /// keeps the first seen timestamp. Only `feed_deliveries` is touched (AC #3).
    pub(super) async fn mark_seen_impl(
        &self,
        id: FeedDeliveryId,
        user_id: UserId,
    ) -> Result<FeedDelivery, AppError> {
        let row = sqlx::query_as!(
            DeliveryRow,
            "UPDATE feed_deliveries \
             SET seen_at = COALESCE(seen_at, now()), updated_at = now() \
             WHERE id = $1 AND user_id = $2 \
             RETURNING id, user_id, subscription_id, source_id, source_entry_id, document_id, \
                       delivered_at, seen_at, dismissed_at, hidden_at, created_at, updated_at",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_delivery_error)?;

        row.map(FeedDelivery::from).ok_or_else(|| not_found(id))
    }

    /// Mark all unseen, non-dismissed, non-hidden deliveries seen, optionally scoped to one
    /// subscription. Returns the number of rows transitioned to seen.
    pub(super) async fn mark_all_seen_impl(
        &self,
        user_id: UserId,
        subscription_id: Option<FeedSubscriptionId>,
    ) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "UPDATE feed_deliveries \
             SET seen_at = now(), updated_at = now() \
             WHERE user_id = $1 AND seen_at IS NULL AND dismissed_at IS NULL \
               AND hidden_at IS NULL AND ($2::uuid IS NULL OR subscription_id = $2)",
            user_id.into_uuid(),
            subscription_id.map(|id| id.into_uuid()),
        )
        .execute(&self.pool)
        .await
        .map_err(map_delivery_error)?;

        Ok(result.rows_affected())
    }

    /// Dismiss a delivery, removing it from both Feed lists. Only `feed_deliveries` is touched.
    pub(super) async fn dismiss_impl(
        &self,
        id: FeedDeliveryId,
        user_id: UserId,
    ) -> Result<FeedDelivery, AppError> {
        let row = sqlx::query_as!(
            DeliveryRow,
            "UPDATE feed_deliveries \
             SET dismissed_at = COALESCE(dismissed_at, now()), updated_at = now() \
             WHERE id = $1 AND user_id = $2 \
             RETURNING id, user_id, subscription_id, source_id, source_entry_id, document_id, \
                       delivered_at, seen_at, dismissed_at, hidden_at, created_at, updated_at",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_delivery_error)?;

        row.map(FeedDelivery::from).ok_or_else(|| not_found(id))
    }
}
