use uuid::Uuid;

use crate::error::AppError;
use ind_domain::{
    Document, DocumentId, DocumentOriginType, DocumentProvenance, NewOriginDocument,
    NewUrlDocument, UserId,
};

/// YouTube enrichment resolved from the player API and applied by `document.youtube_ingest`.
#[derive(Debug, Clone, Default)]
pub struct DocumentYoutubeEnrichment {
    pub title: Option<String>,
    pub excerpt: Option<String>,
    pub lead_image_url: Option<String>,
    /// Video runtime + channel are type-specific fields persisted to `document_video_metadata`,
    /// not the wide `documents` table.
    pub duration_seconds: Option<i32>,
    pub youtube_channel_name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentRenderedMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub excerpt: Option<String>,
}

/// Repository for the document identity layer of the document/feed/library model.
/// See docs/document-feed-library-architecture.md (documents, document_origins).
///
/// The upsert helpers are standalone materialize-or-find primitives: each resolves or
/// creates a document (and its origin row) in its own transaction and does not back-link
/// feed deliveries, write library entries, or enqueue jobs. They are NOT meant to be
/// chained inside a larger transaction. The atomic materialize -> back-link -> save ->
/// outbox/events lifecycle performs its writes inside a single transaction rather than calling
/// these self-committing methods.
#[async_trait::async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn find_by_id(
        &self,
        user_id: UserId,
        id: DocumentId,
    ) -> Result<Option<Document>, AppError>;

    /// Global lookup by document id (no user scope), for worker jobs that key by `document_id`
    /// only (e.g. durable search reindex). Document ids are globally unique.
    async fn find_by_id_global(&self, id: DocumentId) -> Result<Option<Document>, AppError>;

    /// Keyset page of `(document_id, created_at)` ordered by `created_at DESC, id DESC`, for the
    /// bulk `search.reindex_all` walk. Cross-user (admin maintenance job keyed by document id).
    async fn list_ids_for_reindex(
        &self,
        after_created_at: Option<chrono::DateTime<chrono::Utc>>,
        after_id: Option<uuid::Uuid>,
        limit: i64,
    ) -> Result<Vec<(DocumentId, chrono::DateTime<chrono::Utc>)>, AppError>;

    async fn find_by_canonical_url(
        &self,
        user_id: UserId,
        canonical_url: &str,
    ) -> Result<Option<Document>, AppError>;

    async fn find_by_origin(
        &self,
        user_id: UserId,
        origin_type: DocumentOriginType,
        origin_id: Uuid,
    ) -> Result<Option<Document>, AppError>;

    /// URL-backed materialize-or-find on `(user_id, canonical_url)`. The input type
    /// guarantees a canonical URL, so this path cannot create a no-URL document.
    async fn upsert_url_backed(&self, document: NewUrlDocument) -> Result<Document, AppError>;

    /// No-URL materialize-or-find. Identity precedence: an existing origin mapping wins;
    /// otherwise dedup by `(user_id, content_hash)` when a hash is present; otherwise the
    /// `document_origins` row is the sole identity.
    async fn upsert_origin_backed(
        &self,
        document: NewOriginDocument,
        origin_type: DocumentOriginType,
        origin_id: Uuid,
    ) -> Result<Document, AppError>;

    /// Record provenance for an already-materialized document. Idempotent for the same
    /// `(origin, document)`; errors if the origin already maps to a different document.
    async fn record_origin(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        origin_type: DocumentOriginType,
        origin_id: Uuid,
    ) -> Result<(), AppError>;

    /// Apply YouTube enrichment to a document: set `document_type = 'video'` and fill
    /// title/excerpt/lead_image_url when resolved. Targeted column update (CLAUDE.md repo policy).
    async fn apply_youtube_enrichment(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        enrichment: DocumentYoutubeEnrichment,
    ) -> Result<(), AppError>;

    /// Targeted reading-metrics write (CLAUDE.md repo policy: no full-row updates). Called by
    /// the content preparation paths once readable text exists: feed prepare (renderer
    /// metadata), provided-content attach (email/extension/autosave readable HTML), and the
    /// Readwise import asset arms (HTML/PDF/EPUB).
    async fn set_reading_metrics(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        word_count: i32,
        reading_time_minutes: i32,
    ) -> Result<(), AppError>;

    /// Persist a reliably detected language without replacing declared or previously detected
    /// metadata. Returns whether the missing value was filled.
    async fn set_language_if_missing(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        language: &str,
    ) -> Result<bool, AppError>;

    /// Fill metadata resolved from rendered content without replacing values supplied by the
    /// caller or an upstream feed. URL-placeholder titles and absent author/excerpt fields are
    /// enriched; existing descriptive metadata is preserved.
    async fn apply_rendered_metadata(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        metadata: DocumentRenderedMetadata,
    ) -> Result<(), AppError>;

    /// Targeted lead-image write applied after rendering (CLAUDE.md repo policy: no full-row
    /// updates). Fills `lead_image_url` (and `thumbnail_url` when also absent) only when no image
    /// is set yet, so a feed/RSS-provided image is preserved and the rendered og:image fills the
    /// gap. Best-effort enrichment: a no-op when an image already exists.
    async fn set_lead_image(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        lead_image_url: &str,
    ) -> Result<(), AppError>;

    /// Composed provenance read model: saved/source from active `library_entries`, origins from
    /// `document_origins`, and durable engagement from capability rows. Returns `None` when the
    /// document does not exist for the user.
    async fn load_provenance(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<DocumentProvenance>, AppError>;
}
