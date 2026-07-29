//! Active-feed readable preparation for the document/feed/library model.
//!
//! `prepare_read_ahead` runs when the user opens Feed: it selects the newest eligible
//! deliveries (active subscription, unread, URL-backed, not already prepared) and, for each,
//! materializes/adopts the document through `DocumentLifecycle` and commits one
//! `feed.prepare_document` render job in the same transaction. `prepare_delivery` is the on-tap
//! canonical-reader path for a single delivery. Neither uses the feed's inline content: the
//! worker always re-renders the canonical URL. See docs/document-feed-library-architecture.md
//! (Readable Content Preparation Policy; Read-ahead prepares active feed entries).

use std::sync::Arc;

use chrono::Utc;
use futures::future::BoxFuture;
use ind_domain::{
    Document, DocumentId, DomainError, FeedDeliveryId, FeedSourceEntry, FeedSubscriptionId,
    PrepareDocumentJob, UserId, job_types,
};

use crate::error::AppError;
use crate::handlers::feed_identity::feed_entry_identity;
use crate::ports::{FeedPreparationOperations, PrepareDeliveryOutcome, ReadAheadOutcome};
use crate::repos::document_lifecycle::{
    DocumentLifecycle, DocumentStateInput, MaterializeRequest, MaterializeSideEffects,
    MaterializeSideEffectsFn,
};
use crate::repos::feed::FeedRepository;
use crate::repos::feed_delivery::FeedDeliveryRepository;
use crate::repos::lifecycle_outbox::OutboxEntry;

/// Static read-ahead knobs (docs/document-feed-library-architecture.md defaults). There is no
/// `truncated_threshold_chars`: preparation never trusts inline feed content.
#[derive(Debug, Clone, Copy)]
pub struct FeedPreparationConfig {
    pub enabled: bool,
    pub read_ahead_count: u32,
    pub active_within_days: i64,
}

pub struct FeedPreparationService {
    lifecycle: Arc<dyn DocumentLifecycle>,
    feed_delivery: Arc<dyn FeedDeliveryRepository>,
    feed: Arc<dyn FeedRepository>,
    config: FeedPreparationConfig,
}

impl FeedPreparationService {
    pub fn new(
        lifecycle: Arc<dyn DocumentLifecycle>,
        feed_delivery: Arc<dyn FeedDeliveryRepository>,
        feed: Arc<dyn FeedRepository>,
        config: FeedPreparationConfig,
    ) -> Self {
        Self {
            lifecycle,
            feed_delivery,
            feed,
            config,
        }
    }

    /// Read-ahead: materialize/adopt and enqueue preparation for the newest eligible deliveries.
    /// A no-op when prefetch is disabled. The candidate query already excludes deliveries whose
    /// document has a completed readable asset, so each returned document is prepared exactly once.
    pub async fn prepare_read_ahead(
        &self,
        user_id: UserId,
        subscription_id: Option<FeedSubscriptionId>,
    ) -> Result<ReadAheadOutcome, AppError> {
        if !self.config.enabled {
            return Ok(ReadAheadOutcome {
                prepared: 0,
                document_ids: Vec::new(),
            });
        }

        let candidates = self
            .feed_delivery
            .list_prefetch_candidates(
                user_id,
                subscription_id,
                self.config.active_within_days,
                self.config.read_ahead_count,
            )
            .await?;

        let mut document_ids = Vec::new();
        for delivery in candidates {
            let Some(entry) = self
                .feed
                .find_source_entry_by_id(delivery.source_entry_id)
                .await?
            else {
                continue;
            };
            // Selection already requires canonical_url; this guards against a race where it was
            // cleared, and keeps preparation strictly URL-backed.
            let Some(canonical) = entry.canonical_url.clone() else {
                continue;
            };
            let document_id = self
                .materialize_and_enqueue(user_id, &entry, &canonical, None)
                .await?;
            document_ids.push(document_id);
        }

        Ok(ReadAheadOutcome {
            prepared: document_ids.len() as u32,
            document_ids,
        })
    }

    /// On-tap canonical-reader preparation for a single delivery. Requires a canonical URL
    /// (422 otherwise — there is nothing to render and no inline fallback). Materializes/adopts
    /// the document, sets `last_opened_at`/`first_opened_at`, enqueues a render, and marks the
    /// delivery seen.
    pub async fn prepare_delivery(
        &self,
        user_id: UserId,
        delivery_id: FeedDeliveryId,
    ) -> Result<PrepareDeliveryOutcome, AppError> {
        let delivery = self
            .feed_delivery
            .find_by_id(delivery_id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "FeedDelivery",
                    id: delivery_id.to_string(),
                })
            })?;

        let entry = self
            .feed
            .find_source_entry_by_id(delivery.source_entry_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "FeedSourceEntry",
                    id: delivery.source_entry_id.to_string(),
                })
            })?;

        let Some(canonical) = entry.canonical_url.clone() else {
            return Err(AppError::Domain(DomainError::Validation {
                field: "delivery_id".into(),
                message: "feed delivery has no canonical URL to prepare; readable content \
                          requires a URL"
                    .into(),
            }));
        };

        let document_state = Some(DocumentStateInput {
            opened_at: Some(Utc::now()),
        });
        let document_id = self
            .materialize_and_enqueue(user_id, &entry, &canonical, document_state)
            .await?;

        // Mark seen only after the document is materialized and the render is queued (lifecycle
        // step 6). Doing it earlier would drop the delivery from Unseen even if preparation
        // failed, leaving partial lifecycle state with nothing prepared to read.
        self.feed_delivery.mark_seen(delivery_id, user_id).await?;

        Ok(PrepareDeliveryOutcome { document_id })
    }

    /// Materialize/adopt the URL-backed document for a feed entry and attach a
    /// `feed.prepare_document` render job committed in the same transaction. Enqueuing is
    /// unconditional: the worker skips when a completed readable asset already exists and the
    /// outbox `dedupe_key` collapses duplicate pending jobs, so no pre-check is needed.
    async fn materialize_and_enqueue(
        &self,
        user_id: UserId,
        entry: &FeedSourceEntry,
        canonical: &str,
        document_state: Option<DocumentStateInput>,
    ) -> Result<DocumentId, AppError> {
        let url = canonical.to_string();
        let side_effects: MaterializeSideEffectsFn =
            Box::new(move |document: &Document| MaterializeSideEffects {
                events: Vec::new(),
                outbox: vec![prepare_document_outbox(document.id, user_id, url)],
            });

        let outcome = self
            .lifecycle
            .materialize_document(MaterializeRequest {
                identity: feed_entry_identity(user_id, entry),
                document_state,
                side_effects: Some(side_effects),
            })
            .await?;

        Ok(outcome.document.id)
    }
}

/// Build the readable-preparation outbox row from the resolved document. The `dedupe_key`
/// collapses duplicate pending jobs (concurrent read-ahead + on-tap) before any render
/// finishes; the worker's completed-asset check is the post-completion guard.
fn prepare_document_outbox(document_id: DocumentId, user_id: UserId, url: String) -> OutboxEntry {
    #[expect(
        clippy::expect_used,
        reason = "serializing a plain owned struct with no map keys or non-string keys into serde_json::Value cannot fail"
    )]
    let payload = serde_json::to_value(PrepareDocumentJob {
        document_id,
        user_id,
        url,
    })
    .expect("PrepareDocumentJob serializes");
    OutboxEntry {
        job_type: job_types::FEED_PREPARE_DOCUMENT.into(),
        payload,
        dedupe_key: Some(format!(
            "{}:{document_id}",
            job_types::FEED_PREPARE_DOCUMENT
        )),
        available_at: Utc::now(),
    }
}

impl FeedPreparationOperations for FeedPreparationService {
    fn prepare_read_ahead(
        &self,
        user_id: UserId,
        subscription_id: Option<FeedSubscriptionId>,
    ) -> BoxFuture<'_, Result<ReadAheadOutcome, AppError>> {
        Box::pin(self.prepare_read_ahead(user_id, subscription_id))
    }

    fn prepare_delivery(
        &self,
        user_id: UserId,
        delivery_id: FeedDeliveryId,
    ) -> BoxFuture<'_, Result<PrepareDeliveryOutcome, AppError>> {
        Box::pin(self.prepare_delivery(user_id, delivery_id))
    }
}
