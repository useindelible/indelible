use futures::future::BoxFuture;

use crate::error::AppError;
use crate::ports::content::{FeedOperations, FeedOpmlImportResult, FeedSubscribeResult};
use crate::repos::{Cursor, Page};
use ind_domain::{FeedSearchSurface, FeedSource, FeedSubscription, FeedSubscriptionId, UserId};

use super::{FeedService, SubscribeInput, UpdateSubscriptionInput};

impl FeedOperations for FeedService {
    fn search_public_sources(
        &self,
        query: String,
        surface: FeedSearchSurface,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Vec<FeedSource>, AppError>> {
        Box::pin(async move {
            self.search_public_sources(&query, surface, limit.unwrap_or(20))
                .await
        })
    }

    fn subscribe(
        &self,
        user_id: UserId,
        url: String,
        title_override: Option<String>,
        poll_interval_override_minutes: Option<i32>,
    ) -> BoxFuture<'_, Result<FeedSubscribeResult, AppError>> {
        Box::pin(async move {
            let result = self
                .subscribe(
                    user_id,
                    SubscribeInput {
                        url,
                        title_override,
                        poll_interval_override_minutes,
                    },
                )
                .await?;
            Ok(FeedSubscribeResult {
                subscription: result.subscription,
                is_new: result.is_new,
            })
        })
    }

    fn unsubscribe(
        &self,
        user_id: UserId,
        subscription_id: FeedSubscriptionId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.unsubscribe(user_id, subscription_id))
    }

    fn update_subscription(
        &self,
        user_id: UserId,
        subscription_id: FeedSubscriptionId,
        input: UpdateSubscriptionInput,
    ) -> BoxFuture<'_, Result<FeedSubscription, AppError>> {
        Box::pin(self.update_subscription(user_id, subscription_id, input))
    }

    fn list_subscriptions(
        &self,
        user_id: UserId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<FeedSubscription>, AppError>> {
        Box::pin(async move {
            self.list_subscriptions(user_id, cursor.map(Cursor), limit.unwrap_or(50))
                .await
        })
    }

    fn import_opml(
        &self,
        user_id: UserId,
        opml_xml: String,
    ) -> BoxFuture<'_, Result<FeedOpmlImportResult, AppError>> {
        Box::pin(async move {
            let result = self.import_opml(user_id, &opml_xml).await?;
            Ok(FeedOpmlImportResult {
                created: result.created,
                skipped: result.skipped,
                errors: result.errors,
            })
        })
    }

    fn retry_subscription(
        &self,
        user_id: UserId,
        subscription_id: FeedSubscriptionId,
    ) -> BoxFuture<'_, Result<FeedSubscription, AppError>> {
        Box::pin(self.retry_subscription(user_id, subscription_id))
    }
}
