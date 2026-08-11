use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;

use ind_application::AppError;
use ind_application::repos::document::DocumentRepository;
use ind_application::repos::document_asset::DocumentAssetRepository;
use ind_application::repos::mila_config::MilaConfigRepository;
use ind_application::repos::prepared_content::PreparedContentProvider;
use ind_application::storage::ObjectStorage;
use ind_application::text::ChunkingConfig;

use ind_domain::{
    ArchiveAssetKind, ArchiveAssetStatus, Document, DocumentAsset, DocumentId, NewDocumentAsset,
    PreparedItemContent,
};
use ind_html::html_to_markdown;

const FALLBACK_CHUNK_SIZE: usize = 512;
const FALLBACK_CHUNK_OVERLAP: usize = 64;

pub(super) struct HandlerContext<'a> {
    pub(super) document: &'a Document,
    pub(super) assets: &'a [DocumentAsset],
    pub(super) asset_repo: &'a dyn DocumentAssetRepository,
    pub(super) object_storage: Option<&'a Arc<dyn ObjectStorage>>,
    pub(super) chunking: ChunkingConfig,
}

pub struct AssetBackedPreparedContentProvider {
    document_repo: Arc<dyn DocumentRepository>,
    document_asset_repo: Arc<dyn DocumentAssetRepository>,
    mila_config_repo: Arc<dyn MilaConfigRepository>,
    object_storage: Option<Arc<dyn ObjectStorage>>,
}

impl AssetBackedPreparedContentProvider {
    pub fn new(
        document_repo: Arc<dyn DocumentRepository>,
        document_asset_repo: Arc<dyn DocumentAssetRepository>,
        mila_config_repo: Arc<dyn MilaConfigRepository>,
        object_storage: Option<Arc<dyn ObjectStorage>>,
    ) -> Self {
        Self {
            document_repo,
            document_asset_repo,
            mila_config_repo,
            object_storage,
        }
    }

    pub async fn load_readable_html_markdown_for_document(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<String>, AppError> {
        let Some(_document) = self.document_repo.find_by_id_global(document_id).await? else {
            return Ok(None);
        };

        let Some(asset) = self
            .document_asset_repo
            .find_by_document_and_kind(document_id, ArchiveAssetKind::ReadableHtml)
            .await?
        else {
            return Ok(None);
        };
        if asset.status != ArchiveAssetStatus::Completed || asset.content_type != "text/html" {
            return Ok(None);
        }

        let html = load_text_asset(self.object_storage.as_ref(), &asset.s3_key).await?;
        let markdown = html_to_markdown(&html);
        if markdown.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(markdown))
        }
    }
}

#[async_trait]
impl PreparedContentProvider for AssetBackedPreparedContentProvider {
    async fn load_for_document(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<PreparedItemContent>, AppError> {
        let Some(document) = self.document_repo.find_by_id_global(document_id).await? else {
            return Ok(None);
        };

        let config = self.mila_config_repo.get_by_user(document.user_id).await?;
        let assets = self
            .document_asset_repo
            .find_by_document(document_id)
            .await?;

        let ctx = HandlerContext {
            document: &document,
            assets: &assets,
            asset_repo: self.document_asset_repo.as_ref(),
            object_storage: self.object_storage.as_ref(),
            chunking: config
                .as_ref()
                .map(|config| ChunkingConfig {
                    chunk_size: config.chunk_size.max(1) as usize,
                    chunk_overlap: config.chunk_overlap.max(0) as usize,
                })
                .unwrap_or(ChunkingConfig {
                    chunk_size: FALLBACK_CHUNK_SIZE,
                    chunk_overlap: FALLBACK_CHUNK_OVERLAP,
                }),
        };

        if let Some(result) = try_readable_html(&ctx).await? {
            return Ok(Some(result));
        }
        if let Some(result) = try_epub(&ctx).await? {
            return Ok(Some(result));
        }
        if let Some(result) = try_pdf(&ctx).await? {
            return Ok(Some(result));
        }

        Ok(None)
    }

    async fn load_readable_text_for_document(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<String>, AppError> {
        let content = self.load_for_document(document_id).await?;
        Ok(content.and_then(|content| {
            (!content.root_text.trim().is_empty()).then_some(content.root_text)
        }))
    }
}

mod assets;
mod epub;
mod pdf;
mod root;

use assets::load_text_asset;
use epub::try_epub;
use pdf::try_pdf;
use root::try_readable_html;

pub(super) async fn upsert_extracted_text_asset(
    ctx: &HandlerContext<'_>,
    text: Option<&str>,
    status: ArchiveAssetStatus,
    failed_reason: Option<String>,
) -> Result<DocumentAsset, AppError> {
    let (s3_key, s3_bucket, size_bytes) =
        if let Some(text) = text.filter(|text| !text.trim().is_empty()) {
            let storage = ctx
                .object_storage
                .ok_or_else(|| AppError::ExternalService {
                    service: "storage".into(),
                    message: "storage not configured".into(),
                })?;
            let key = format!(
                "documents/{}/{}/extracted.txt",
                ctx.document.user_id.into_uuid(),
                ctx.document.id.into_uuid()
            );
            let upload = storage
                .upload(&key, "text/plain", Bytes::from(text.to_owned()))
                .await?;
            (upload.key, upload.bucket, upload.size_bytes)
        } else {
            (String::new(), String::new(), 0)
        };

    ctx.asset_repo
        .upsert_document_asset(NewDocumentAsset {
            document_id: ctx.document.id,
            asset_kind: ArchiveAssetKind::ExtractedText,
            s3_key,
            s3_bucket,
            content_type: "text/plain".into(),
            size_bytes,
            status,
            failed_reason,
        })
        .await
}
