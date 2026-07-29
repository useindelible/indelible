use chrono::Utc;
use sqlx::PgPool;

use ind_application::AppError;
use ind_application::event_intents;
use ind_application::repos::document_lifecycle::SaveToLibraryOutcome;
use ind_application::repos::document_upload::{
    DocumentUploadRepository, SaveUploadedDocumentRequest, StagedDocumentAsset,
};
use ind_application::repos::lifecycle_outbox::{
    document_ai_processing_outbox, search_reindex_document_outbox,
};
use ind_domain::{ArchiveAssetId, ArchiveAssetKind, ArchiveAssetStatus, Document, DocumentId};

use super::document_lifecycle_repo::steps::materialize_document_tx;
use super::document_repo::rows::DocumentRow;
use super::document_repo::tx_writes::PgTx;
use super::library_repo::tx_writes::insert_or_restore_library_entry_tx;
use super::write_helpers::{apply_domain_events_tx, apply_outbox_tx};

pub struct PgDocumentUploadRepository {
    pool: PgPool,
}

impl PgDocumentUploadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DocumentUploadRepository for PgDocumentUploadRepository {
    async fn save_uploaded_document(
        &self,
        request: SaveUploadedDocumentRequest,
    ) -> Result<SaveToLibraryOutcome, AppError> {
        save_uploaded_document(&self.pool, request).await
    }
}

async fn save_uploaded_document(
    pool: &PgPool,
    request: SaveUploadedDocumentRequest,
) -> Result<SaveToLibraryOutcome, AppError> {
    let SaveUploadedDocumentRequest {
        identity,
        source,
        assets,
        word_count,
        reading_time_minutes,
        asset_base_url,
    } = request;

    let mut tx = pool
        .begin()
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

    let (mut document, document_created) = materialize_document_tx(&mut tx, &identity).await?;
    let upsert = insert_or_restore_library_entry_tx(
        &mut tx,
        document.user_id,
        document.id,
        source,
        None,
        Default::default(),
    )
    .await?;

    let thumbnail_url = assets
        .iter()
        .any(|asset| asset.asset_kind == ArchiveAssetKind::Thumbnail)
        .then(|| document_thumbnail_url(&asset_base_url, document.id));

    for asset in assets {
        upsert_staged_asset_tx(&mut tx, document.id, asset).await?;
    }

    if word_count.is_some() || reading_time_minutes.is_some() || thumbnail_url.is_some() {
        document = set_document_upload_metadata_tx(
            &mut tx,
            document.user_id,
            document.id,
            word_count,
            reading_time_minutes,
            thumbnail_url.as_deref(),
        )
        .await?;
    }

    let now = Utc::now();
    let events = vec![event_intents::library_entry_saved(
        document.user_id,
        upsert.entry.id,
        document.id,
        source,
    )];
    let mut outbox = vec![search_reindex_document_outbox(document.id, now)];
    outbox.extend(document_ai_processing_outbox(document.id, now));

    apply_domain_events_tx(&mut tx, events).await?;
    apply_outbox_tx(&mut tx, &outbox).await?;

    tx.commit()
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

    Ok(SaveToLibraryOutcome {
        document,
        document_created,
        entry: upsert.entry,
        restored: upsert.restored,
        skipped_restore: false,
        already_active: upsert.already_active,
        backlinked_deliveries: 0,
        hidden_deliveries: 0,
    })
}

pub(crate) async fn upsert_staged_asset_tx(
    tx: &mut PgTx<'_>,
    document_id: DocumentId,
    asset: StagedDocumentAsset,
) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO archive_assets \
            (id, document_id, asset_kind, s3_key, s3_bucket, content_type, size_bytes, \
             status, failed_reason, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now()) \
         ON CONFLICT (document_id, asset_kind) WHERE document_id IS NOT NULL DO UPDATE SET \
             s3_key = EXCLUDED.s3_key, s3_bucket = EXCLUDED.s3_bucket, \
             content_type = EXCLUDED.content_type, size_bytes = EXCLUDED.size_bytes, \
             status = EXCLUDED.status, failed_reason = EXCLUDED.failed_reason, created_at = now()",
        ArchiveAssetId::new().into_uuid(),
        document_id.into_uuid(),
        asset.asset_kind.to_string(),
        asset.s3_key,
        asset.s3_bucket,
        asset.content_type,
        asset.size_bytes,
        asset_status_to_str(asset.status),
        asset.failed_reason,
    )
    .execute(&mut **tx)
    .await
    .map_err(|err| super::map_sqlx_error("document_asset", "document asset conflict", err))?;
    Ok(())
}

pub(crate) async fn set_document_upload_metadata_tx(
    tx: &mut PgTx<'_>,
    user_id: ind_domain::UserId,
    document_id: DocumentId,
    word_count: Option<i32>,
    reading_time_minutes: Option<i32>,
    thumbnail_url: Option<&str>,
) -> Result<Document, AppError> {
    let row = sqlx::query_as!(
        DocumentRow,
        "UPDATE documents \
         SET word_count = COALESCE($3, word_count), \
             reading_time_minutes = COALESCE($4, reading_time_minutes), \
             thumbnail_url = COALESCE($5, thumbnail_url), \
             updated_at = now() \
         WHERE user_id = $1 AND id = $2 \
         RETURNING id, user_id, document_type, canonical_url, original_url, content_hash, \
                   title, author, excerpt, published_at, language, domain, lead_image_url, \
                   thumbnail_url, word_count, reading_time_minutes, created_at, updated_at",
        user_id.into_uuid(),
        document_id.into_uuid(),
        word_count,
        reading_time_minutes,
        thumbnail_url,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|err| super::map_sqlx_error("Document", "document metrics update failed", err))?;

    row.into_document()
}

fn document_thumbnail_url(asset_base_url: &str, document_id: DocumentId) -> String {
    format!(
        "{}/api/v1/assets/documents/{document_id}/thumbnail",
        asset_base_url.trim_end_matches('/')
    )
}

fn asset_status_to_str(status: ArchiveAssetStatus) -> &'static str {
    match status {
        ArchiveAssetStatus::Pending => "pending",
        ArchiveAssetStatus::Completed => "completed",
        ArchiveAssetStatus::Degraded => "degraded",
        ArchiveAssetStatus::Failed => "failed",
        ArchiveAssetStatus::Unsupported => "unsupported",
    }
}
