use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::AppError;
use crate::repos::lifecycle_outbox::OutboxEntry;
use ind_domain::{
    ContentSource, Document, DocumentId, DocumentOriginType, FeedDeliveryId, LibraryEntry,
    MilaSession, MilaSessionId, NewDomainEvent, NewOriginDocument, NewUrlDocument,
    UserDocumentState, UserId,
};

/// Provenance origin recorded for a materialized document
/// (`document_origins(user_id, origin_type, origin_id)`).
#[derive(Debug, Clone, Copy)]
pub struct MaterializeOrigin {
    pub origin_type: DocumentOriginType,
    pub origin_id: Uuid,
}

/// How the document identity is resolved during materialization.
pub enum MaterializeIdentity {
    /// URL-backed content. Materialize-or-find on `(user_id, canonical_url)`, then
    /// back-link matching unlinked deliveries by canonical URL. An optional provenance
    /// origin (e.g. the feed source entry that triggered preparation) is also recorded.
    Url {
        document: NewUrlDocument,
        origin: Option<MaterializeOrigin>,
    },
    /// No-URL content. Identity is the origin row; back-linking uses the source entry
    /// when `origin_type` is a feed source entry.
    Origin {
        document: NewOriginDocument,
        origin: MaterializeOrigin,
    },
}

/// Targeted `user_document_state` transition applied only when present on the request.
/// `first_opened_at` is written only when currently NULL; `last_opened_at` only moves forward
/// and never regresses to an older `opened_at`.
#[derive(Debug, Clone, Default)]
pub struct DocumentStateInput {
    pub opened_at: Option<DateTime<Utc>>,
}

/// Domain events and job-outbox rows committed atomically with the materialization.
#[derive(Default)]
pub struct MaterializeSideEffects {
    pub events: Vec<NewDomainEvent>,
    pub outbox: Vec<OutboxEntry>,
}

/// Builds side effects from the RESOLVED document. The lifecycle invokes this only after
/// it has materialized or found the durable document, so events/outbox always reference
/// the real `Document` id — never the caller's candidate id, which is discarded when an
/// existing document is found on conflict.
pub type MaterializeSideEffectsFn = Box<dyn FnOnce(&Document) -> MaterializeSideEffects + Send>;

/// One unit of atomic work for the document materialization/engagement transaction.
/// Deliberately carries no library-save field: creating library membership is a separate
/// public lifecycle method that owns its own transaction (see the module/trait docs).
pub struct MaterializeRequest {
    pub identity: MaterializeIdentity,
    pub document_state: Option<DocumentStateInput>,
    pub side_effects: Option<MaterializeSideEffectsFn>,
}

/// Result of a materialization. `created` is true when this call inserted the document.
pub struct MaterializeOutcome {
    pub document: Document,
    pub created: bool,
    pub backlinked_deliveries: u64,
    pub state: Option<UserDocumentState>,
}

/// Read-only view of the resolved rows passed to a save-side-effect builder. The intrinsic
/// `library_entry.saved` event is built by `save_to_library` itself; this builder is only for
/// ADDITIONAL caller effects (events/outbox) that need the real resolved ids.
pub struct SaveContext<'a> {
    pub document: &'a Document,
    pub entry: &'a LibraryEntry,
    pub document_created: bool,
    pub restored: bool,
    pub already_active: bool,
}

/// Builds extra side effects from the RESOLVED `(document, entry)`. The save flow already emits
/// its own intrinsic `library_entry.saved` event; this closure is optional and only for callers
/// that need to attach more events/outbox rows.
pub type SaveSideEffectsFn =
    Box<dyn for<'a> FnOnce(&SaveContext<'a>) -> MaterializeSideEffects + Send>;

#[derive(Debug, Clone, Copy, Default)]
pub enum LibraryRestorePolicy {
    #[default]
    RestoreDeleted,
    SkipIfDeletedAfter(DateTime<Utc>),
}

/// One atomic save: materialize-or-find the document, back-link matching deliveries,
/// insert-or-restore the `library_entries` row, optionally hide linked deliveries, and commit the
/// intrinsic save event + outbox.
pub struct SaveToLibraryRequest {
    pub identity: MaterializeIdentity,
    pub source: ContentSource,
    pub source_delivery_id: Option<FeedDeliveryId>,
    pub hide_deliveries: bool,
    /// Content-gated AI enablement (TASK-234). When true the save transaction resolves the AI
    /// outbox via `build_engaged_document_ai_outbox_tx` (embed if a completed readable asset
    /// exists, else enqueue preparation). The decision is made inside the tx, not in the
    /// synchronous DB-less `side_effects` closure.
    pub enqueue_engaged_ai: bool,
    pub restore_policy: LibraryRestorePolicy,
    pub side_effects: Option<SaveSideEffectsFn>,
}

/// How the chat target document is resolved for `start_single_document_chat`.
pub enum ChatIdentity {
    /// An already-saved or already-materialized document. The transaction loads it (verifying
    /// ownership) and skips materialization/back-linking; no Library change (AC#1/AC#5).
    Existing { document_id: DocumentId },
    /// An unprepared feed delivery (or URL). Materialize-or-find through the same machinery as
    /// `materialize_document`/`save_to_library`, back-linking matching deliveries (AC#2). Boxed
    /// because `MaterializeIdentity` is far larger than the `Existing` variant.
    Materialize(Box<MaterializeIdentity>),
}

/// Seed for the `mila_sessions` row inserted in the chat-start transaction. The lifecycle fills
/// `document_id` from the resolved document, so the seed carries only the session identity.
pub struct NewChatSession {
    pub session_id: MilaSessionId,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
}

/// One atomic chat-start: resolve the document (load-existing or materialize-or-find), insert the
/// single-document `mila_sessions` row, resolve the content-gated AI outbox (when
/// `enqueue_engaged_ai`), and emit `document.engaged` when the document is not already saved — all
/// in ONE transaction.
pub struct StartDocumentChatRequest {
    pub chat_identity: ChatIdentity,
    pub session: NewChatSession,
    pub enqueue_engaged_ai: bool,
    pub side_effects: Option<MaterializeSideEffectsFn>,
}

/// Result of a chat-start. `document_created` is true when the document row was inserted by this
/// call; `backlinked_deliveries` counts deliveries linked when materializing a feed delivery.
pub struct StartDocumentChatOutcome {
    pub document: Document,
    pub session: MilaSession,
    pub document_created: bool,
    pub backlinked_deliveries: u64,
}

/// Result of a save. `document_created` is true when the document row was inserted by this call;
/// `restored` when a soft-deleted entry was revived; `already_active` when an active entry already
/// existed (idempotent save).
pub struct SaveToLibraryOutcome {
    pub document: Document,
    pub document_created: bool,
    pub entry: LibraryEntry,
    pub restored: bool,
    pub skipped_restore: bool,
    pub already_active: bool,
    pub backlinked_deliveries: u64,
    pub hidden_deliveries: u64,
}

/// Atomic document materialization and adoption.
/// See docs/document-feed-library-architecture.md (Materialization and adoption must be atomic).
///
/// Public methods are the transaction boundary. `materialize_document` owns one
/// transaction for document creation/adoption, provenance, delivery back-linking,
/// opened `user_document_state`, `domain_events`, and `job_outbox`. It must NOT be used by flows
/// that also create library membership: a save flow must be a single public
/// lifecycle call (`save_to_library` composing the same internal tx helpers),
/// never `materialize_document` followed by a separate `library_repo` insert.
#[async_trait::async_trait]
pub trait DocumentLifecycle: Send + Sync {
    async fn materialize_document(
        &self,
        request: MaterializeRequest,
    ) -> Result<MaterializeOutcome, AppError>;

    /// Atomic save to Library. Owns ONE transaction composing the same `*_tx` helpers as
    /// `materialize_document` plus the library-entry insert/restore and optional delivery hiding.
    /// This is the single public save flow — callers must NOT `materialize_document` then insert a
    /// library row separately. See docs/document-feed-library-architecture.md (User saves a
    /// feed-delivered document; Materialization and adoption must be atomic).
    async fn save_to_library(
        &self,
        request: SaveToLibraryRequest,
    ) -> Result<SaveToLibraryOutcome, AppError>;

    /// Atomic single-document chat-start (TASK-234). Composes the same `*_tx` helpers as
    /// `materialize_document` plus the `mila_sessions` insert and content-gated AI outbox. This is
    /// the single public chat-start flow — callers must NOT materialize then insert a session separately
    /// (`materialize_document` cannot commit a session row). See
    /// docs/document-feed-library-architecture.md (single-document chat attaches to document_id
    /// regardless of saved state).
    async fn start_single_document_chat(
        &self,
        request: StartDocumentChatRequest,
    ) -> Result<StartDocumentChatOutcome, AppError>;
}
