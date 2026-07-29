use std::sync::Arc;

use chrono::Utc;

use ind_domain::{DocumentId, DomainError, FeedDeliveryId, MilaSessionId, UserId};

use crate::AppError;
use crate::handlers::feed_identity::feed_entry_identity;
use crate::repos::document::DocumentRepository;
use crate::repos::document_lifecycle::{
    ChatIdentity, DocumentLifecycle, MaterializeSideEffects, NewChatSession,
    StartDocumentChatOutcome, StartDocumentChatRequest,
};
use crate::repos::feed::FeedRepository;
use crate::repos::feed_delivery::FeedDeliveryRepository;
use crate::repos::lifecycle_outbox::search_reindex_document_outbox;

/// What the user is opening a single-document chat against (TASK-234).
pub enum ChatTarget {
    /// An already-saved or already-materialized document (AC#1/AC#5).
    ExistingDocument(DocumentId),
    /// A feed delivery, possibly unprepared. Resolved to its document (already linked) or
    /// materialized-or-found from the source entry (AC#2).
    Delivery(FeedDeliveryId),
}

/// Owns single-document Mila chat-start orchestration in the hexagon: it resolves the chat target
/// to a `ChatIdentity` and delegates the atomic materialize/back-link/session/retain/outbox work
/// to `DocumentLifecycle::start_single_document_chat`. The embed-vs-prepare decision is made
/// inside the persistence transaction (`enqueue_engaged_ai`), not here.
pub struct MilaSessionService {
    lifecycle: Arc<dyn DocumentLifecycle>,
    document_repo: Arc<dyn DocumentRepository>,
    feed_delivery: Arc<dyn FeedDeliveryRepository>,
    feed: Arc<dyn FeedRepository>,
}

impl MilaSessionService {
    pub fn new(
        lifecycle: Arc<dyn DocumentLifecycle>,
        document_repo: Arc<dyn DocumentRepository>,
        feed_delivery: Arc<dyn FeedDeliveryRepository>,
        feed: Arc<dyn FeedRepository>,
    ) -> Self {
        Self {
            lifecycle,
            document_repo,
            feed_delivery,
            feed,
        }
    }

    pub async fn start_single_document_chat(
        &self,
        user_id: UserId,
        target: ChatTarget,
    ) -> Result<StartDocumentChatOutcome, AppError> {
        let chat_identity = self.resolve_chat_identity(user_id, target).await?;

        let session = NewChatSession {
            session_id: MilaSessionId::new(),
            user_id,
            created_at: Utc::now(),
        };

        self.lifecycle
            .start_single_document_chat(StartDocumentChatRequest {
                chat_identity,
                session,
                enqueue_engaged_ai: true,
                // Keep the durable document searchable; the content-gated AI outbox is resolved
                // inside the transaction (this closure is synchronous and DB-less).
                side_effects: Some(Box::new(|document| MaterializeSideEffects {
                    events: Vec::new(),
                    outbox: vec![search_reindex_document_outbox(document.id, Utc::now())],
                })),
            })
            .await
    }

    pub async fn load_provenance(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<ind_domain::DocumentProvenance>, AppError> {
        self.document_repo
            .load_provenance(user_id, document_id)
            .await
    }

    async fn resolve_chat_identity(
        &self,
        user_id: UserId,
        target: ChatTarget,
    ) -> Result<ChatIdentity, AppError> {
        match target {
            ChatTarget::ExistingDocument(document_id) => Ok(ChatIdentity::Existing { document_id }),
            ChatTarget::Delivery(delivery_id) => {
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

                // An already-linked delivery points at its document; chat against it directly so
                // chat-start does not redundantly re-materialize.
                if let Some(document_id) = delivery.document_id {
                    return Ok(ChatIdentity::Existing { document_id });
                }

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

                Ok(ChatIdentity::Materialize(Box::new(feed_entry_identity(
                    user_id, &entry,
                ))))
            }
        }
    }
}
