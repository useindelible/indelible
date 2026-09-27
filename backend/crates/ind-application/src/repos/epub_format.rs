use futures::future::BoxFuture;
use ind_domain::{ArchiveAssetId, DocumentId};

use crate::error::AppError;

/// An EPUB TOC at this version lists every stored chapter in `metadata.chapters`.
pub const EPUB_CHAPTER_LIST_FORMAT_VERSION: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpubFormatIssue {
    /// A chapter the book should have is missing from storage; a reprocess rebuilds it.
    MissingChapter,
    /// The original upload is gone or unreadable; only a new upload can fix the book.
    NeedsReupload,
}

impl EpubFormatIssue {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingChapter => "missing_chapter",
            Self::NeedsReupload => "needs_reupload",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "missing_chapter" => Some(Self::MissingChapter),
            "needs_reupload" => Some(Self::NeedsReupload),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpubChapterListState {
    Ready,
    Pending,
    NeedsReprocess,
    NeedsReupload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpubAssetFormat {
    pub version: i32,
    pub issue: Option<EpubFormatIssue>,
}

impl EpubAssetFormat {
    pub fn chapter_list(self) -> EpubChapterListState {
        if self.version >= EPUB_CHAPTER_LIST_FORMAT_VERSION {
            return EpubChapterListState::Ready;
        }
        match self.issue {
            Some(EpubFormatIssue::MissingChapter) => EpubChapterListState::NeedsReprocess,
            Some(EpubFormatIssue::NeedsReupload) => EpubChapterListState::NeedsReupload,
            None => EpubChapterListState::Pending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OriginalUploadLocation {
    pub s3_key: String,
    pub content_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingEpubAsset {
    pub asset_id: ArchiveAssetId,
    pub document_id: DocumentId,
    pub toc_key: String,
    pub original: Option<OriginalUploadLocation>,
}

#[async_trait::async_trait]
pub trait EpubFormatRepository: Send + Sync {
    async fn epub_assets_pending_after(
        &self,
        cursor: Option<ArchiveAssetId>,
        limit: i64,
    ) -> Result<Vec<PendingEpubAsset>, AppError>;

    async fn pending_epub_asset(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<PendingEpubAsset>, AppError>;

    async fn find_epub_format(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<EpubAssetFormat>, AppError>;

    /// Records the format and clears any issue; `size_bytes` is the rewritten TOC's size.
    async fn mark_format(
        &self,
        asset_id: ArchiveAssetId,
        version: i32,
        size_bytes: i64,
    ) -> Result<(), AppError>;

    async fn mark_format_issue(
        &self,
        asset_id: ArchiveAssetId,
        issue: EpubFormatIssue,
    ) -> Result<(), AppError>;

    /// Runs `work` holding the lock every writer of the document's EPUB TOC takes.
    async fn with_document_lock<'a>(
        &'a self,
        document_id: DocumentId,
        work: BoxFuture<'a, Result<(), AppError>>,
    ) -> Result<(), AppError>;

    /// Runs `work` under the backfill lock and returns true, or returns false without running
    /// it when another worker holds the lock.
    async fn try_global_backfill_lock<'a>(
        &'a self,
        work: BoxFuture<'a, Result<(), AppError>>,
    ) -> Result<bool, AppError>;
}
