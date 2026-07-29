//! Library save + read service for the document/feed/library model.
//!
//! Saves go through the atomic `DocumentLifecycle::save_to_library` (materialize-or-find the
//! document, insert/restore the library entry, force retained='saved', hide deliveries) so the
//! whole save commits in one transaction. Reads delegate to `LibraryRepository`, which always
//! joins `documents` and never scans the feed firehose. See
//! docs/document-feed-library-architecture.md (library_entries; User saves a feed-delivered
//! document; Query Surfaces -> Library).

use std::sync::Arc;

use chrono::Utc;
use ind_domain::{
    CanonicalizationConfig, ContentSource, DocumentId, DomainError, FeedDeliveryId, LibraryEntry,
    LibraryEntryId, LibraryEntryWithDocument, NewUrlDocument, TriageState, UserId,
    canonicalize_url,
};

use futures::future::BoxFuture;

use crate::dispatch::infer_item_type_for_url;
use crate::error::AppError;
use crate::handlers::feed_identity::{document_type_for, domain_from_url, feed_entry_identity};
use crate::ports::{LibraryOperations, OutboundUrlGuard, SaveUrlRequest};
use crate::repos::document_lifecycle::{
    DocumentLifecycle, MaterializeIdentity, MaterializeSideEffects, SaveToLibraryOutcome,
    SaveToLibraryRequest,
};
use crate::repos::event::MutationSideEffects;
use crate::repos::feed::FeedRepository;
use crate::repos::feed_delivery::FeedDeliveryRepository;
use crate::repos::library::{LibraryRepository, LibraryScopeCounts};
use crate::repos::lifecycle_outbox::search_reindex_document_outbox;
use crate::repos::{Cursor, Page};

pub struct LibraryService {
    lifecycle: Arc<dyn DocumentLifecycle>,
    library: Arc<dyn LibraryRepository>,
    feed_delivery: Arc<dyn FeedDeliveryRepository>,
    feed: Arc<dyn FeedRepository>,
    url_guard: Arc<dyn OutboundUrlGuard>,
}

impl LibraryService {
    pub fn new(
        lifecycle: Arc<dyn DocumentLifecycle>,
        library: Arc<dyn LibraryRepository>,
        feed_delivery: Arc<dyn FeedDeliveryRepository>,
        feed: Arc<dyn FeedRepository>,
        url_guard: Arc<dyn OutboundUrlGuard>,
    ) -> Self {
        Self {
            lifecycle,
            library,
            feed_delivery,
            feed,
            url_guard,
        }
    }

    /// Manual/URL/API save: canonicalize, materialize-or-find the document, and insert/restore
    /// the library entry. A URL already seen in Feed leaves Feed (`hide_deliveries`).
    pub async fn save_url(
        &self,
        user_id: UserId,
        req: SaveUrlRequest,
    ) -> Result<SaveToLibraryOutcome, AppError> {
        // SSRF guard: reject private/internal targets before queuing a render
        // job. The renderer also pre-flights (defense in depth), but rejecting
        // here returns a clean 422 instead of a doomed background job.
        self.url_guard.check_url(&req.url).await.map_err(|e| {
            AppError::Domain(DomainError::Validation {
                field: "url".into(),
                message: e.message().to_string(),
            })
        })?;

        let canonical = canonicalize_url(&req.url, &CanonicalizationConfig::default())
            .map(|c| c.into_string())
            .map_err(|_| {
                AppError::Domain(DomainError::Validation {
                    field: "url".into(),
                    message: "url could not be canonicalized".into(),
                })
            })?;

        let document_type = req
            .item_type
            .unwrap_or_else(|| document_type_for(infer_item_type_for_url(&req.url)));

        let document = NewUrlDocument {
            id: DocumentId::new(),
            user_id,
            document_type,
            canonical_url: canonical,
            original_url: Some(req.url.clone()),
            content_hash: None,
            title: req.title.unwrap_or_else(|| req.url.clone()),
            author: None,
            excerpt: None,
            published_at: None,
            language: None,
            domain: domain_from_url(&req.url),
            lead_image_url: None,
            thumbnail_url: None,
        };

        self.lifecycle
            .save_to_library(SaveToLibraryRequest {
                identity: MaterializeIdentity::Url {
                    document,
                    origin: None,
                },
                source: ContentSource::Manual,
                source_delivery_id: None,
                hide_deliveries: true,
                // TASK-234: the save transaction resolves the content-gated AI outbox
                // (embed-if-prepared, else enqueue preparation) from the resolved document.
                enqueue_engaged_ai: true,
                restore_policy: Default::default(),
                // AC #4: enqueue durable document search reindex atomically with the save.
                side_effects: Some(Box::new(|ctx| MaterializeSideEffects {
                    events: Vec::new(),
                    outbox: vec![search_reindex_document_outbox(ctx.document.id, Utc::now())],
                })),
            })
            .await
    }

    /// Save a feed delivery: resolve the delivery and its source entry, then materialize-or-find
    /// the document (URL-backed when the entry has a canonical URL, otherwise origin-backed),
    /// record provenance via `source_delivery_id`, and hide the matching deliveries.
    pub async fn save_from_delivery(
        &self,
        user_id: UserId,
        delivery_id: FeedDeliveryId,
    ) -> Result<SaveToLibraryOutcome, AppError> {
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

        let identity = feed_entry_identity(user_id, &entry);

        self.lifecycle
            .save_to_library(SaveToLibraryRequest {
                identity,
                source: ContentSource::Feed,
                source_delivery_id: Some(delivery_id),
                hide_deliveries: true,
                // TASK-234: content-gated AI outbox resolved from the resolved document.
                enqueue_engaged_ai: true,
                restore_policy: Default::default(),
                // AC #4: enqueue durable document search reindex atomically with the save.
                side_effects: Some(Box::new(|ctx| MaterializeSideEffects {
                    events: Vec::new(),
                    outbox: vec![search_reindex_document_outbox(ctx.document.id, Utc::now())],
                })),
            })
            .await
    }

    pub async fn list(
        &self,
        user_id: UserId,
        triage: Option<TriageState>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        self.library
            .list_by_user(user_id, triage, cursor, limit)
            .await
    }

    pub async fn get(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
    ) -> Result<Option<LibraryEntryWithDocument>, AppError> {
        self.library.find_by_id(id, user_id).await
    }

    pub async fn check_url(
        &self,
        user_id: UserId,
        canonical_url: &str,
    ) -> Result<Option<crate::ports::LibraryUrlCheckResult>, AppError> {
        Ok(self
            .library
            .find_active_by_canonical_url(user_id, canonical_url)
            .await?
            .map(|joined| crate::ports::LibraryUrlCheckResult {
                entry: joined.entry,
                document: joined.document,
            }))
    }

    pub async fn set_triage(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
        state: TriageState,
    ) -> Result<LibraryEntry, AppError> {
        self.library
            .set_triage_state(id, user_id, state, MutationSideEffects::none())
            .await
    }

    pub async fn toggle_favorite(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
    ) -> Result<LibraryEntry, AppError> {
        self.library
            .toggle_favorite(id, user_id, MutationSideEffects::none())
            .await
    }

    pub async fn toggle_shortlist(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
    ) -> Result<LibraryEntry, AppError> {
        self.library
            .toggle_shortlist(id, user_id, MutationSideEffects::none())
            .await
    }

    pub async fn delete(&self, user_id: UserId, id: LibraryEntryId) -> Result<(), AppError> {
        self.library
            .soft_delete(id, user_id, MutationSideEffects::none())
            .await
    }

    pub async fn restore(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
    ) -> Result<LibraryEntry, AppError> {
        self.library
            .restore(id, user_id, MutationSideEffects::none())
            .await
    }

    pub async fn purge(&self, user_id: UserId, id: LibraryEntryId) -> Result<(), AppError> {
        self.library
            .purge(id, user_id, MutationSideEffects::none())
            .await
    }

    pub async fn empty_trash(&self, user_id: UserId) -> Result<u64, AppError> {
        self.library.purge_all_trashed(user_id).await
    }

    pub async fn list_trashed(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        self.library.list_trashed(user_id, cursor, limit).await
    }

    pub async fn count(&self, user_id: UserId) -> Result<i64, AppError> {
        self.library.count_active(user_id).await
    }

    pub async fn count_trashed(&self, user_id: UserId) -> Result<i64, AppError> {
        self.library.count_trashed(user_id).await
    }

    pub async fn scope_counts(
        &self,
        user_id: UserId,
        triage: Option<TriageState>,
    ) -> Result<LibraryScopeCounts, AppError> {
        self.library.scope_counts(user_id, triage).await
    }
}

/// The port delegates to the inherent async methods (which shadow the trait methods of the
/// same name), so `LibraryService` can be used directly as `Arc<dyn LibraryOperations>` from
/// both the API wiring and the test harness without a separate adapter.
impl LibraryOperations for LibraryService {
    fn save_url(
        &self,
        user_id: UserId,
        req: SaveUrlRequest,
    ) -> BoxFuture<'_, Result<SaveToLibraryOutcome, AppError>> {
        Box::pin(self.save_url(user_id, req))
    }

    fn save_from_delivery(
        &self,
        user_id: UserId,
        delivery_id: FeedDeliveryId,
    ) -> BoxFuture<'_, Result<SaveToLibraryOutcome, AppError>> {
        Box::pin(self.save_from_delivery(user_id, delivery_id))
    }

    fn list(
        &self,
        user_id: UserId,
        triage: Option<TriageState>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> BoxFuture<'_, Result<Page<LibraryEntryWithDocument>, AppError>> {
        Box::pin(self.list(user_id, triage, cursor, limit))
    }

    fn get(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<Option<LibraryEntryWithDocument>, AppError>> {
        Box::pin(self.get(user_id, id))
    }

    fn check_url(
        &self,
        user_id: UserId,
        canonical_url: &str,
    ) -> BoxFuture<'_, Result<Option<crate::ports::LibraryUrlCheckResult>, AppError>> {
        let canonical_url = canonical_url.to_owned();
        Box::pin(async move { self.check_url(user_id, &canonical_url).await })
    }

    fn set_triage(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
        state: TriageState,
    ) -> BoxFuture<'_, Result<LibraryEntry, AppError>> {
        Box::pin(self.set_triage(user_id, id, state))
    }

    fn toggle_favorite(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<LibraryEntry, AppError>> {
        Box::pin(self.toggle_favorite(user_id, id))
    }

    fn toggle_shortlist(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<LibraryEntry, AppError>> {
        Box::pin(self.toggle_shortlist(user_id, id))
    }

    fn delete(&self, user_id: UserId, id: LibraryEntryId) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.delete(user_id, id))
    }

    fn restore(
        &self,
        user_id: UserId,
        id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<LibraryEntry, AppError>> {
        Box::pin(self.restore(user_id, id))
    }

    fn purge(&self, user_id: UserId, id: LibraryEntryId) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.purge(user_id, id))
    }

    fn empty_trash(&self, user_id: UserId) -> BoxFuture<'_, Result<u64, AppError>> {
        Box::pin(self.empty_trash(user_id))
    }

    fn list_trashed(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> BoxFuture<'_, Result<Page<LibraryEntryWithDocument>, AppError>> {
        Box::pin(self.list_trashed(user_id, cursor, limit))
    }

    fn count(&self, user_id: UserId) -> BoxFuture<'_, Result<i64, AppError>> {
        Box::pin(self.count(user_id))
    }

    fn count_trashed(&self, user_id: UserId) -> BoxFuture<'_, Result<i64, AppError>> {
        Box::pin(self.count_trashed(user_id))
    }

    fn scope_counts(
        &self,
        user_id: UserId,
        triage: Option<TriageState>,
    ) -> BoxFuture<'_, Result<LibraryScopeCounts, AppError>> {
        Box::pin(self.scope_counts(user_id, triage))
    }
}
