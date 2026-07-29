use super::*;

pub trait FeedOperations: Send + Sync {
    fn search_public_sources(
        &self,
        query: String,
        surface: FeedSearchSurface,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Vec<FeedSource>, AppError>>;

    fn subscribe(
        &self,
        user_id: UserId,
        url: String,
        title_override: Option<String>,
        poll_interval_override_minutes: Option<i32>,
    ) -> BoxFuture<'_, Result<FeedSubscribeResult, AppError>>;

    fn unsubscribe(
        &self,
        user_id: UserId,
        subscription_id: FeedSubscriptionId,
    ) -> BoxFuture<'_, Result<(), AppError>>;

    fn update_subscription(
        &self,
        user_id: UserId,
        subscription_id: FeedSubscriptionId,
        input: UpdateSubscriptionInput,
    ) -> BoxFuture<'_, Result<FeedSubscription, AppError>>;

    fn list_subscriptions(
        &self,
        user_id: UserId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<FeedSubscription>, AppError>>;

    fn import_opml(
        &self,
        user_id: UserId,
        opml_xml: String,
    ) -> BoxFuture<'_, Result<FeedOpmlImportResult, AppError>>;

    fn retry_subscription(
        &self,
        user_id: UserId,
        subscription_id: FeedSubscriptionId,
    ) -> BoxFuture<'_, Result<FeedSubscription, AppError>>;
}

pub struct FeedSubscribeResult {
    pub subscription: FeedSubscription,
    pub is_new: bool,
}

pub struct FeedOpmlImportResult {
    pub created: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}
