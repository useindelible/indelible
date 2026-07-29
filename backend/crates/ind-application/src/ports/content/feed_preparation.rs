use super::*;
use ind_domain::{DocumentId, FeedDeliveryId, FeedSubscriptionId};

/// Outcome of a read-ahead pass: how many documents were materialized and queued for
/// readable preparation, and their ids.
pub struct ReadAheadOutcome {
    pub prepared: u32,
    pub document_ids: Vec<DocumentId>,
}

/// Outcome of an on-tap prepare: the resolved/adopted document id. A render job is always
/// enqueued (the worker is idempotent), so callers load the document and poll its readable
/// asset rather than relying on a queued flag.
pub struct PrepareDeliveryOutcome {
    pub document_id: DocumentId,
}

/// HTTP-facing port for active-feed readable preparation (docs/document-feed-library-
/// architecture.md, Readable Content Preparation Policy). Read-ahead prepares the newest
/// eligible deliveries when the user opens Feed; on-tap prepares a single delivery for the
/// canonical reader. Both materialize/adopt the document through the lifecycle and enqueue a
/// low-priority `feed.prepare_document` render; neither uses the feed's inline content.
pub trait FeedPreparationOperations: Send + Sync {
    fn prepare_read_ahead(
        &self,
        user_id: UserId,
        subscription_id: Option<FeedSubscriptionId>,
    ) -> BoxFuture<'_, Result<ReadAheadOutcome, AppError>>;

    fn prepare_delivery(
        &self,
        user_id: UserId,
        delivery_id: FeedDeliveryId,
    ) -> BoxFuture<'_, Result<PrepareDeliveryOutcome, AppError>>;
}
