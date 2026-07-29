use bytes::Bytes;
use ind_application::AppError;
use ind_application::handlers::feed_identity::feed_entry_identity;
use ind_application::handlers::provided_content::stage_provided_content;
use ind_application::repos::document_lifecycle::{
    LibraryRestorePolicy, MaterializeSideEffects, SaveSideEffectsFn, SaveToLibraryRequest,
};
use ind_domain::{ArchiveAssetKind, ContentSource, DomainError, FeedAutosaveJob};
use ind_integrations::email::prepare_email_for_reader;

use crate::context::FeedJobDeps;

/// Autosave a newly delivered feed entry to the Library (TASK-236). Both URL and no-URL
/// (newsletter) entries route through the atomic `save_to_library` primitive keyed by the shared
/// `feed_entry_identity`, so save and read-ahead preparation converge on one document and the save
/// is idempotent (insert-or-restore). URL entries enable content-gated AI (the prep pipeline
/// renders readable content); no-URL entries attach their inline HTML as a document-keyed readable
/// asset and then index. When the subscription has an auto-save collection, the saved library
/// entry is added to it.
pub async fn handle_feed_autosave(ctx: &FeedJobDeps, job: FeedAutosaveJob) -> Result<(), AppError> {
    let delivery = ctx
        .feed_delivery_repo
        .find_by_id(job.feed_delivery_id, job.user_id)
        .await?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "FeedDelivery",
                id: job.feed_delivery_id.to_string(),
            })
        })?;

    let Some(entry) = ctx
        .feed_repo
        .find_source_entry_by_id(job.source_entry_id)
        .await?
    else {
        return Err(AppError::Domain(DomainError::NotFound {
            entity: "FeedSourceEntry",
            id: job.source_entry_id.to_string(),
        }));
    };

    // A renderable URL is any raw entry URL, even one that failed canonicalization: the Origin
    // identity keeps it as the document's `original_url`, and the engaged-AI builder renders that
    // raw URL when no canonical URL exists. Gating on `canonical_url` instead would strand an
    // un-canonicalizable entry that also lacks inline HTML with no readable path at all.
    let has_renderable_url = entry.url.is_some();

    // Only genuinely URL-less newsletter entries stage provided content: their inline HTML is the
    // only readable source. Entries with any URL are prepared by the content-gated AI pipeline.
    let staged_readable = if !has_renderable_url {
        match entry.content_html.as_deref() {
            Some(content_html) => {
                let storage = ctx.object_storage.as_ref().ok_or_else(|| {
                    AppError::Repository(
                        "object storage is not configured; cannot attach readable document asset"
                            .into(),
                    )
                })?;
                let reader_html = prepare_email_for_reader(content_html);
                Some(
                    stage_provided_content(
                        storage,
                        job.user_id,
                        ArchiveAssetKind::ReadableHtml,
                        "text/html",
                        Bytes::from(reader_html),
                    )
                    .await?,
                )
            }
            None => None,
        }
    } else {
        None
    };

    let user_id = job.user_id;
    let side_effects: Option<SaveSideEffectsFn> =
        staged_readable.map(|staged| -> SaveSideEffectsFn {
            Box::new(move |ctx| MaterializeSideEffects {
                events: Vec::new(),
                outbox: vec![staged.outbox(ctx.document.id, user_id)],
            })
        });

    let outcome = ctx
        .lifecycle
        .save_to_library(SaveToLibraryRequest {
            identity: feed_entry_identity(job.user_id, &entry),
            source: ContentSource::Feed,
            source_delivery_id: Some(job.feed_delivery_id),
            hide_deliveries: true,
            enqueue_engaged_ai: has_renderable_url,
            restore_policy: LibraryRestorePolicy::SkipIfDeletedAfter(delivery.delivered_at),
            side_effects,
        })
        .await?;

    if let Some(collection_id) = job.collection_id
        && !outcome.skipped_restore
    {
        ctx.collection_repo
            .add_library_entry_to_collection(job.user_id, collection_id, outcome.entry.id)
            .await?;
    }

    Ok(())
}
