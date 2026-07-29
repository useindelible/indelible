mod entries;
mod providers;
mod sources;
mod subscription_mutations;
mod subscription_rows;
mod subscriptions;
mod types;

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::feed::FeedRepository;
use ind_application::repos::{Cursor, Page};
use ind_domain::{
    ActiveSubscription, CollectionId, FeedProviderInstance, FeedSearchSurface, FeedSource,
    FeedSourceEntry, FeedSourceEntryId, FeedSourceId, FeedStatus, FeedSubscription,
    FeedSubscriptionId, PollOutcome, UserId,
};

pub struct PgFeedRepository {
    pool: PgPool,
}

impl PgFeedRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FeedRepository for PgFeedRepository {
    async fn find_source_by_id(&self, id: FeedSourceId) -> Result<Option<FeedSource>, AppError> {
        self.find_source_by_id_impl(id).await
    }

    async fn find_source_by_canonical_key(
        &self,
        canonical_key: &str,
    ) -> Result<Option<FeedSource>, AppError> {
        self.find_source_by_canonical_key_impl(canonical_key).await
    }

    async fn create_source(&self, source: FeedSource) -> Result<FeedSource, AppError> {
        self.create_source_impl(source).await
    }

    async fn update_source_details(
        &self,
        id: FeedSourceId,
        details: ind_domain::SourceDetailsUpdate,
    ) -> Result<FeedSource, AppError> {
        self.update_source_details_impl(id, details).await
    }

    async fn bump_source_popularity(
        &self,
        id: FeedSourceId,
        delta: i32,
    ) -> Result<FeedSource, AppError> {
        self.bump_source_popularity_impl(id, delta).await
    }

    async fn mark_source_poll_requested(
        &self,
        id: FeedSourceId,
        next_poll_at: DateTime<Utc>,
    ) -> Result<FeedSource, AppError> {
        self.mark_source_poll_requested_impl(id, next_poll_at).await
    }

    async fn mark_source_poll_success(
        &self,
        id: FeedSourceId,
        state: PollOutcome,
        last_entry_added_at: Option<DateTime<Utc>>,
    ) -> Result<FeedSource, AppError> {
        self.mark_source_poll_success_impl(id, state, last_entry_added_at)
            .await
    }

    async fn mark_source_poll_failure(
        &self,
        id: FeedSourceId,
        next_poll_at: DateTime<Utc>,
        error: String,
        consecutive_failures: i32,
    ) -> Result<FeedSource, AppError> {
        self.mark_source_poll_failure_impl(id, next_poll_at, error, consecutive_failures)
            .await
    }

    async fn clear_source_lease(&self, id: FeedSourceId) -> Result<(), AppError> {
        self.clear_source_lease_impl(id).await
    }

    async fn claim_due_sources(
        &self,
        now: DateTime<Utc>,
        worker_id: &str,
        limit: i64,
        lease_duration: Duration,
    ) -> Result<Vec<FeedSource>, AppError> {
        self.claim_due_sources_impl(now, worker_id, limit, lease_duration)
            .await
    }

    async fn search_public_sources(
        &self,
        query: &str,
        surface: FeedSearchSurface,
        limit: u32,
    ) -> Result<Vec<FeedSource>, AppError> {
        self.search_public_sources_impl(query, surface, limit).await
    }

    async fn find_subscription_by_id(
        &self,
        id: FeedSubscriptionId,
    ) -> Result<Option<FeedSubscription>, AppError> {
        self.find_subscription_by_id_impl(id).await
    }

    async fn find_subscription_by_user_and_source(
        &self,
        user_id: UserId,
        source_id: FeedSourceId,
    ) -> Result<Option<FeedSubscription>, AppError> {
        self.find_subscription_by_user_and_source_impl(user_id, source_id)
            .await
    }

    async fn create_subscription(
        &self,
        subscription: FeedSubscription,
    ) -> Result<FeedSubscription, AppError> {
        self.create_subscription_impl(subscription).await
    }

    async fn delete_subscription(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
    ) -> Result<FeedSourceId, AppError> {
        self.delete_subscription_impl(id, user_id).await
    }

    async fn delete_source_if_orphaned(&self, id: FeedSourceId) -> Result<(), AppError> {
        self.delete_source_if_orphaned_impl(id).await
    }

    async fn list_subscriptions_by_user(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<FeedSubscription>, AppError> {
        self.list_subscriptions_by_user_impl(user_id, cursor, limit)
            .await
    }

    async fn list_active_subscriptions_for_source(
        &self,
        source_id: FeedSourceId,
    ) -> Result<Vec<ActiveSubscription>, AppError> {
        self.list_active_subscriptions_for_source_impl(source_id)
            .await
    }

    async fn set_subscription_title_override(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
        title_override: Option<String>,
    ) -> Result<FeedSubscription, AppError> {
        self.set_subscription_title_override_impl(id, user_id, title_override)
            .await
    }

    async fn set_subscription_auto_save(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
        auto_save: bool,
        collection_id: Option<Option<CollectionId>>,
    ) -> Result<FeedSubscription, AppError> {
        self.set_subscription_auto_save_impl(id, user_id, auto_save, collection_id)
            .await
    }

    async fn set_subscription_poll_interval(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
        poll_interval_override_minutes: Option<i32>,
    ) -> Result<FeedSubscription, AppError> {
        self.set_subscription_poll_interval_impl(id, user_id, poll_interval_override_minutes)
            .await
    }

    async fn set_subscription_status(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
        status: FeedStatus,
    ) -> Result<FeedSubscription, AppError> {
        self.set_subscription_status_impl(id, user_id, status).await
    }

    async fn find_source_entry_by_source_guid(
        &self,
        source_id: FeedSourceId,
        guid: &str,
    ) -> Result<Option<FeedSourceEntry>, AppError> {
        self.find_source_entry_by_source_guid_impl(source_id, guid)
            .await
    }

    async fn find_source_entry_by_id(
        &self,
        id: FeedSourceEntryId,
    ) -> Result<Option<FeedSourceEntry>, AppError> {
        self.find_source_entry_by_id_impl(id).await
    }

    async fn create_source_entry(
        &self,
        entry: FeedSourceEntry,
    ) -> Result<FeedSourceEntry, AppError> {
        self.create_source_entry_impl(entry).await
    }

    async fn create_or_adopt_polled_source_entry(
        &self,
        entry: FeedSourceEntry,
    ) -> Result<FeedSourceEntry, AppError> {
        self.create_or_adopt_polled_source_entry_impl(entry).await
    }

    async fn set_source_entry_canonical_url(
        &self,
        entry_id: FeedSourceEntryId,
        canonical_url: &str,
    ) -> Result<(), AppError> {
        self.set_source_entry_canonical_url_impl(entry_id, canonical_url)
            .await
    }

    async fn set_source_entry_language_if_missing(
        &self,
        entry_id: FeedSourceEntryId,
        language: &str,
    ) -> Result<bool, AppError> {
        self.set_source_entry_language_if_missing_impl(entry_id, language)
            .await
    }

    async fn source_entries_missing_canonical_url_after(
        &self,
        after_id: uuid::Uuid,
        limit: i64,
    ) -> Result<Vec<(FeedSourceEntryId, String)>, AppError> {
        self.source_entries_missing_canonical_url_after_impl(after_id, limit)
            .await
    }

    async fn list_provider_instances(
        &self,
        provider_type: &str,
    ) -> Result<Vec<FeedProviderInstance>, AppError> {
        self.list_provider_instances_impl(provider_type).await
    }

    async fn list_all_enabled_provider_instances(
        &self,
    ) -> Result<Vec<FeedProviderInstance>, AppError> {
        self.list_all_enabled_provider_instances_impl().await
    }

    async fn record_provider_instance_success(&self, id: uuid::Uuid) -> Result<(), AppError> {
        self.record_provider_instance_success_impl(id).await
    }

    async fn record_provider_instance_failure(&self, id: uuid::Uuid) -> Result<(), AppError> {
        self.record_provider_instance_failure_impl(id).await
    }
}
