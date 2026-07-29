use std::sync::Arc;

use futures::StreamExt;
use ind_application::AppError;
use ind_application::storage::ObjectStorage;
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, DocumentAsset};

pub(super) fn find_asset(
    assets: &[DocumentAsset],
    kind: ArchiveAssetKind,
) -> Option<&DocumentAsset> {
    assets.iter().find(|a| {
        a.asset_kind == kind
            && a.status == ArchiveAssetStatus::Completed
            && !a.s3_key.trim().is_empty()
    })
}

pub(super) async fn load_asset_bytes(
    object_storage: Option<&Arc<dyn ObjectStorage>>,
    key: &str,
) -> Result<Vec<u8>, AppError> {
    let storage = object_storage.ok_or_else(|| AppError::ExternalService {
        service: "storage".into(),
        message: "storage not configured".into(),
    })?;
    let object = storage.get_object(key).await?;
    let mut stream = object.body;
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|err| AppError::Repository(Box::new(err)))?;
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

pub(super) async fn load_text_asset(
    object_storage: Option<&Arc<dyn ObjectStorage>>,
    key: &str,
) -> Result<String, AppError> {
    let bytes = load_asset_bytes(object_storage, key).await?;
    String::from_utf8(bytes).map_err(|err| AppError::Repository(Box::new(err)))
}
