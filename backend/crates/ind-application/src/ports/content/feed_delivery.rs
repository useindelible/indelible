use super::*;
use crate::repos::Cursor;
use ind_domain::{
    FeedDelivery, FeedDeliveryDisplay, FeedDeliveryId, FeedDeliveryState, FeedSubscriptionId,
};

/// HTTP-facing port for the document-model Feed surface. Reads go through
/// `feed_deliveries JOIN feed_source_entries LEFT JOIN documents LEFT JOIN library_entries`;
/// seen/dismiss mutations only touch `feed_deliveries` and never materialize documents or
/// enqueue jobs.
pub trait FeedDeliveryOperations: Send + Sync {
    fn list(
        &self,
        user_id: UserId,
        state: FeedDeliveryState,
        subscription_id: Option<FeedSubscriptionId>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> BoxFuture<'_, Result<Page<FeedDeliveryDisplay>, AppError>>;

    fn get(
        &self,
        user_id: UserId,
        id: FeedDeliveryId,
    ) -> BoxFuture<'_, Result<Option<FeedDeliveryDisplay>, AppError>>;

    fn mark_seen(
        &self,
        user_id: UserId,
        id: FeedDeliveryId,
    ) -> BoxFuture<'_, Result<FeedDelivery, AppError>>;

    fn mark_all_seen(
        &self,
        user_id: UserId,
        subscription_id: Option<FeedSubscriptionId>,
    ) -> BoxFuture<'_, Result<u64, AppError>>;

    fn dismiss(
        &self,
        user_id: UserId,
        id: FeedDeliveryId,
    ) -> BoxFuture<'_, Result<FeedDelivery, AppError>>;

    fn count_unseen(&self, user_id: UserId) -> BoxFuture<'_, Result<i64, AppError>>;
}
