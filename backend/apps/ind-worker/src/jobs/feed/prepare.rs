use ind_application::AppError;
use ind_application::renderer::RenderUrlRequest;
use ind_application::repos::document::DocumentRenderedMetadata;
use ind_domain::{
    ArchiveAssetKind, ArchiveAssetStatus, ItemId, NewDocumentAsset, PrepareDocumentJob,
};

use crate::context::FeedJobDeps;
use crate::jobs::ai::enqueue_document_embed_if_engaged;
use crate::jobs::search::enqueue_search_reindex_document;

/// Readable-content preparation for a feed-discovered document (docs/document-feed-library-
/// architecture.md, Readable Content Preparation Policy). Renders the canonical URL and writes
/// the result to `archive_assets(document_id)`. The feed's inline content is never used; a
/// missing/failed readable asset is a retryable failure, not a silent success.
pub async fn handle_prepare_document(
    ctx: &FeedJobDeps,
    job: PrepareDocumentJob,
) -> Result<(), AppError> {
    let archival = ctx
        .user_preferences_repo
        .get_archival(job.user_id)
        .await?
        .unwrap_or_default();
    let existing_assets = ctx
        .document_asset_repo
        .find_by_document(job.document_id)
        .await?;
    let has_completed = |kind: ArchiveAssetKind| {
        existing_assets
            .iter()
            .any(|asset| asset.asset_kind == kind && asset.status == ArchiveAssetStatus::Completed)
    };

    let mut outputs = Vec::new();
    if !has_completed(ArchiveAssetKind::ReadableHtml) {
        outputs.push(ArchiveAssetKind::ReadableHtml.to_string());
    }
    if archival.archive_formats.monolith && !has_completed(ArchiveAssetKind::Monolith) {
        outputs.push(ArchiveAssetKind::Monolith.to_string());
    }
    if archival.archive_formats.pdf && !has_completed(ArchiveAssetKind::Pdf) {
        outputs.push(ArchiveAssetKind::Pdf.to_string());
    }
    if archival.archive_formats.screenshot && !has_completed(ArchiveAssetKind::Screenshot) {
        outputs.push(ArchiveAssetKind::Screenshot.to_string());
    }

    // Post-completion idempotency: a prior prepare (read-ahead or on-tap) may already have
    // rendered every enabled asset. The dedupe_key collapses pending duplicates; this skips
    // re-renders once all requested assets exist.
    if outputs.is_empty() {
        // Ensure durable search exists even when the render was already done (e.g. a re-tap
        // after read-ahead prepared it). The dedupe_key collapses duplicate enqueues.
        enqueue_search_reindex_document(ctx, job.document_id).await?;
        enqueue_document_embed_if_engaged(ctx, job.user_id, job.document_id).await?;
        return Ok(());
    }

    // A YouTube URL must be transcript-ingested, not archived as its watch page. Route it to
    // document.youtube_ingest instead of the generic render. This is the single choke point every
    // feed delivery / read-ahead / extension quick_save prepare passes through; the dedupe key
    // collapses overlapping enqueues.
    if ind_application::dispatch::is_youtube_url(&job.url) {
        crate::jobs::youtube::enqueue_youtube_ingest_document(
            ctx,
            job.user_id,
            job.document_id,
            &job.url,
        )
        .await?;
        return Ok(());
    }

    // The renderer namespaces storage by the subject UUID; the document id is that subject for
    // document-keyed preparation (collision-safe: net-new documents have fresh ids).
    let result = match ctx
        .renderer
        .render_url(RenderUrlRequest {
            item_id: ItemId::from_uuid(job.document_id.into_uuid()),
            user_id: job.user_id,
            url: job.url.clone(),
            outputs,
        })
        .await
    {
        Ok(result) => result,
        Err(error) => {
            if !has_completed(ArchiveAssetKind::ReadableHtml) {
                mark_readable_failure(ctx, job.document_id, &error.to_string()).await?;
            }
            return Err(error);
        }
    };

    let readable = result
        .artifacts
        .iter()
        .find(|artifact| artifact.kind == "readable_html");
    if !has_completed(ArchiveAssetKind::ReadableHtml) && readable.is_none() {
        let reason = result
            .asset_errors
            .iter()
            .find(|e| e.kind == "readable_html")
            .map(|e| e.error.clone())
            .unwrap_or_else(|| "renderer returned no readable_html artifact".to_string());
        mark_readable_failure(ctx, job.document_id, &reason).await?;
        return Err(AppError::ExternalService {
            service: "renderer".into(),
            message: format!(
                "feed prepare for document {} produced no readable content: {reason}",
                job.document_id
            ),
        });
    }

    let word_count = readable
        .and_then(|artifact| artifact.metadata.as_ref())
        .and_then(|metadata| metadata.word_count)
        .filter(|count| *count > 0);
    if !has_completed(ArchiveAssetKind::ReadableHtml) && word_count.is_none() {
        let reason = "renderer returned readable_html without visible readable text";
        mark_readable_failure(ctx, job.document_id, reason).await?;
        return Err(AppError::ExternalService {
            service: "renderer".into(),
            message: format!(
                "feed prepare for document {} failed: {reason}",
                job.document_id
            ),
        });
    }

    for asset in ind_ingest::build_document_assets(job.document_id, &ctx.s3_bucket, &result) {
        ctx.document_asset_repo.upsert_document_asset(asset).await?;
    }

    // Persist the renderer's reading metrics (word count + 238 WPM reading time) computed from
    // the extracted readable text. Targeted column write; absent metadata leaves the columns NULL.
    if let Some(word_count) = word_count {
        ctx.document_repo
            .set_reading_metrics(
                job.user_id,
                job.document_id,
                word_count,
                ind_domain::reading_time_minutes_from_words(word_count),
            )
            .await?;
    }

    if let Some(metadata) = readable.and_then(|artifact| artifact.metadata.as_ref()) {
        ctx.document_repo
            .apply_rendered_metadata(
                job.user_id,
                job.document_id,
                DocumentRenderedMetadata {
                    title: metadata
                        .title
                        .clone()
                        .filter(|title| !title.eq_ignore_ascii_case("untitled")),
                    author: metadata.byline.clone(),
                    excerpt: metadata.excerpt.clone(),
                },
            )
            .await?;
    }

    // Persist the lead image the renderer extracted from the page (og:image -> twitter:image ->
    // first substantial article image), mirroring the extension save. Fill-if-absent so a
    // feed/RSS-provided image set at materialize time is preserved.
    if let Some(lead_image) = result
        .artifacts
        .iter()
        .find(|a| a.kind == "readable_html")
        .and_then(|a| a.metadata.as_ref())
        .and_then(|m| m.lead_image.as_deref())
    {
        ctx.document_repo
            .set_lead_image(job.user_id, job.document_id, lead_image)
            .await?;
    }

    // Now that readable content exists, make the document durably searchable.
    enqueue_search_reindex_document(ctx, job.document_id).await?;
    enqueue_document_embed_if_engaged(ctx, job.user_id, job.document_id).await?;

    ind_application::handlers::article_toc::apply_article_toc(
        ctx.object_storage.as_deref(),
        ctx.document_asset_repo.as_ref(),
        ctx.document_repo.as_ref(),
        job.document_id,
    )
    .await;

    Ok(())
}

async fn mark_readable_failure(
    ctx: &FeedJobDeps,
    document_id: ind_domain::DocumentId,
    reason: &str,
) -> Result<(), AppError> {
    ctx.document_asset_repo
        .upsert_document_asset(NewDocumentAsset {
            document_id,
            asset_kind: ArchiveAssetKind::ReadableHtml,
            s3_key: String::new(),
            s3_bucket: ctx.s3_bucket.clone(),
            content_type: "text/html".into(),
            size_bytes: 0,
            status: ArchiveAssetStatus::Failed,
            failed_reason: Some(reason.to_string()),
        })
        .await?;
    Ok(())
}
