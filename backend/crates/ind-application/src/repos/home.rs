use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ind_domain::{
    Collection, DailyReviewSummary, DocumentId, FeedDeliveryId, FeedSubscriptionId, ItemType,
    ReadingStatsSummary, UserId,
};

use crate::AppError;

/// Document read-model for the home dashboard. Keyed by `document_id`; the field set is exactly
/// what the home widget queries project, so it is not a full `Document`.
#[derive(Debug, Clone)]
pub struct HomeDocument {
    pub document_id: DocumentId,
    pub item_type: ItemType,
    pub title: String,
    pub excerpt: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub author: Option<String>,
    pub reading_time_minutes: Option<i32>,
    pub lead_image_url: Option<String>,
    pub progress_percent: Option<f32>,
    pub max_progress_percent: Option<f32>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Feed-delivery read-model for the home digest widget. Projects exactly the fields the digest
/// response surfaces.
#[derive(Debug, Clone)]
pub struct HomeFeedEntry {
    pub delivery_id: FeedDeliveryId,
    pub subscription_id: FeedSubscriptionId,
    pub title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub excerpt: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait HomeRepository: Send + Sync {
    async fn continue_reading(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<HomeDocument>, AppError>;
    async fn daily_review_summary(&self, user_id: UserId) -> Result<DailyReviewSummary, AppError>;
    async fn recently_added(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<HomeDocument>, AppError>;
    async fn quick_reads(&self, user_id: UserId, limit: i64)
    -> Result<Vec<HomeDocument>, AppError>;
    async fn pinned_collections(
        &self,
        user_id: UserId,
        collection_limit: i64,
        items_per: i64,
    ) -> Result<Vec<(Collection, Vec<HomeDocument>)>, AppError>;
    async fn reading_stats_weekly(&self, user_id: UserId) -> Result<ReadingStatsSummary, AppError>;
    async fn feed_digest(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<HomeFeedEntry>, AppError>;
    async fn get_widget_config(
        &self,
        user_id: UserId,
    ) -> Result<Option<serde_json::Value>, AppError>;
    async fn set_widget_config(
        &self,
        user_id: UserId,
        config: serde_json::Value,
    ) -> Result<(), AppError>;
}
