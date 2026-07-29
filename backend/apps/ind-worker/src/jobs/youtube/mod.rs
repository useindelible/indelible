use chrono::Utc;
use ind_application::error::AppError;
use ind_domain::{DocumentId, UserId};

use crate::context::IndexQueueContext;

mod document;
mod html;
mod player;
#[cfg(test)]
mod tests;
mod transcript;

pub use document::handle_youtube_ingest_document;

/// Enqueue document-keyed YouTube ingest so a YouTube URL gets transcript-enriched reader content
/// instead of the generic render pipeline archiving a watch page (TASK-240). Shared by Readwise
/// import and the feed `prepare` choke point; the dedupe key collapses overlapping routes.
pub(crate) async fn enqueue_youtube_ingest_document(
    ctx: &impl IndexQueueContext,
    user_id: UserId,
    document_id: DocumentId,
    url: &str,
) -> Result<(), AppError> {
    let entry = ind_application::repos::lifecycle_outbox::youtube_ingest_document_outbox(
        document_id,
        user_id,
        url.to_string(),
        Utc::now(),
    );
    ctx.outbox_repo()
        .enqueue(
            &entry.job_type,
            entry.payload,
            entry.dedupe_key,
            entry.available_at,
        )
        .await?;
    Ok(())
}

fn truncate_chars(s: &str, n: usize) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i >= n {
            break;
        }
        out.push(ch);
    }
    out
}
