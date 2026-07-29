//! Worker driver for the `document.attach_provided_content` outbox row committed by the
//! provided-content saves (see `ind_application::handlers::provided_content` for the staging
//! design). Idempotently inserts the `archive_assets` row pointing at the already-uploaded
//! `storage_key`, and for the readable asset enqueues search reindex and the content-gated embed.

use futures::StreamExt;
use ind_application::AppError;
use ind_domain::{
    ArchiveAssetKind, ArchiveAssetStatus, AttachProvidedContentJob, DomainError, NewDocumentAsset,
};

use crate::context::CaptureJobDeps;
use crate::jobs::ai::enqueue_document_embed_if_engaged;
use crate::jobs::reading_metrics::{apply_reading_metrics, word_count_from_html};
use crate::jobs::search::enqueue_search_reindex_document;

pub async fn handle_attach_provided_content(
    ctx: &CaptureJobDeps,
    job: AttachProvidedContentJob,
) -> Result<(), AppError> {
    // Verify the document still exists for this owner before attaching (the save committed it in
    // the same transaction as this job, so a missing row means the document was since deleted).
    if ctx
        .document_repo
        .find_by_id(job.user_id, job.document_id)
        .await?
        .is_none()
    {
        return Err(AppError::Domain(DomainError::NotFound {
            entity: "Document",
            id: job.document_id.to_string(),
        }));
    }

    let already_attached = ctx
        .document_asset_repo
        .has_successful_asset(job.document_id, job.asset_kind)
        .await?;

    if !already_attached {
        ctx.document_asset_repo
            .upsert_document_asset(NewDocumentAsset {
                document_id: job.document_id,
                asset_kind: job.asset_kind,
                s3_key: job.storage_key.clone(),
                s3_bucket: job.storage_bucket,
                content_type: job.content_type,
                size_bytes: job.size_bytes,
                status: ArchiveAssetStatus::Completed,
                failed_reason: None,
            })
            .await?;
    }

    // Only the readable asset drives content indexing; the monolith/original captures are archival.
    // The document is already engaged (just saved), so the embed is content-gated on the readable
    // asset that now exists. Both jobs dedupe per document.
    if job.asset_kind == ArchiveAssetKind::ReadableHtml {
        // Single choke point for reading metrics on provided content: email ingest, extension
        // reader/full-archive saves, and feed autosave all stage their readable HTML through
        // this job, so counting the staged object here covers every path without each save
        // handler re-implementing it. Best-effort: a metrics failure never blocks the attach.
        match load_object_string(ctx, &job.storage_key).await {
            Ok(html) => {
                apply_reading_metrics(
                    ctx.document_repo.as_ref(),
                    job.user_id,
                    job.document_id,
                    word_count_from_html(&html),
                )
                .await;
            }
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    document_id = %job.document_id,
                    "failed to load readable content for reading metrics"
                );
            }
        }
        enqueue_search_reindex_document(ctx, job.document_id).await?;
        enqueue_document_embed_if_engaged(ctx, job.user_id, job.document_id).await?;
        ind_application::handlers::article_toc::apply_article_toc(
            ctx.object_storage.as_deref(),
            ctx.document_asset_repo.as_ref(),
            ctx.document_repo.as_ref(),
            job.document_id,
        )
        .await;
    }

    Ok(())
}

async fn load_object_string(ctx: &CaptureJobDeps, key: &str) -> Result<String, AppError> {
    let storage = ctx
        .object_storage
        .as_ref()
        .ok_or_else(|| AppError::ExternalService {
            service: "storage".into(),
            message: "object storage not configured".into(),
        })?;
    let object = storage.get_object(key).await?;
    let mut stream = object.body;
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|err| AppError::Repository(Box::new(err)))?;
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|err| AppError::Repository(Box::new(err)))
}
