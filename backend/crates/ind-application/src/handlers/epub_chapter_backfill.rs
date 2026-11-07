//! Adds `metadata.chapters` to EPUB TOCs stored before it existed, computed through the upload
//! port new uploads use so its spine indices match the stored `epub_ch_<n>.html` objects.

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, DocumentId};
use serde::Deserialize;
use serde_json::Value;

use crate::error::AppError;
use crate::ports::{FileUploadProcessor, UploadFileProcessRequest};
use crate::repos::epub_format::{
    EPUB_CHAPTER_LIST_FORMAT_VERSION, EpubFormatIssue, EpubFormatRepository,
    OriginalUploadLocation, PendingEpubAsset,
};
use crate::storage::{ObjectStorage, get_object_bytes};

const MAX_ATTEMPTS: u32 = 3;
const TOC_FILENAME: &str = "epub_toc.json";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EpubChapterBackfillStats {
    pub upgraded: u64,
    pub already_current: u64,
    pub needs_reprocess: u64,
    pub needs_reupload: u64,
    pub left_pending: u64,
}

enum BookOutcome {
    Upgraded,
    AlreadyCurrent,
    Flagged(EpubFormatIssue),
}

impl EpubChapterBackfillStats {
    fn record(&mut self, outcome: Option<BookOutcome>) {
        let counter = match outcome {
            Some(BookOutcome::Upgraded) => &mut self.upgraded,
            Some(BookOutcome::AlreadyCurrent) => &mut self.already_current,
            Some(BookOutcome::Flagged(EpubFormatIssue::MissingChapter)) => {
                &mut self.needs_reprocess
            }
            Some(BookOutcome::Flagged(EpubFormatIssue::NeedsReupload)) => &mut self.needs_reupload,
            None => &mut self.left_pending,
        };
        *counter += 1;
    }
}

struct CurrentChapterList {
    chapters: Value,
    spine_indices: Vec<usize>,
}

#[derive(Deserialize)]
struct ChapterRef {
    spine_index: usize,
}

pub struct EpubChapterBackfill {
    formats: Arc<dyn EpubFormatRepository>,
    storage: Arc<dyn ObjectStorage>,
    processor: Arc<dyn FileUploadProcessor>,
    retry_backoff: Duration,
}

impl EpubChapterBackfill {
    pub fn new(
        formats: Arc<dyn EpubFormatRepository>,
        storage: Arc<dyn ObjectStorage>,
        processor: Arc<dyn FileUploadProcessor>,
        retry_backoff: Duration,
    ) -> Self {
        Self {
            formats,
            storage,
            processor,
            retry_backoff,
        }
    }

    /// Visits every pending book once; a book that keeps failing stays pending for the next run.
    pub async fn run(&self, batch_size: i64) -> Result<EpubChapterBackfillStats, AppError> {
        let mut stats = EpubChapterBackfillStats::default();
        let mut cursor = None;
        loop {
            let batch = self
                .formats
                .epub_assets_pending_after(cursor, batch_size)
                .await?;
            let Some(last) = batch.last() else {
                break;
            };
            cursor = Some(last.asset_id);
            for asset in batch {
                stats.record(self.backfill_with_retry(asset.document_id).await);
            }
        }
        tracing::info!(
            upgraded = stats.upgraded,
            already_current = stats.already_current,
            needs_reprocess = stats.needs_reprocess,
            needs_reupload = stats.needs_reupload,
            left_pending = stats.left_pending,
            "EPUB chapter list backfill complete"
        );
        Ok(stats)
    }

    async fn backfill_with_retry(&self, document_id: DocumentId) -> Option<BookOutcome> {
        for attempt in 1..=MAX_ATTEMPTS {
            match self.backfill_locked(document_id).await {
                Ok(outcome) => return Some(outcome),
                Err(error) => {
                    tracing::warn!(
                        document_id = %document_id,
                        attempt,
                        error = %error,
                        "EPUB chapter list backfill attempt failed"
                    );
                    if attempt < MAX_ATTEMPTS {
                        tokio::time::sleep(self.retry_backoff * attempt).await;
                    }
                }
            }
        }
        None
    }

    async fn backfill_locked(&self, document_id: DocumentId) -> Result<BookOutcome, AppError> {
        let mut outcome = BookOutcome::AlreadyCurrent;
        let work = async {
            if let Some(asset) = self.formats.pending_epub_asset(document_id).await? {
                outcome = self.backfill_book(&asset).await?;
            }
            Ok(())
        };
        self.formats
            .with_document_lock(document_id, Box::pin(work))
            .await?;
        Ok(outcome)
    }

    async fn backfill_book(&self, asset: &PendingEpubAsset) -> Result<BookOutcome, AppError> {
        let Some(original) = &asset.original else {
            return self.flag(asset, EpubFormatIssue::NeedsReupload).await;
        };
        if !self.storage.exists(&original.s3_key).await? {
            return self.flag(asset, EpubFormatIssue::NeedsReupload).await;
        }
        let data = get_object_bytes(self.storage.as_ref(), &original.s3_key).await?;
        let Some(current) = self.current_chapter_list(asset, original, data).await else {
            return self.flag(asset, EpubFormatIssue::NeedsReupload).await;
        };

        let Some(prefix) = asset.toc_key.strip_suffix(TOC_FILENAME) else {
            return self.flag(asset, EpubFormatIssue::MissingChapter).await;
        };
        if !self.storage.exists(&asset.toc_key).await? {
            return self.flag(asset, EpubFormatIssue::MissingChapter).await;
        }
        for index in &current.spine_indices {
            let chapter_key = format!("{prefix}epub_ch_{index}.html");
            if !self.storage.exists(&chapter_key).await? {
                return self.flag(asset, EpubFormatIssue::MissingChapter).await;
            }
        }

        let stored = get_object_bytes(self.storage.as_ref(), &asset.toc_key).await?;
        let Some(patched) = with_chapter_list(&stored, current.chapters) else {
            return self.flag(asset, EpubFormatIssue::MissingChapter).await;
        };
        let upload = self
            .storage
            .upload(&asset.toc_key, "application/json", Bytes::from(patched))
            .await?;
        self.formats
            .mark_format(
                asset.asset_id,
                EPUB_CHAPTER_LIST_FORMAT_VERSION,
                upload.size_bytes,
            )
            .await?;
        Ok(BookOutcome::Upgraded)
    }

    async fn current_chapter_list(
        &self,
        asset: &PendingEpubAsset,
        original: &OriginalUploadLocation,
        data: Vec<u8>,
    ) -> Option<CurrentChapterList> {
        let processed = self
            .processor
            .process_upload(UploadFileProcessRequest {
                filename: "original_upload.epub".into(),
                content_type: original.content_type.clone(),
                data: Bytes::from(data),
                title_override: None,
                max_bytes: usize::MAX,
            })
            .await
            .inspect_err(|error| {
                tracing::warn!(
                    document_id = %asset.document_id,
                    error = %error,
                    "EPUB original upload no longer processes"
                );
            })
            .ok()?;
        let toc = processed.assets.into_iter().find(|processed| {
            processed.asset_kind == Some(ArchiveAssetKind::Epub)
                && processed.status == ArchiveAssetStatus::Completed
        })?;
        let toc: Value = serde_json::from_slice(&toc.bytes).ok()?;
        let chapters = toc.get("metadata")?.get("chapters")?.clone();
        let refs: Vec<ChapterRef> = serde_json::from_value(chapters.clone()).ok()?;
        Some(CurrentChapterList {
            chapters,
            spine_indices: refs
                .into_iter()
                .map(|chapter| chapter.spine_index)
                .collect(),
        })
    }

    async fn flag(
        &self,
        asset: &PendingEpubAsset,
        issue: EpubFormatIssue,
    ) -> Result<BookOutcome, AppError> {
        self.formats
            .mark_format_issue(asset.asset_id, issue)
            .await?;
        Ok(BookOutcome::Flagged(issue))
    }
}

/// Sets `metadata.chapters` on the stored TOC and leaves every other field as stored.
fn with_chapter_list(stored: &[u8], chapters: Value) -> Option<Vec<u8>> {
    let mut toc: Value = serde_json::from_slice(stored).ok()?;
    toc.get_mut("metadata")?
        .as_object_mut()?
        .insert("chapters".into(), chapters);
    serde_json::to_vec(&toc).ok()
}

#[cfg(test)]
#[path = "epub_chapter_backfill_doubles.rs"]
mod test_doubles;
#[cfg(test)]
#[path = "epub_chapter_backfill_tests.rs"]
mod tests;
