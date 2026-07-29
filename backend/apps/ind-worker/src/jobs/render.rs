use ind_application::error::AppError;
use ind_domain::{
    AttachProvidedContentJob, EnsureArticleTocJob, GenericJobEnvelope, ReprocessDocumentJob,
    YoutubeIngestDocumentJob, job_types,
};

use crate::context::WorkerContext;

pub async fn dispatch_generic_job(
    ctx: &WorkerContext,
    envelope: GenericJobEnvelope,
) -> Result<(), AppError> {
    let email = ctx.email_jobs();
    if crate::jobs::email_ingest::dispatch_generic_job(&email, envelope.clone())
        .await?
        .is_some()
    {
        return Ok(());
    }

    if crate::jobs::email_unsubscribe::dispatch_generic_job(&email, envelope.clone())
        .await?
        .is_some()
    {
        return Ok(());
    }

    let feed = ctx.feed_jobs();
    if crate::jobs::feed::dispatch_generic_job(&feed, envelope.clone())
        .await?
        .is_some()
    {
        return Ok(());
    }

    let ai_search = ctx.ai_search_jobs();
    if crate::jobs::ai::dispatch_generic_job(&ai_search, envelope.clone())
        .await?
        .is_some()
    {
        return Ok(());
    }

    if crate::jobs::search::dispatch_generic_job(&ai_search, envelope.clone())
        .await?
        .is_some()
    {
        return Ok(());
    }

    let webhooks = ctx.webhook_jobs();
    if crate::jobs::webhooks::dispatch_generic_job(&webhooks, envelope.clone())
        .await?
        .is_some()
    {
        return Ok(());
    }

    let integrations = ctx.integration_jobs();
    if crate::jobs::integrations::dispatch_envelope(&integrations, envelope.clone())
        .await?
        .is_some()
    {
        return Ok(());
    }

    match envelope.job_type.as_str() {
        "document.youtube_ingest" => {
            let job: YoutubeIngestDocumentJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::Repository(Box::new(e)))?;
            crate::jobs::youtube::handle_youtube_ingest_document(&ctx.capture_jobs(), job).await
        }
        job_types::DOCUMENT_ATTACH_PROVIDED_CONTENT => {
            let job: AttachProvidedContentJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::Repository(Box::new(e)))?;
            crate::jobs::attach_provided_content::handle_attach_provided_content(
                &ctx.capture_jobs(),
                job,
            )
            .await
        }
        job_types::DOCUMENT_REPROCESS => {
            let job: ReprocessDocumentJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::Repository(Box::new(e)))?;
            crate::jobs::reprocess::handle_document_reprocess(&ctx.capture_jobs(), job).await
        }
        job_types::DOCUMENT_TOC_ENSURE => {
            let job: EnsureArticleTocJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::Repository(Box::new(e)))?;
            crate::jobs::article_toc::handle_toc_ensure(&ctx.capture_jobs(), job).await
        }
        other => {
            tracing::warn!(job_type = other, "unknown job type, skipping");
            Ok(())
        }
    }
}
