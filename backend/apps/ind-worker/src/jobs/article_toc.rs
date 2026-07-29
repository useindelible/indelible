//! `document.toc.ensure` — lazy/backfill derivation of an article's table of
//! contents. Enqueued by the ToC read path when the stored outline is missing
//! or stale; new-content ingest arms derive inline instead.

use ind_application::AppError;
use ind_application::handlers::article_toc::{EnsureOutcome, ensure_article_toc};
use ind_domain::EnsureArticleTocJob;

use crate::context::CaptureJobDeps;

pub async fn handle_toc_ensure(
    ctx: &CaptureJobDeps,
    job: EnsureArticleTocJob,
) -> Result<(), AppError> {
    let Some(storage) = ctx.object_storage.as_deref() else {
        tracing::warn!(
            document_id = %job.document_id,
            "object storage not configured; skipping toc ensure"
        );
        return Ok(());
    };
    let outcome = ensure_article_toc(
        storage,
        ctx.document_asset_repo.as_ref(),
        ctx.document_repo.as_ref(),
        job.document_id,
    )
    .await?;
    match outcome {
        EnsureOutcome::Committed(_) => {}
        // Both are terminal for this run: the read path re-enqueues if a ToC is
        // still wanted for the (newer) content.
        EnsureOutcome::LostRace => {
            tracing::info!(document_id = %job.document_id, "toc ensure lost a reprocess race");
        }
        EnsureOutcome::NoReadableHtml => {
            tracing::info!(document_id = %job.document_id, "toc ensure found no readable html");
        }
    }
    Ok(())
}
