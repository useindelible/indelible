use crate::error::AppError;

#[derive(Debug, Clone, Copy)]
pub struct FeedDeliveryRetentionWindows {
    pub unseen_days: i64,
    pub seen_days: i64,
    pub dismissed_days: i64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FeedDeliveryPruneCounts {
    pub unseen: u64,
    pub seen: u64,
    pub dismissed: u64,
}

impl FeedDeliveryPruneCounts {
    pub fn total(self) -> u64 {
        self.unseen + self.seen + self.dismissed
    }
}

#[async_trait::async_trait]
pub trait RetentionCleanupRepository: Send + Sync {
    async fn prune_feed_deliveries(
        &self,
        windows: FeedDeliveryRetentionWindows,
    ) -> Result<FeedDeliveryPruneCounts, AppError>;

    async fn compact_orphaned_feed_source_entries(
        &self,
        older_than_days: i64,
    ) -> Result<u64, AppError>;

    async fn delete_disposable_documents(
        &self,
        windows: FeedDeliveryRetentionWindows,
        document_grace_days: i64,
    ) -> Result<u64, AppError>;
}
