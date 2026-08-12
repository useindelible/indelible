use crate::error::AppError;
use crate::repos::{Cursor, Page};
use chrono::{DateTime, Utc};
use ind_domain::{
    FeedAutosaveJob, FeedDelivery, FeedDeliveryDisplay, FeedDeliveryId, FeedDeliveryState,
    FeedSubscriptionId, UserId,
};

#[derive(Debug)]
pub struct FeedDeliveryUpsert {
    pub delivery: FeedDelivery,
    pub newly_inserted: bool,
}

/// Repository for user-visible feed deliveries.
/// See docs/document-feed-library-architecture.md (feed_deliveries; Query Surfaces -> Feed).
///
/// Discovery queries are delivery-keyed and treat `document_id` as optional: unlinked
/// deliveries render from `feed_source_entries` (AC #1) and linked deliveries carry a
/// document overlay (AC #2) without dropping the delivery row. Saved documents are excluded
/// from the Feed lists by joining `library_entries` (AC #5). Mark-seen/mark-all-seen/dismiss
/// only mutate `feed_deliveries` state and never materialize documents or enqueue jobs
/// (AC #3/#4).
#[async_trait::async_trait]
pub trait FeedDeliveryRepository: Send + Sync {
    /// Discovery upsert keyed by `(user_id, subscription_id, source_entry_id)`.
    async fn upsert_delivery(&self, delivery: FeedDelivery)
    -> Result<FeedDeliveryUpsert, AppError>;

    /// Discovery upsert plus optional auto-save enqueue in one repository-managed transaction.
    /// The auto-save job is enqueued only when the delivery row is newly inserted.
    async fn upsert_delivery_with_autosave(
        &self,
        delivery: FeedDelivery,
        autosave: Option<FeedAutosaveJob>,
        available_at: DateTime<Utc>,
    ) -> Result<FeedDeliveryUpsert, AppError>;

    async fn find_by_id(
        &self,
        id: FeedDeliveryId,
        user_id: UserId,
    ) -> Result<Option<FeedDelivery>, AppError>;

    /// Unseen/Seen Feed list. Excludes dismissed, hidden, and saved (active `library_entries`)
    /// deliveries. Unseen orders by source-entry publication time, falling back to
    /// `delivered_at`; Seen orders by `seen_at DESC`.
    async fn list_deliveries(
        &self,
        user_id: UserId,
        state: FeedDeliveryState,
        subscription_id: Option<FeedSubscriptionId>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<FeedDeliveryDisplay>, AppError>;

    /// Single delivery with its document overlay and `saved` flag. Unlike the list reads, this
    /// does not exclude saved/seen deliveries, so it can report the true `saved` state.
    async fn find_display_by_id(
        &self,
        id: FeedDeliveryId,
        user_id: UserId,
    ) -> Result<Option<FeedDeliveryDisplay>, AppError>;

    /// Mark a delivery seen (`seen_at = COALESCE(seen_at, now())`). NotFound when the delivery
    /// does not exist for the user.
    async fn mark_seen(
        &self,
        id: FeedDeliveryId,
        user_id: UserId,
    ) -> Result<FeedDelivery, AppError>;

    /// Mark all unseen deliveries seen, optionally scoped to one subscription. Returns the count
    /// of rows transitioned from unseen to seen.
    async fn mark_all_seen(
        &self,
        user_id: UserId,
        subscription_id: Option<FeedSubscriptionId>,
    ) -> Result<u64, AppError>;

    /// Dismiss a delivery (`dismissed_at = now()`); removes it from both Feed lists.
    async fn dismiss(&self, id: FeedDeliveryId, user_id: UserId) -> Result<FeedDelivery, AppError>;

    /// Count of unseen, non-dismissed, non-hidden, non-saved deliveries for the Feed badge.
    async fn count_unseen(&self, user_id: UserId) -> Result<i64, AppError>;

    /// Newest eligible deliveries for read-ahead preparation, ordered by source-entry
    /// publication time with `delivered_at` as the fallback.
    /// Eligibility (docs/document-feed-library-architecture.md, Readable Content Preparation
    /// Policy): subscription `status='active'`; unread, not hidden/dismissed; the source entry
    /// has a `canonical_url` (URL-backed); the linked/materialized document has no `completed`
    /// `readable_html` asset; and the subscription is active for the user — either a delivery
    /// from it was seen within `active_within_days`, or the call targets that `subscription_id`
    /// directly (which bypasses the activity check). There is deliberately no inline-content
    /// (`content_html`) gate: preparation always re-renders the canonical URL.
    async fn list_prefetch_candidates(
        &self,
        user_id: UserId,
        subscription_id: Option<FeedSubscriptionId>,
        active_within_days: i64,
        limit: u32,
    ) -> Result<Vec<FeedDelivery>, AppError>;
}
