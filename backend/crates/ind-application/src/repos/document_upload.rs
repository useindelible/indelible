use crate::error::AppError;
use crate::repos::document_lifecycle::{MaterializeIdentity, SaveToLibraryOutcome};
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, ContentSource};

/// A document asset already staged in object storage. The upload repository attaches these rows
/// to the resolved document id inside the save transaction.
#[derive(Debug, Clone)]
pub struct StagedDocumentAsset {
    pub asset_kind: ArchiveAssetKind,
    pub s3_key: String,
    pub s3_bucket: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub status: ArchiveAssetStatus,
    pub failed_reason: Option<String>,
}

pub struct SaveUploadedDocumentRequest {
    pub identity: MaterializeIdentity,
    pub source: ContentSource,
    pub assets: Vec<StagedDocumentAsset>,
    pub word_count: Option<i32>,
    pub reading_time_minutes: Option<i32>,
    pub asset_base_url: String,
}

/// Atomic persistence boundary for manual file uploads: materialize/find the no-URL document,
/// insert/restore Library membership, attach document-owned archive assets, persist reading
/// metrics, and enqueue search/Mila work in one transaction.
#[async_trait::async_trait]
pub trait DocumentUploadRepository: Send + Sync {
    async fn save_uploaded_document(
        &self,
        request: SaveUploadedDocumentRequest,
    ) -> Result<SaveToLibraryOutcome, AppError>;
}
