//! Transaction-scoped feed-delivery write primitive for the save flow.
//!
//! Composed by `PgDocumentLifecycle::save_to_library` so hiding the saved document's
//! deliveries commits atomically with the library membership. See
//! docs/document-feed-library-architecture.md (Saving a document should usually hide or
//! dismiss its feed deliveries).

use ind_application::AppError;
use ind_domain::{DocumentId, UserId};

use super::super::document_repo::tx_writes::PgTx;

/// Hide the user's active feed deliveries linked to `document_id` (set `hidden_at`). Runs
/// after back-linking so every matching delivery already carries the document id. Already
/// dismissed/hidden deliveries are left untouched. Returns the number hidden.
pub(crate) async fn hide_deliveries_for_document_tx(
    tx: &mut PgTx<'_>,
    user_id: UserId,
    document_id: DocumentId,
) -> Result<u64, AppError> {
    let result = sqlx::query!(
        "UPDATE feed_deliveries \
         SET hidden_at = now(), updated_at = now() \
         WHERE user_id = $1 AND document_id = $2 \
           AND hidden_at IS NULL AND dismissed_at IS NULL",
        user_id.into_uuid(),
        document_id.into_uuid(),
    )
    .execute(&mut **tx)
    .await
    .map_err(|e| AppError::Repository(Box::new(e)))?;

    Ok(result.rows_affected())
}
