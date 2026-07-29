use ind_application::AppError;
use ind_domain::{FeedAutosaveJob, FeedPollJob, GenericJobEnvelope, PrepareDocumentJob};

use crate::context::FeedJobDeps;

use super::autosave::handle_feed_autosave;
use super::poll::handle_feed_poll;
use super::prepare::handle_prepare_document;

pub async fn dispatch_generic_job(
    ctx: &FeedJobDeps,
    envelope: GenericJobEnvelope,
) -> Result<Option<()>, AppError> {
    match envelope.job_type.as_str() {
        "feed.poll" => {
            let job: FeedPollJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::Repository(Box::new(e)))?;
            handle_feed_poll(ctx, job).await?;
            Ok(Some(()))
        }
        "feed.autosave" => {
            let job: FeedAutosaveJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::Repository(Box::new(e)))?;
            handle_feed_autosave(ctx, job).await?;
            Ok(Some(()))
        }
        "feed.prepare_document" => {
            let job: PrepareDocumentJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::Repository(Box::new(e)))?;
            handle_prepare_document(ctx, job).await?;
            Ok(Some(()))
        }
        _ => Ok(None),
    }
}
