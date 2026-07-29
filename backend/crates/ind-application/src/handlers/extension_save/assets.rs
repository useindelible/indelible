use crate::error::AppError;
use crate::handlers::provided_content::StagedProvidedContent;
use ind_domain::{ArchiveAssetStatus, DocumentId, NewDocumentAsset};

use super::ExtensionSaveService;

impl ExtensionSaveService {
    pub(super) async fn attach_staged_document_asset(
        &self,
        document_id: DocumentId,
        staged: &StagedProvidedContent,
    ) -> Result<(), AppError> {
        if self
            .document_asset_repo
            .has_successful_asset(document_id, staged.asset_kind)
            .await?
        {
            return Ok(());
        }

        self.document_asset_repo
            .upsert_document_asset(NewDocumentAsset {
                document_id,
                asset_kind: staged.asset_kind,
                s3_key: staged.storage_key.clone(),
                s3_bucket: staged.storage_bucket.clone(),
                content_type: staged.content_type.clone(),
                size_bytes: staged.size_bytes,
                status: ArchiveAssetStatus::Completed,
                failed_reason: None,
            })
            .await?;

        Ok(())
    }
}
