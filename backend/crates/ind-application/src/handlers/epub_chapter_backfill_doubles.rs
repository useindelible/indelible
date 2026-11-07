use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use bytes::Bytes;
use futures::future::BoxFuture;
use ind_domain::{ArchiveAssetId, ArchiveAssetKind, ArchiveAssetStatus, DocumentId, DocumentType};
use serde_json::{Value, json};

use crate::error::AppError;
use crate::ports::{
    FileUploadProcessor, ProcessedUpload, ProcessedUploadAsset, UploadFileProcessRequest,
};
use crate::repos::epub_format::{
    EpubAssetFormat, EpubFormatIssue, EpubFormatRepository, PendingEpubAsset,
};
use crate::storage::{ObjectData, ObjectStorage, UploadResult};

pub(super) struct Row {
    pub(super) asset: PendingEpubAsset,
    pub(super) format: EpubAssetFormat,
}

#[derive(Default)]
pub(super) struct FakeFormats {
    pub(super) rows: Mutex<Vec<Row>>,
    pub(super) reprocess_holds_lock_first: bool,
}

impl FakeFormats {
    fn pending(&self, keep: impl Fn(&PendingEpubAsset) -> bool) -> Vec<PendingEpubAsset> {
        let rows = self.rows.lock().unwrap();
        rows.iter()
            .filter(|row| row.format.version < 2 && row.format.issue.is_none() && keep(&row.asset))
            .map(|row| row.asset.clone())
            .collect()
    }

    pub(super) fn format(&self, document_id: DocumentId) -> EpubAssetFormat {
        let rows = self.rows.lock().unwrap();
        rows.iter()
            .find(|row| row.asset.document_id == document_id)
            .unwrap()
            .format
    }

    fn update(&self, asset_id: ArchiveAssetId, change: impl FnOnce(&mut EpubAssetFormat)) {
        let mut rows = self.rows.lock().unwrap();
        change(
            &mut rows
                .iter_mut()
                .find(|row| row.asset.asset_id == asset_id)
                .unwrap()
                .format,
        );
    }
}

#[async_trait::async_trait]
impl EpubFormatRepository for FakeFormats {
    async fn epub_assets_pending_after(
        &self,
        cursor: Option<ArchiveAssetId>,
        limit: i64,
    ) -> Result<Vec<PendingEpubAsset>, AppError> {
        let after = |asset: &PendingEpubAsset| {
            cursor.is_none_or(|cursor| asset.asset_id.into_uuid() > cursor.into_uuid())
        };
        let mut pending = self.pending(after);
        pending.sort_by_key(|asset| asset.asset_id.into_uuid());
        pending.truncate(usize::try_from(limit).unwrap());
        Ok(pending)
    }

    async fn pending_epub_asset(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<PendingEpubAsset>, AppError> {
        Ok(self.pending(|asset| asset.document_id == document_id).pop())
    }

    async fn find_epub_format(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<EpubAssetFormat>, AppError> {
        Ok(Some(self.format(document_id)))
    }

    async fn mark_format(&self, id: ArchiveAssetId, version: i32, _: i64) -> Result<(), AppError> {
        self.update(id, |format| {
            *format = EpubAssetFormat {
                version,
                issue: None,
            }
        });
        Ok(())
    }

    async fn mark_format_issue(
        &self,
        id: ArchiveAssetId,
        issue: EpubFormatIssue,
    ) -> Result<(), AppError> {
        self.update(id, |format| format.issue = Some(issue));
        Ok(())
    }

    async fn with_document_lock<'a>(
        &'a self,
        document_id: DocumentId,
        work: BoxFuture<'a, Result<(), AppError>>,
    ) -> Result<(), AppError> {
        if self.reprocess_holds_lock_first {
            let mut rows = self.rows.lock().unwrap();
            for row in rows
                .iter_mut()
                .filter(|row| row.asset.document_id == document_id)
            {
                row.format.version = 2;
            }
        }
        work.await
    }

    async fn try_global_backfill_lock<'a>(
        &'a self,
        work: BoxFuture<'a, Result<(), AppError>>,
    ) -> Result<bool, AppError> {
        work.await.map(|()| true)
    }
}

#[derive(Default)]
pub(super) struct FakeStorage {
    objects: Mutex<HashMap<String, Bytes>>,
    pub(super) failing_reads: AtomicU32,
    pub(super) reads: AtomicU32,
    pub(super) uploads: AtomicU32,
}

impl FakeStorage {
    pub(super) fn put(&self, key: &str, bytes: impl Into<Bytes>) {
        self.objects
            .lock()
            .unwrap()
            .insert(key.into(), bytes.into());
    }

    pub(super) fn json(&self, key: &str) -> Value {
        serde_json::from_slice(&self.objects.lock().unwrap()[key]).unwrap()
    }
}

#[async_trait::async_trait]
impl ObjectStorage for FakeStorage {
    async fn upload(&self, key: &str, _: &str, body: Bytes) -> Result<UploadResult, AppError> {
        self.uploads.fetch_add(1, Ordering::SeqCst);
        let size_bytes = i64::try_from(body.len()).unwrap();
        self.put(key, body);
        Ok(UploadResult {
            key: key.into(),
            bucket: "books".into(),
            size_bytes,
        })
    }

    async fn presigned_url(&self, _: &str, _: Duration) -> Result<String, AppError> {
        Err(AppError::Repository("unused".into()))
    }

    async fn delete(&self, key: &str) -> Result<(), AppError> {
        self.objects.lock().unwrap().remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, AppError> {
        Ok(self.objects.lock().unwrap().contains_key(key))
    }

    async fn get_object(&self, key: &str) -> Result<ObjectData, AppError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        if self.failing_reads.load(Ordering::SeqCst) > 0 {
            self.failing_reads.fetch_sub(1, Ordering::SeqCst);
            return Err(AppError::Repository("storage unavailable".into()));
        }
        let bytes = self
            .objects
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or_else(|| AppError::Repository(format!("no object at {key}").into()))?;
        let content_length = i64::try_from(bytes.len()).unwrap();
        Ok(ObjectData {
            body: Box::pin(futures::stream::iter([Ok(bytes)])),
            content_type: "application/octet-stream".into(),
            content_length,
        })
    }
}

/// Stands in for the EPUB parser: a book whose original is `b"bad"` fails to parse.
pub(super) struct FakeProcessor {
    pub(super) spine_indices: Vec<usize>,
    pub(super) calls: AtomicU32,
}

#[async_trait::async_trait]
impl FileUploadProcessor for FakeProcessor {
    async fn process_upload(
        &self,
        request: UploadFileProcessRequest,
    ) -> Result<ProcessedUpload, AppError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if request.data.as_ref() == b"bad" {
            return Err(AppError::Repository("not an EPUB".into()));
        }
        let toc = json!({ "metadata": { "chapters": chapter_list(&self.spine_indices) } });
        Ok(ProcessedUpload {
            document_type: DocumentType::Book,
            original_extension: "epub",
            title: "Book".into(),
            author: None,
            word_count: None,
            reading_time_minutes: None,
            assets: vec![ProcessedUploadAsset {
                asset_kind: Some(ArchiveAssetKind::Epub),
                filename: "epub_toc.json".into(),
                content_type: "application/json".into(),
                bytes: Bytes::from(serde_json::to_vec(&toc).unwrap()),
                status: ArchiveAssetStatus::Completed,
                failed_reason: None,
                format_version: 2,
            }],
        })
    }
}

pub(super) fn chapter_list(spine_indices: &[usize]) -> Value {
    let chapters = spine_indices.iter().map(
        |index| json!({ "spine_index": index, "href": format!("ch{index}.xhtml"), "title": "" }),
    );
    Value::Array(chapters.collect())
}
