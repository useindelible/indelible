use chrono::{DateTime, Duration, Utc};

use crate::error::AppError;
use crate::repos::{Cursor, Page};
use ind_domain::{
    ActiveSubscription, CollectionId, FeedProviderInstance, FeedSearchSurface, FeedSource,
    FeedSourceEntry, FeedSourceEntryId, FeedSourceId, FeedStatus, FeedSubscription,
    FeedSubscriptionId, PollOutcome, SourceDetailsUpdate, UserId,
};

#[async_trait::async_trait]
pub trait FeedRepository: Send + Sync {
    async fn find_source_by_id(&self, id: FeedSourceId) -> Result<Option<FeedSource>, AppError>;

    async fn find_source_by_canonical_key(
        &self,
        canonical_key: &str,
    ) -> Result<Option<FeedSource>, AppError>;

    async fn create_source(&self, source: FeedSource) -> Result<FeedSource, AppError>;

    async fn update_source_details(
        &self,
        id: FeedSourceId,
        details: SourceDetailsUpdate,
    ) -> Result<FeedSource, AppError>;

    async fn bump_source_popularity(
        &self,
        id: FeedSourceId,
        delta: i32,
    ) -> Result<FeedSource, AppError>;

    async fn mark_source_poll_requested(
        &self,
        id: FeedSourceId,
        next_poll_at: DateTime<Utc>,
    ) -> Result<FeedSource, AppError>;

    async fn mark_source_poll_success(
        &self,
        id: FeedSourceId,
        state: PollOutcome,
        last_entry_added_at: Option<DateTime<Utc>>,
    ) -> Result<FeedSource, AppError>;

    async fn mark_source_poll_failure(
        &self,
        id: FeedSourceId,
        next_poll_at: DateTime<Utc>,
        error: String,
        consecutive_failures: i32,
    ) -> Result<FeedSource, AppError>;

    async fn clear_source_lease(&self, id: FeedSourceId) -> Result<(), AppError>;

    async fn claim_due_sources(
        &self,
        now: DateTime<Utc>,
        worker_id: &str,
        limit: i64,
        lease_duration: Duration,
    ) -> Result<Vec<FeedSource>, AppError>;

    async fn search_public_sources(
        &self,
        query: &str,
        surface: FeedSearchSurface,
        limit: u32,
    ) -> Result<Vec<FeedSource>, AppError>;

    async fn find_subscription_by_id(
        &self,
        id: FeedSubscriptionId,
    ) -> Result<Option<FeedSubscription>, AppError>;

    async fn find_subscription_by_user_and_source(
        &self,
        user_id: UserId,
        source_id: FeedSourceId,
    ) -> Result<Option<FeedSubscription>, AppError>;

    async fn create_subscription(
        &self,
        subscription: FeedSubscription,
    ) -> Result<FeedSubscription, AppError>;

    async fn delete_subscription(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
    ) -> Result<FeedSourceId, AppError>;

    async fn delete_source_if_orphaned(&self, id: FeedSourceId) -> Result<(), AppError>;

    async fn list_subscriptions_by_user(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<FeedSubscription>, AppError>;

    async fn list_active_subscriptions_for_source(
        &self,
        source_id: FeedSourceId,
    ) -> Result<Vec<ActiveSubscription>, AppError>;

    async fn set_subscription_title_override(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
        title_override: Option<String>,
    ) -> Result<FeedSubscription, AppError>;

    async fn set_subscription_auto_save(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
        auto_save: bool,
        collection_id: Option<Option<CollectionId>>,
    ) -> Result<FeedSubscription, AppError>;

    async fn set_subscription_poll_interval(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
        poll_interval_override_minutes: Option<i32>,
    ) -> Result<FeedSubscription, AppError>;

    async fn set_subscription_status(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
        status: FeedStatus,
    ) -> Result<FeedSubscription, AppError>;

    async fn find_source_entry_by_source_guid(
        &self,
        source_id: FeedSourceId,
        guid: &str,
    ) -> Result<Option<FeedSourceEntry>, AppError>;

    /// Look up a single source entry by id. Needed by the save-from-delivery flow to read the
    /// entry's `canonical_url`/metadata when materializing the document.
    async fn find_source_entry_by_id(
        &self,
        id: FeedSourceEntryId,
    ) -> Result<Option<FeedSourceEntry>, AppError>;

    async fn create_source_entry(
        &self,
        entry: FeedSourceEntry,
    ) -> Result<FeedSourceEntry, AppError>;

    /// Insert a newly polled entry or adopt the most recent semantically identical fallback
    /// entry created by the legacy timestamp-sensitive GUID algorithm.
    async fn create_or_adopt_polled_source_entry(
        &self,
        entry: FeedSourceEntry,
    ) -> Result<FeedSourceEntry, AppError>;

    /// Targeted update of a single entry's `canonical_url` (TASK-239). Used by the poll-reuse
    /// recompute and the one-off backfill; never a full-row write.
    async fn set_source_entry_canonical_url(
        &self,
        entry_id: FeedSourceEntryId,
        canonical_url: &str,
    ) -> Result<(), AppError>;

    /// Fill a missing entry language and rebuild its generated search projection without
    /// replacing language metadata already stored for the shared source entry.
    async fn set_source_entry_language_if_missing(
        &self,
        entry_id: FeedSourceEntryId,
        language: &str,
    ) -> Result<bool, AppError>;

    /// Keyset page of `(id, url)` for entries with `url IS NOT NULL AND canonical_url IS NULL`,
    /// ordered by `id` and starting after `after_id`, for the one-off canonical_url backfill
    /// (TASK-239). Keyset paging guarantees the backfill terminates even when some urls never
    /// canonicalize.
    async fn source_entries_missing_canonical_url_after(
        &self,
        after_id: uuid::Uuid,
        limit: i64,
    ) -> Result<Vec<(FeedSourceEntryId, String)>, AppError>;

    async fn list_provider_instances(
        &self,
        provider_type: &str,
    ) -> Result<Vec<FeedProviderInstance>, AppError>;

    async fn list_all_enabled_provider_instances(
        &self,
    ) -> Result<Vec<FeedProviderInstance>, AppError>;

    async fn record_provider_instance_success(&self, id: uuid::Uuid) -> Result<(), AppError>;

    async fn record_provider_instance_failure(&self, id: uuid::Uuid) -> Result<(), AppError>;
}
