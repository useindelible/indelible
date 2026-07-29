use ind_application::renderer::RenderResult;
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, DocumentId, NewDocumentAsset};

/// Map a render result's successful artifacts to document-keyed assets for the
/// document/feed/library preparation path. Unknown artifact kinds are skipped (the worker
/// only requests readable_html for prefetch).
pub fn build_document_assets(
    document_id: DocumentId,
    s3_bucket: &str,
    result: &RenderResult,
) -> Vec<NewDocumentAsset> {
    let mut assets = Vec::new();

    for artifact in &result.artifacts {
        let Ok(asset_kind) = artifact.kind.parse::<ArchiveAssetKind>() else {
            tracing::warn!(kind = artifact.kind, "unknown artifact kind, skipping");
            continue;
        };

        assets.push(NewDocumentAsset {
            document_id,
            asset_kind,
            s3_key: artifact.s3_key.clone(),
            s3_bucket: s3_bucket.to_string(),
            content_type: artifact.content_type.clone(),
            size_bytes: artifact.size_bytes,
            status: ArchiveAssetStatus::Completed,
            failed_reason: None,
        });
    }

    assets
}
