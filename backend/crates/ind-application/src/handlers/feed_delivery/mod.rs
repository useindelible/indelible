//! Feed delivery read + seen-state service for the document/feed/library model.
//!
//! Reads delegate to `FeedDeliveryRepository`, which queries
//! `feed_deliveries JOIN feed_source_entries LEFT JOIN documents LEFT JOIN library_entries`
//! so unprepared deliveries (`document_id = NULL`) still render and saved documents are
//! hidden. Seen/dismiss mutations only update `feed_deliveries` and never materialize a
//! document or enqueue jobs. Saving a delivery goes through the Library surface
//! (`POST /api/v1/library/from-delivery`), not this service. See
//! docs/document-feed-library-architecture.md (Query Surfaces -> Feed; User browses or opens
//! an external feed link).

use std::sync::Arc;

use futures::future::BoxFuture;

use ind_domain::{
    FeedDelivery, FeedDeliveryDisplay, FeedDeliveryId, FeedDeliveryState, FeedSubscriptionId,
    UserId,
};

use crate::error::AppError;
use crate::ports::FeedDeliveryOperations;
use crate::repos::feed_delivery::FeedDeliveryRepository;
use crate::repos::{Cursor, Page};

pub struct FeedDeliveryService {
    delivery_repo: Arc<dyn FeedDeliveryRepository>,
}

impl FeedDeliveryService {
    pub fn new(delivery_repo: Arc<dyn FeedDeliveryRepository>) -> Self {
        Self { delivery_repo }
    }

    pub async fn list(
        &self,
        user_id: UserId,
        state: FeedDeliveryState,
        subscription_id: Option<FeedSubscriptionId>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<FeedDeliveryDisplay>, AppError> {
        self.delivery_repo
            .list_deliveries(user_id, state, subscription_id, cursor, limit)
            .await
    }

    pub async fn get(
        &self,
        user_id: UserId,
        id: FeedDeliveryId,
    ) -> Result<Option<FeedDeliveryDisplay>, AppError> {
        self.delivery_repo.find_display_by_id(id, user_id).await
    }

    pub async fn mark_seen(
        &self,
        user_id: UserId,
        id: FeedDeliveryId,
    ) -> Result<FeedDelivery, AppError> {
        self.delivery_repo.mark_seen(id, user_id).await
    }

    pub async fn mark_all_seen(
        &self,
        user_id: UserId,
        subscription_id: Option<FeedSubscriptionId>,
    ) -> Result<u64, AppError> {
        self.delivery_repo
            .mark_all_seen(user_id, subscription_id)
            .await
    }

    pub async fn dismiss(
        &self,
        user_id: UserId,
        id: FeedDeliveryId,
    ) -> Result<FeedDelivery, AppError> {
        self.delivery_repo.dismiss(id, user_id).await
    }

    pub async fn count_unseen(&self, user_id: UserId) -> Result<i64, AppError> {
        self.delivery_repo.count_unseen(user_id).await
    }
}

/// The port delegates to the inherent async methods (which shadow the trait methods of the
/// same name), so `FeedDeliveryService` can be used directly as `Arc<dyn FeedDeliveryOperations>`
/// from both the API wiring and the test harness without a separate adapter.
impl FeedDeliveryOperations for FeedDeliveryService {
    fn list(
        &self,
        user_id: UserId,
        state: FeedDeliveryState,
        subscription_id: Option<FeedSubscriptionId>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> BoxFuture<'_, Result<Page<FeedDeliveryDisplay>, AppError>> {
        Box::pin(self.list(user_id, state, subscription_id, cursor, limit))
    }

    fn get(
        &self,
        user_id: UserId,
        id: FeedDeliveryId,
    ) -> BoxFuture<'_, Result<Option<FeedDeliveryDisplay>, AppError>> {
        Box::pin(self.get(user_id, id))
    }

    fn mark_seen(
        &self,
        user_id: UserId,
        id: FeedDeliveryId,
    ) -> BoxFuture<'_, Result<FeedDelivery, AppError>> {
        Box::pin(self.mark_seen(user_id, id))
    }

    fn mark_all_seen(
        &self,
        user_id: UserId,
        subscription_id: Option<FeedSubscriptionId>,
    ) -> BoxFuture<'_, Result<u64, AppError>> {
        Box::pin(self.mark_all_seen(user_id, subscription_id))
    }

    fn dismiss(
        &self,
        user_id: UserId,
        id: FeedDeliveryId,
    ) -> BoxFuture<'_, Result<FeedDelivery, AppError>> {
        Box::pin(self.dismiss(user_id, id))
    }

    fn count_unseen(&self, user_id: UserId) -> BoxFuture<'_, Result<i64, AppError>> {
        Box::pin(self.count_unseen(user_id))
    }
}
