//! Transaction-scoped lifecycle steps composed by `materialize_document`.
//!
//! These run inside a caller-owned transaction so a future library-save lifecycle
//! method can reuse them in the same transaction. See
//! docs/document-feed-library-architecture.md (Materialization and adoption must be atomic).

use chrono::{DateTime, Utc};

use ind_application::AppError;
use ind_application::repos::document_lifecycle::{DocumentStateInput, MaterializeIdentity};
use ind_application::repos::lifecycle_outbox::{
    OutboxEntry, document_ai_processing_outbox, feed_prepare_document_outbox,
};
use ind_domain::{Document, DocumentId, UserDocumentState, UserId};

use super::super::document_repo::tx_writes::{
    PgTx, materialize_origin_backed_tx, materialize_url_backed_tx, record_origin_tx,
};

/// Materialize-or-find the document for `identity` and record provenance. URL-backed
/// identities optionally record an extra origin row; origin-backed identities have their
/// origin recorded by the materialize step itself. Returns the document and whether this
/// call inserted it.
pub(crate) async fn materialize_document_tx(
    tx: &mut PgTx<'_>,
    identity: &MaterializeIdentity,
) -> Result<(Document, bool), AppError> {
    let (row, created) = match identity {
        MaterializeIdentity::Url { document, origin } => {
            let (row, created) = materialize_url_backed_tx(tx, document).await?;
            if let Some(origin) = origin {
                record_origin_tx(
                    tx,
                    document.user_id,
                    DocumentId::from_uuid(row.id),
                    origin.origin_type,
                    origin.origin_id,
                )
                .await?;
            }
            (row, created)
        }
        MaterializeIdentity::Origin { document, origin } => {
            materialize_origin_backed_tx(tx, document, origin.origin_type, origin.origin_id).await?
        }
    };
    Ok((row.into_document()?, created))
}

/// Adoption key for back-linking unlinked feed deliveries to a materialized document.
pub(crate) enum BacklinkKey<'a> {
    CanonicalUrl(&'a str),
    SourceEntry(uuid::Uuid),
}

/// Back-link the user's matching unlinked feed deliveries to `document_id`. Uses the
/// indexed adoption query from the architecture doc. Returns the number of deliveries
/// linked.
pub(crate) async fn backlink_feed_deliveries_tx(
    tx: &mut PgTx<'_>,
    user_id: UserId,
    document_id: DocumentId,
    key: BacklinkKey<'_>,
) -> Result<u64, AppError> {
    let result = match key {
        BacklinkKey::CanonicalUrl(canonical_url) => {
            sqlx::query!(
                "UPDATE feed_deliveries fd \
             SET document_id = $1, updated_at = now() \
             FROM feed_source_entries fse \
             WHERE fd.source_entry_id = fse.id \
               AND fd.user_id = $2 \
               AND fd.document_id IS NULL \
               AND fse.canonical_url = $3",
                document_id.into_uuid(),
                user_id.into_uuid(),
                canonical_url,
            )
            .execute(&mut **tx)
            .await
        }
        BacklinkKey::SourceEntry(source_entry_id) => {
            sqlx::query!(
                "UPDATE feed_deliveries \
             SET document_id = $1, updated_at = now() \
             WHERE user_id = $2 \
               AND document_id IS NULL \
               AND source_entry_id = $3",
                document_id.into_uuid(),
                user_id.into_uuid(),
                source_entry_id,
            )
            .execute(&mut **tx)
            .await
        }
    };

    Ok(result
        .map_err(|e| AppError::Repository(Box::new(e)))?
        .rows_affected())
}

struct UserDocumentStateRow {
    user_id: uuid::Uuid,
    document_id: uuid::Uuid,
    progress_percent: Option<i32>,
    max_progress_percent: Option<i32>,
    scroll_position: Option<serde_json::Value>,
    chapter_locator: Option<String>,
    chapter_offset: Option<i32>,
    last_read_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
    first_opened_at: Option<DateTime<Utc>>,
    last_opened_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl UserDocumentStateRow {
    fn into_state(self) -> UserDocumentState {
        UserDocumentState {
            user_id: UserId::from_uuid(self.user_id),
            document_id: DocumentId::from_uuid(self.document_id),
            progress_percent: self.progress_percent,
            max_progress_percent: self.max_progress_percent,
            scroll_position: self.scroll_position,
            chapter_locator: self.chapter_locator,
            chapter_offset: self.chapter_offset,
            last_read_at: self.last_read_at,
            finished_at: self.finished_at,
            first_opened_at: self.first_opened_at,
            last_opened_at: self.last_opened_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// Targeted `user_document_state` upsert for opened timestamps. `first_opened_at` is set when
/// currently NULL (COALESCE); `last_opened_at` only moves forward via GREATEST so a delayed/older
/// `opened_at` cannot regress it. No whole-row read-modify-write.
pub(crate) async fn upsert_user_document_state_tx(
    tx: &mut PgTx<'_>,
    user_id: UserId,
    document_id: DocumentId,
    input: &DocumentStateInput,
) -> Result<UserDocumentState, AppError> {
    let row = sqlx::query_as!(
        UserDocumentStateRow,
        "INSERT INTO user_document_state \
            (user_id, document_id, first_opened_at, last_opened_at, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, now(), now()) \
         ON CONFLICT (user_id, document_id) DO UPDATE SET \
            first_opened_at = COALESCE(user_document_state.first_opened_at, EXCLUDED.first_opened_at), \
            last_opened_at = GREATEST(user_document_state.last_opened_at, EXCLUDED.last_opened_at), \
            updated_at = now() \
         RETURNING user_id, document_id, progress_percent, max_progress_percent, \
                   scroll_position AS \"scroll_position?: serde_json::Value\", chapter_locator, \
                   chapter_offset, last_read_at, finished_at, first_opened_at, last_opened_at, \
                   created_at, updated_at",
        user_id.into_uuid(),
        document_id.into_uuid(),
        input.opened_at,
        input.opened_at,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| super::super::map_sqlx_error("user_document_state", "state conflict", e))?;

    Ok(row.into_state())
}

/// Whether a completed readable (`readable_html`) asset already exists for the document.
pub(crate) async fn document_has_completed_readable_asset_tx(
    tx: &mut PgTx<'_>,
    document_id: DocumentId,
) -> Result<bool, AppError> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS( \
            SELECT 1 FROM archive_assets \
            WHERE document_id = $1 \
              AND asset_kind = 'readable_html' \
              AND status = 'completed')",
        document_id.into_uuid(),
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| AppError::Repository(Box::new(e)))?;

    Ok(exists.unwrap_or(false))
}

/// Whether the user has an active (non-soft-deleted) library entry for the document.
pub(crate) async fn document_is_saved_tx(
    tx: &mut PgTx<'_>,
    user_id: UserId,
    document_id: DocumentId,
) -> Result<bool, AppError> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS( \
            SELECT 1 FROM library_entries \
            WHERE user_id = $1 AND document_id = $2 AND deleted_at IS NULL)",
        user_id.into_uuid(),
        document_id.into_uuid(),
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| AppError::Repository(Box::new(e)))?;

    Ok(exists.unwrap_or(false))
}

/// Content-gated AI outbox for an engaged document (TASK-234, Codex P1). The decision lives in
/// the transaction (not a synchronous side-effect closure) because it must query the DB:
/// embed immediately if a completed readable asset exists; else enqueue readable preparation
/// (which embeds on completion because the document is engaged); else nothing (origin-backed
/// documents with no URL already carry rendered content from ingest).
pub(crate) async fn build_engaged_document_ai_outbox_tx(
    tx: &mut PgTx<'_>,
    document: &Document,
) -> Result<Vec<OutboxEntry>, AppError> {
    let now = Utc::now();
    if document_has_completed_readable_asset_tx(tx, document.id).await? {
        return Ok(document_ai_processing_outbox(document.id, now));
    }
    if let Some(url) = document
        .canonical_url
        .clone()
        .or_else(|| document.original_url.clone())
    {
        return Ok(vec![feed_prepare_document_outbox(
            document.id,
            document.user_id,
            url,
            now,
        )]);
    }
    Ok(Vec::new())
}
