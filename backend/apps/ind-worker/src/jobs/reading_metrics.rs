//! Reading-metric computation shared by the content preparation paths (provided-content
//! attach, Readwise import arms). Word counting matches the renderer's readability path
//! (`split_whitespace` on extracted text); reading time derives from the single 238 WPM
//! constant in `ind_domain::reading_time_minutes_from_words`.

use ind_domain::{DocumentId, UserId, reading_time_minutes_from_words};

use ind_application::repos::document::DocumentRepository;

pub(crate) fn word_count_from_html(html: &str) -> i32 {
    word_count_from_text(&ind_html::html_to_text(html))
}

pub(crate) fn word_count_from_text(text: &str) -> i32 {
    text.split_whitespace().count() as i32
}

/// Best-effort targeted write: reading metrics are cosmetic reader metadata, so a failure here
/// must never fail the content attach/import that carries them — log and move on. Non-positive
/// counts are skipped to keep "unknown" as NULL rather than a misleading 0.
pub(crate) async fn apply_reading_metrics(
    document_repo: &dyn DocumentRepository,
    user_id: UserId,
    document_id: DocumentId,
    word_count: i32,
) {
    if word_count <= 0 {
        return;
    }
    if let Err(err) = document_repo
        .set_reading_metrics(
            user_id,
            document_id,
            word_count,
            reading_time_minutes_from_words(word_count),
        )
        .await
    {
        tracing::warn!(
            error = %err,
            document_id = %document_id,
            "failed to persist reading metrics"
        );
    }
}
