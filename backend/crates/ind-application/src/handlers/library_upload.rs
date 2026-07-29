use std::collections::HashMap;
use std::sync::Arc;

use futures::future::BoxFuture;
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::ports::{
    FileUploadProcessor, LibraryUploadOperations, UploadFileProcessRequest, UploadFileRequest,
};
use crate::repos::document_lifecycle::{
    MaterializeIdentity, MaterializeOrigin, SaveToLibraryOutcome,
};
use crate::repos::document_upload::{
    DocumentUploadRepository, SaveUploadedDocumentRequest, StagedDocumentAsset,
};
use crate::storage::ObjectStorage;
use ind_domain::{
    ArchiveAssetKind, ContentSource, DocumentId, DocumentOriginType, NewOriginDocument, UserId,
    deterministic_origin_id,
};

pub struct LibraryUploadService {
    processor: Arc<dyn FileUploadProcessor>,
    storage: Arc<dyn ObjectStorage>,
    uploads: Arc<dyn DocumentUploadRepository>,
    document_repo: Arc<dyn crate::repos::document::DocumentRepository>,
    document_asset_repo: Arc<dyn crate::repos::document_asset::DocumentAssetRepository>,
}

impl LibraryUploadService {
    pub fn new(
        processor: Arc<dyn FileUploadProcessor>,
        storage: Arc<dyn ObjectStorage>,
        uploads: Arc<dyn DocumentUploadRepository>,
        document_repo: Arc<dyn crate::repos::document::DocumentRepository>,
        document_asset_repo: Arc<dyn crate::repos::document_asset::DocumentAssetRepository>,
    ) -> Self {
        Self {
            processor,
            storage,
            uploads,
            document_repo,
            document_asset_repo,
        }
    }

    pub async fn upload_file(
        &self,
        user_id: UserId,
        req: UploadFileRequest,
    ) -> Result<SaveToLibraryOutcome, AppError> {
        let content_hash = sha256_hex(&req.data);
        let processed = self
            .processor
            .process_upload(UploadFileProcessRequest {
                filename: req.filename,
                content_type: req.content_type,
                data: req.data,
                title_override: req.title_override,
                max_bytes: req.max_bytes,
            })
            .await?;

        let prefix = format!("documents/uploads/{user_id}/{content_hash}/");
        let mut uploaded_by_filename: HashMap<String, (String, String, i64)> = HashMap::new();
        let mut staged_assets = Vec::new();
        for object in processed.assets {
            let (key, bucket, size_bytes) = if object.bytes.is_empty()
                && object.status != ind_domain::ArchiveAssetStatus::Completed
            {
                (String::new(), String::new(), 0)
            } else {
                if let Some(existing) = uploaded_by_filename.get(&object.filename) {
                    existing.clone()
                } else {
                    let key = format!("{prefix}{}", object.filename);
                    let upload = self
                        .storage
                        .upload(&key, &object.content_type, object.bytes)
                        .await?;
                    let record = (upload.key, upload.bucket, upload.size_bytes);
                    uploaded_by_filename.insert(object.filename.clone(), record.clone());
                    record
                }
            };

            if let Some(asset_kind) = object.asset_kind {
                staged_assets.push(StagedDocumentAsset {
                    asset_kind,
                    s3_key: key,
                    s3_bucket: bucket,
                    content_type: object.content_type,
                    size_bytes,
                    status: object.status,
                    failed_reason: object.failed_reason,
                });
            }
        }

        let origin_id =
            deterministic_origin_id(DocumentOriginType::ManualUpload, user_id, &content_hash);
        let document_id = DocumentId::new();
        let thumbnail_url = staged_assets
            .iter()
            .any(|asset| asset.asset_kind == ArchiveAssetKind::Thumbnail)
            .then(|| document_thumbnail_url(&req.asset_base_url, document_id));
        let document = NewOriginDocument {
            id: document_id,
            user_id,
            document_type: processed.document_type,
            content_hash: Some(content_hash),
            original_url: None,
            title: processed.title,
            author: processed.author,
            excerpt: None,
            published_at: None,
            language: None,
            domain: None,
            lead_image_url: None,
            thumbnail_url,
            sender_id: None,
        };

        let has_readable = staged_assets
            .iter()
            .any(|asset| asset.asset_kind == ArchiveAssetKind::ReadableHtml);
        let outcome = self
            .uploads
            .save_uploaded_document(SaveUploadedDocumentRequest {
                identity: MaterializeIdentity::Origin {
                    document,
                    origin: MaterializeOrigin {
                        origin_type: DocumentOriginType::ManualUpload,
                        origin_id,
                    },
                },
                source: ContentSource::Manual,
                assets: staged_assets,
                word_count: processed.word_count,
                reading_time_minutes: processed.reading_time_minutes,
                asset_base_url: req.asset_base_url,
            })
            .await?;

        if has_readable {
            // The save may have deduplicated onto an existing document (same
            // content hash), so derive against the document the outcome names,
            // not the id staged above.
            super::article_toc::apply_article_toc(
                Some(self.storage.as_ref()),
                self.document_asset_repo.as_ref(),
                self.document_repo.as_ref(),
                outcome.document.id,
            )
            .await;
        }
        Ok(outcome)
    }
}

impl LibraryUploadOperations for LibraryUploadService {
    fn upload_file(
        &self,
        user_id: UserId,
        req: UploadFileRequest,
    ) -> BoxFuture<'_, Result<SaveToLibraryOutcome, AppError>> {
        Box::pin(LibraryUploadService::upload_file(self, user_id, req))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn document_thumbnail_url(asset_base_url: &str, document_id: DocumentId) -> String {
    format!(
        "{}/api/v1/assets/documents/{document_id}/thumbnail",
        asset_base_url.trim_end_matches('/')
    )
}
