//! Shared staging for provided-content saves (extension reader/full-archive, email-body ingest,
//! feed autosave). The browser/email-supplied bytes must be uploaded BEFORE the save transaction so
//! the in-tx `document.attach_provided_content` outbox row only references an object that already
//! exists, but the canonical asset key needs the `document_id` that is resolved only inside the
//! save tx. The staging key is therefore content-addressed (`sha256`) and user-scoped, not
//! document-keyed; the worker points the asset row at this key directly so no second copy is needed.
//! Content addressing also makes the upload idempotent across save retries, and an orphan object
//! from a save that crashed before commit is harmless.

use std::sync::Arc;

use bytes::Bytes;
use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::repos::lifecycle_outbox::{OutboxEntry, attach_provided_content_outbox};
use crate::storage::ObjectStorage;
use ind_domain::{ArchiveAssetKind, DocumentId, UserId};

#[derive(Debug, Clone)]
pub struct StagedProvidedContent {
    pub asset_kind: ArchiveAssetKind,
    pub storage_key: String,
    pub storage_bucket: String,
    pub content_type: String,
    pub size_bytes: i64,
}

impl StagedProvidedContent {
    /// Call inside the save `side_effects` closure with `ctx.document.id` once the document is
    /// resolved.
    pub fn outbox(&self, document_id: DocumentId, user_id: UserId) -> OutboxEntry {
        attach_provided_content_outbox(
            document_id,
            user_id,
            self.asset_kind,
            self.storage_key.clone(),
            self.storage_bucket.clone(),
            self.content_type.clone(),
            self.size_bytes,
            Utc::now(),
        )
    }
}

fn staging_key(user_id: UserId, kind: ArchiveAssetKind, bytes: &[u8]) -> String {
    let digest = hex::encode(Sha256::digest(bytes));
    format!(
        "documents/provided/{}/{}/{kind}.html",
        user_id.into_uuid(),
        digest
    )
}

pub async fn stage_provided_content(
    storage: &Arc<dyn ObjectStorage>,
    user_id: UserId,
    kind: ArchiveAssetKind,
    content_type: &str,
    bytes: Bytes,
) -> Result<StagedProvidedContent, AppError> {
    // Provided ReadableHtml is untrusted third-party content (browser-extracted article HTML)
    // that is later served inline in the reader; sanitize it before it rests in storage and
    // prepare its anchors for ToC navigation. Other kinds (Monolith full-page archives) are
    // stored verbatim so the capture stays faithful. The sanitize fallback keeps the capture
    // alive when anchor preparation fails.
    let bytes = if kind == ArchiveAssetKind::ReadableHtml {
        let raw = String::from_utf8_lossy(&bytes);
        let prepared = ind_html::prepare_reader_html(&raw).unwrap_or_else(|err| {
            tracing::warn!(error = %err, "anchor preparation failed; storing sanitized only");
            ind_html::sanitize_reader_html(&raw)
        });
        Bytes::from(prepared.into_bytes())
    } else {
        bytes
    };
    let key = staging_key(user_id, kind, &bytes);
    let upload = storage.upload(&key, content_type, bytes).await?;
    Ok(StagedProvidedContent {
        asset_kind: kind,
        storage_key: upload.key,
        storage_bucket: upload.bucket,
        content_type: content_type.to_string(),
        size_bytes: upload.size_bytes,
    })
}
