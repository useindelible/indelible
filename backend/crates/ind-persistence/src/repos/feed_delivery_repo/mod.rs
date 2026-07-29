mod mutations;
mod reads;
mod rows;
pub(crate) mod tx_writes;

use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::feed_delivery::{FeedDeliveryRepository, FeedDeliveryUpsert};
use ind_application::repos::{Cursor, Page};
use ind_domain::{
    FeedAutosaveJob, FeedDelivery, FeedDeliveryDisplay, FeedDeliveryId, FeedDeliveryState,
    FeedSubscriptionId, UserId,
};

pub struct PgFeedDeliveryRepository {
    pool: PgPool,
}

impl PgFeedDeliveryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FeedDeliveryRepository for PgFeedDeliveryRepository {
    async fn upsert_delivery(
        &self,
        delivery: FeedDelivery,
    ) -> Result<FeedDeliveryUpsert, AppError> {
        self.upsert_delivery_impl(delivery).await
    }

    async fn upsert_delivery_with_autosave(
        &self,
        delivery: FeedDelivery,
        autosave: Option<FeedAutosaveJob>,
        available_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<FeedDeliveryUpsert, AppError> {
        self.upsert_delivery_with_autosave_impl(delivery, autosave, available_at)
            .await
    }

    async fn find_by_id(
        &self,
        id: FeedDeliveryId,
        user_id: UserId,
    ) -> Result<Option<FeedDelivery>, AppError> {
        self.find_by_id_impl(id, user_id).await
    }

    async fn list_deliveries(
        &self,
        user_id: UserId,
        state: FeedDeliveryState,
        subscription_id: Option<FeedSubscriptionId>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<FeedDeliveryDisplay>, AppError> {
        self.list_deliveries_impl(user_id, state, subscription_id, cursor, limit)
            .await
    }

    async fn find_display_by_id(
        &self,
        id: FeedDeliveryId,
        user_id: UserId,
    ) -> Result<Option<FeedDeliveryDisplay>, AppError> {
        self.find_display_by_id_impl(id, user_id).await
    }

    async fn mark_seen(
        &self,
        id: FeedDeliveryId,
        user_id: UserId,
    ) -> Result<FeedDelivery, AppError> {
        self.mark_seen_impl(id, user_id).await
    }

    async fn mark_all_seen(
        &self,
        user_id: UserId,
        subscription_id: Option<FeedSubscriptionId>,
    ) -> Result<u64, AppError> {
        self.mark_all_seen_impl(user_id, subscription_id).await
    }

    async fn dismiss(&self, id: FeedDeliveryId, user_id: UserId) -> Result<FeedDelivery, AppError> {
        self.dismiss_impl(id, user_id).await
    }

    async fn count_unseen(&self, user_id: UserId) -> Result<i64, AppError> {
        self.count_unseen_impl(user_id).await
    }

    async fn list_prefetch_candidates(
        &self,
        user_id: UserId,
        subscription_id: Option<FeedSubscriptionId>,
        active_within_days: i64,
        limit: u32,
    ) -> Result<Vec<FeedDelivery>, AppError> {
        self.list_prefetch_candidates_impl(user_id, subscription_id, active_within_days, limit)
            .await
    }
}
