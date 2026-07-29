use chrono::{DateTime, Utc};

use crate::error::AppError;
use ind_domain::{ArchiveAssetKind, DocumentAsset, DocumentId, NewDocumentAsset};

/// A prepared (anchor-injected) copy of a document's readable HTML, staged at a
/// content-addressed key and waiting to become the readable asset's pointer.
#[derive(Debug, Clone)]
pub struct PreparedReadableLocation {
    pub s3_key: String,
    pub size_bytes: i64,
}

/// Repository for document-keyed rendered assets (`archive_assets` rows owned by a
/// `documents` row rather than a legacy `items` row). Readable preparation for the
/// document/feed/library model writes here. See
/// docs/document-feed-library-architecture.md (Archive assets and rendered content;
/// Readable Content Preparation Policy).
#[async_trait::async_trait]
pub trait DocumentAssetRepository: Send + Sync {
    /// Upsert an asset keyed by `(document_id, asset_kind)`; an existing row for the
    /// same kind is replaced so re-renders converge rather than conflict.
    async fn upsert_document_asset(
        &self,
        asset: NewDocumentAsset,
    ) -> Result<DocumentAsset, AppError>;

    /// All assets for a document, ordered by `created_at`.
    async fn find_by_document(
        &self,
        document_id: DocumentId,
    ) -> Result<Vec<DocumentAsset>, AppError>;

    /// The asset of a given kind for a document, if present (any status). The reader uses
    /// this to resolve a presigned download URL for the requested representation.
    async fn find_by_document_and_kind(
        &self,
        document_id: DocumentId,
        kind: ArchiveAssetKind,
    ) -> Result<Option<DocumentAsset>, AppError>;

    /// True when a `completed` asset of the given kind already exists for the document.
    /// Used to skip redundant preparation renders (idempotency).
    async fn has_successful_asset(
        &self,
        document_id: DocumentId,
        kind: ArchiveAssetKind,
    ) -> Result<bool, AppError>;

    /// Atomically commit an article ToC against an observed readable-html version.
    ///
    /// When `new_readable_location` is `Some`, the readable asset row's key/size are
    /// swapped to the prepared copy — guarded by `expected_readable_created_at`, and
    /// deliberately NOT refreshing `created_at`: the swap changes representation,
    /// not content version, and the pre-uploaded ToC payload records that stamp as
    /// its source version. Returns `false` when the guard fails (a reprocess won
    /// the race); nothing is written in that case. Callers never delete staged
    /// objects on a lost race — content-addressed keys can be shared with the
    /// winner.
    async fn commit_article_toc(
        &self,
        document_id: DocumentId,
        expected_readable_created_at: DateTime<Utc>,
        new_readable_location: Option<PreparedReadableLocation>,
        toc_asset: NewDocumentAsset,
    ) -> Result<bool, AppError>;
}
