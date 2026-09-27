use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use ind_domain::{ArchiveAssetId, DocumentId};

use super::test_doubles::{FakeFormats, FakeProcessor, FakeStorage, Row, chapter_list};
use super::*;
use crate::repos::epub_format::{EpubAssetFormat, OriginalUploadLocation};

struct Scenario {
    formats: Arc<FakeFormats>,
    storage: Arc<FakeStorage>,
    processor: Arc<FakeProcessor>,
}

impl Scenario {
    fn new(spine_indices: &[usize], reprocess_holds_lock_first: bool) -> Self {
        Self {
            formats: Arc::new(FakeFormats {
                reprocess_holds_lock_first,
                ..FakeFormats::default()
            }),
            storage: Arc::new(FakeStorage::default()),
            processor: Arc::new(FakeProcessor {
                spine_indices: spine_indices.to_vec(),
                calls: AtomicU32::new(0),
            }),
        }
    }

    /// Stores a processed book without a chapter list and returns its document and TOC key.
    fn book(&self, original: Option<&'static [u8]>, chapters: &[usize]) -> (DocumentId, String) {
        let document_id = DocumentId::new();
        let prefix = format!("documents/{document_id}/");
        let toc_key = format!("{prefix}epub_toc.json");
        self.storage
            .put(&toc_key, r#"{"metadata":{"title":"Stored"},"toc":[]}"#);
        for index in chapters {
            self.storage
                .put(&format!("{prefix}epub_ch_{index}.html"), "<p>chapter</p>");
        }
        let original = original.map(|bytes| {
            self.storage
                .put(&format!("{prefix}original_upload.epub"), bytes);
            OriginalUploadLocation {
                s3_key: format!("{prefix}original_upload.epub"),
                content_type: "application/epub+zip".into(),
            }
        });
        let asset = PendingEpubAsset {
            asset_id: ArchiveAssetId::new(),
            document_id,
            toc_key: toc_key.clone(),
            original,
        };
        let format = EpubAssetFormat {
            version: 1,
            issue: None,
        };
        self.formats
            .rows
            .lock()
            .unwrap()
            .push(Row { asset, format });
        (document_id, toc_key)
    }

    async fn run(&self) -> EpubChapterBackfillStats {
        EpubChapterBackfill::new(
            self.formats.clone(),
            self.storage.clone(),
            self.processor.clone(),
            Duration::ZERO,
        )
        .run(10)
        .await
        .unwrap()
    }
}

fn format(version: i32, issue: Option<EpubFormatIssue>) -> EpubAssetFormat {
    EpubAssetFormat { version, issue }
}

#[tokio::test]
async fn book_already_at_version_2_inside_the_lock_is_untouched() {
    let scenario = Scenario::new(&[0], true);
    let (document_id, toc_key) = scenario.book(Some(b"epub"), &[0]);

    let stats = scenario.run().await;

    assert_eq!(stats.already_current, 1);
    assert_eq!(scenario.processor.calls.load(Ordering::SeqCst), 0);
    assert_eq!(scenario.storage.uploads.load(Ordering::SeqCst), 0);
    assert!(
        scenario.storage.json(&toc_key)["metadata"]
            .get("chapters")
            .is_none()
    );
    assert_eq!(scenario.formats.format(document_id), format(2, None));
}

#[tokio::test]
async fn missing_expected_chapter_marks_needs_reprocess() {
    let scenario = Scenario::new(&[0, 1], false);
    let (document_id, toc_key) = scenario.book(Some(b"epub"), &[0]);

    let stats = scenario.run().await;

    assert_eq!(stats.needs_reprocess, 1);
    assert_eq!(
        scenario.formats.format(document_id),
        format(1, Some(EpubFormatIssue::MissingChapter))
    );
    assert!(
        scenario.storage.json(&toc_key)["metadata"]
            .get("chapters")
            .is_none()
    );
}

#[tokio::test]
async fn missing_toc_marks_needs_reprocess_and_is_not_retried() {
    let scenario = Scenario::new(&[0], false);
    let (document_id, toc_key) = scenario.book(Some(b"epub"), &[0]);
    scenario.storage.delete(&toc_key).await.unwrap();

    let stats = scenario.run().await;

    assert_eq!(stats.needs_reprocess, 1);
    assert_eq!(
        scenario.formats.format(document_id),
        format(1, Some(EpubFormatIssue::MissingChapter))
    );
    assert_eq!(scenario.run().await, EpubChapterBackfillStats::default());
}

#[tokio::test]
async fn unrelated_extra_object_is_ignored() {
    let scenario = Scenario::new(&[0, 2], false);
    let (document_id, toc_key) = scenario.book(Some(b"epub"), &[0, 2, 7]);

    let stats = scenario.run().await;

    assert_eq!(stats.upgraded, 1);
    assert_eq!(scenario.formats.format(document_id), format(2, None));
    assert_eq!(
        scenario.storage.json(&toc_key)["metadata"]["chapters"],
        chapter_list(&[0, 2])
    );
}

#[tokio::test]
async fn book_without_original_marks_needs_reupload_and_is_not_retried() {
    let scenario = Scenario::new(&[0], false);
    let (missing, _) = scenario.book(None, &[0]);
    let (unparseable, _) = scenario.book(Some(b"bad"), &[0]);

    let stats = scenario.run().await;

    assert_eq!(stats.needs_reupload, 2);
    assert_eq!(scenario.processor.calls.load(Ordering::SeqCst), 1);
    for document_id in [missing, unparseable] {
        assert_eq!(
            scenario.formats.format(document_id),
            format(1, Some(EpubFormatIssue::NeedsReupload))
        );
    }
    assert_eq!(scenario.run().await, EpubChapterBackfillStats::default());
}

#[tokio::test]
async fn storage_error_is_retried_then_left_pending() {
    let scenario = Scenario::new(&[0], false);
    let (document_id, _) = scenario.book(Some(b"epub"), &[0]);
    scenario
        .storage
        .failing_reads
        .store(u32::MAX, Ordering::SeqCst);

    let stats = scenario.run().await;

    assert_eq!(stats.left_pending, 1);
    assert_eq!(scenario.storage.reads.load(Ordering::SeqCst), 3);
    assert_eq!(scenario.formats.format(document_id), format(1, None));

    scenario.storage.failing_reads.store(0, Ordering::SeqCst);
    assert_eq!(scenario.run().await.upgraded, 1);
}
