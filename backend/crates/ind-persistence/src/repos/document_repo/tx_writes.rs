//! Transaction-scoped document write primitives.
//!
//! These operate on a caller-owned `&mut Transaction` so that higher-level lifecycle
//! flows (TASK-228 `PgDocumentLifecycle`) can compose materialization, delivery
//! back-linking, retained state, domain events, and job outbox rows into a single
//! atomic transaction. The self-committing `DocumentRepository` methods in `writes.rs`
//! are thin wrappers over these. See docs/document-feed-library-architecture.md
//! (Materialization and adoption must be atomic).

use ind_application::{AppError, normalize_language_tag, text::strip_nul};
use ind_domain::{
    DocumentId, DocumentOriginType, DomainError, EmailSenderId, NewOriginDocument, NewUrlDocument,
    UserId,
};
use sqlx::Acquire;
use uuid::Uuid;

use super::rows::{DocumentRow, map_document_error, map_origin_error, origin_type_to_str};

pub(crate) type PgTx<'c> = sqlx::Transaction<'c, sqlx::Postgres>;

/// URL-backed materialize-or-find on `(user_id, canonical_url)`. Returns the resolved
/// document and `true` when this call inserted it. Under READ COMMITTED a losing
/// concurrent insert blocks on the winner, then the re-select observes the committed row.
pub(crate) async fn materialize_url_backed_tx(
    tx: &mut PgTx<'_>,
    doc: &NewUrlDocument,
) -> Result<(DocumentRow, bool), AppError> {
    let language = normalize_language_tag(doc.language.as_deref());
    let title = strip_nul(&doc.title);
    let author = doc.author.as_deref().map(strip_nul);
    let excerpt = doc.excerpt.as_deref().map(strip_nul);
    let inserted = sqlx::query_as!(
        DocumentRow,
        "INSERT INTO documents \
            (id, user_id, document_type, canonical_url, original_url, content_hash, \
             title, author, excerpt, published_at, language, domain, lead_image_url, \
             thumbnail_url) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) \
         ON CONFLICT (user_id, canonical_url) WHERE canonical_url IS NOT NULL DO NOTHING \
         RETURNING id, user_id, document_type, canonical_url, original_url, content_hash, \
                   title, author, excerpt, published_at, language, domain, lead_image_url, \
                   thumbnail_url, word_count, reading_time_minutes, created_at, updated_at",
        doc.id.into_uuid(),
        doc.user_id.into_uuid(),
        doc.document_type.as_str(),
        doc.canonical_url,
        doc.original_url,
        doc.content_hash,
        title,
        author,
        excerpt,
        doc.published_at,
        language,
        doc.domain,
        doc.lead_image_url,
        doc.thumbnail_url,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_document_error)?;

    if let Some(row) = inserted {
        return Ok((row, true));
    }

    let existing = select_document_by_canonical_url(tx, doc.user_id, &doc.canonical_url)
        .await?
        .ok_or_else(|| {
            AppError::Domain(DomainError::InvariantViolation {
                message: "url-backed upsert conflict resolved to no document".into(),
            })
        })?;
    Ok((existing, false))
}

/// No-URL materialize-or-find keyed by the origin. Identity precedence: an existing
/// origin mapping wins; otherwise dedup by `(user_id, content_hash)` when present;
/// otherwise the `document_origins` row is the sole identity. A lost origin race rolls
/// back any speculative document insert (savepoint) so no orphan document is left.
/// Returns the resolved document and `true` when this call inserted it.
pub(crate) async fn materialize_origin_backed_tx(
    tx: &mut PgTx<'_>,
    doc: &NewOriginDocument,
    origin_type: DocumentOriginType,
    origin_id: Uuid,
) -> Result<(DocumentRow, bool), AppError> {
    let user_id = doc.user_id;

    if let Some(row) = select_document_by_origin(tx, user_id, origin_type, origin_id).await? {
        // Re-ingesting the same email resolves the existing document; a sender resolved on this
        // pass (e.g. the sender row only existed after a later delivery) is linked via a targeted
        // column-scoped UPDATE so the linkage is not lost on re-materialization.
        if let Some(sender_id) = doc.sender_id {
            set_document_sender_tx(tx, user_id, DocumentId::from_uuid(row.id), sender_id).await?;
        }
        return Ok((row, false));
    }

    let language = normalize_language_tag(doc.language.as_deref());
    let title = strip_nul(&doc.title);
    let author = doc.author.as_deref().map(strip_nul);
    let excerpt = doc.excerpt.as_deref().map(strip_nul);
    let mut sp = tx.begin().await.map_err(map_document_error)?;
    // `document_created` tracks whether THIS call inserted the document, distinct from
    // whether it claimed the origin: a content-hash hit reuses an existing document and
    // only attaches a new origin, so the caller must not treat that as a fresh document.
    let (candidate, document_created) = if doc.content_hash.is_some() {
        let inserted = sqlx::query_as!(
            DocumentRow,
            "INSERT INTO documents \
                (id, user_id, document_type, content_hash, original_url, title, author, \
                 excerpt, published_at, language, domain, lead_image_url, thumbnail_url, sender_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) \
             ON CONFLICT (user_id, content_hash) \
                WHERE canonical_url IS NULL AND content_hash IS NOT NULL DO NOTHING \
             RETURNING id, user_id, document_type, canonical_url, original_url, content_hash, \
                       title, author, excerpt, published_at, language, domain, lead_image_url, \
                       thumbnail_url, word_count, reading_time_minutes, created_at, updated_at",
            doc.id.into_uuid(),
            doc.user_id.into_uuid(),
            doc.document_type.as_str(),
            doc.content_hash,
            doc.original_url,
            title,
            author,
            excerpt,
            doc.published_at,
            language,
            doc.domain,
            doc.lead_image_url,
            doc.thumbnail_url,
            doc.sender_id.map(|id| id.into_uuid()),
        )
        .fetch_optional(&mut *sp)
        .await
        .map_err(map_document_error)?;

        match inserted {
            Some(row) => (row, true),
            None => {
                let existing = sqlx::query_as!(
                    DocumentRow,
                    "SELECT id, user_id, document_type, canonical_url, original_url, content_hash, \
                            title, author, excerpt, published_at, language, domain, lead_image_url, \
                            thumbnail_url, word_count, reading_time_minutes, created_at, updated_at \
                     FROM documents \
                     WHERE user_id = $1 AND content_hash = $2 AND canonical_url IS NULL",
                    doc.user_id.into_uuid(),
                    doc.content_hash,
                )
                .fetch_one(&mut *sp)
                .await
                .map_err(map_document_error)?;
                (existing, false)
            }
        }
    } else {
        let inserted = sqlx::query_as!(
            DocumentRow,
            "INSERT INTO documents \
                (id, user_id, document_type, content_hash, original_url, title, author, \
                 excerpt, published_at, language, domain, lead_image_url, thumbnail_url, sender_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) \
             RETURNING id, user_id, document_type, canonical_url, original_url, content_hash, \
                       title, author, excerpt, published_at, language, domain, lead_image_url, \
                       thumbnail_url, word_count, reading_time_minutes, created_at, updated_at",
            doc.id.into_uuid(),
            doc.user_id.into_uuid(),
            doc.document_type.as_str(),
            doc.content_hash,
            doc.original_url,
            title,
            author,
            excerpt,
            doc.published_at,
            language,
            doc.domain,
            doc.lead_image_url,
            doc.thumbnail_url,
            doc.sender_id.map(|id| id.into_uuid()),
        )
        .fetch_one(&mut *sp)
        .await
        .map_err(map_document_error)?;
        (inserted, true)
    };

    let origin_inserted =
        insert_origin(&mut sp, user_id, candidate.id, origin_type, origin_id).await?;
    if origin_inserted {
        sp.commit().await.map_err(map_origin_error)?;
        return Ok((candidate, document_created));
    }

    sp.rollback().await.map_err(map_origin_error)?;
    let owned = select_document_by_origin(tx, user_id, origin_type, origin_id)
        .await?
        .ok_or_else(|| {
            AppError::Domain(DomainError::InvariantViolation {
                message: "origin-backed upsert lost race but origin has no document".into(),
            })
        })?;
    Ok((owned, false))
}

/// Record provenance for an already-materialized document. Idempotent for the same
/// `(origin, document)`; errors if the origin already maps to a different document.
pub(crate) async fn record_origin_tx(
    tx: &mut PgTx<'_>,
    user_id: UserId,
    document_id: DocumentId,
    origin_type: DocumentOriginType,
    origin_id: Uuid,
) -> Result<(), AppError> {
    if insert_origin(tx, user_id, document_id.into_uuid(), origin_type, origin_id).await? {
        return Ok(());
    }

    let existing = sqlx::query_scalar!(
        "SELECT document_id FROM document_origins \
         WHERE user_id = $1 AND origin_type = $2 AND origin_id = $3",
        user_id.into_uuid(),
        origin_type_to_str(origin_type),
        origin_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(map_origin_error)?;

    if existing == document_id.into_uuid() {
        Ok(())
    } else {
        Err(AppError::Domain(DomainError::Conflict {
            entity: "document_origin",
            message: "origin already mapped to a different document".into(),
        }))
    }
}

pub(crate) async fn select_document_by_id(
    tx: &mut PgTx<'_>,
    user_id: UserId,
    document_id: DocumentId,
) -> Result<Option<DocumentRow>, AppError> {
    sqlx::query_as!(
        DocumentRow,
        "SELECT id, user_id, document_type, canonical_url, original_url, content_hash, \
                title, author, excerpt, published_at, language, domain, lead_image_url, \
                thumbnail_url, word_count, reading_time_minutes, created_at, updated_at \
         FROM documents \
         WHERE user_id = $1 AND id = $2",
        user_id.into_uuid(),
        document_id.into_uuid(),
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_document_error)
}

pub(crate) async fn select_document_by_canonical_url(
    tx: &mut PgTx<'_>,
    user_id: UserId,
    canonical_url: &str,
) -> Result<Option<DocumentRow>, AppError> {
    sqlx::query_as!(
        DocumentRow,
        "SELECT id, user_id, document_type, canonical_url, original_url, content_hash, \
                title, author, excerpt, published_at, language, domain, lead_image_url, \
                thumbnail_url, word_count, reading_time_minutes, created_at, updated_at \
         FROM documents \
         WHERE user_id = $1 AND canonical_url = $2",
        user_id.into_uuid(),
        canonical_url,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_document_error)
}

pub(crate) async fn select_document_by_origin(
    tx: &mut PgTx<'_>,
    user_id: UserId,
    origin_type: DocumentOriginType,
    origin_id: Uuid,
) -> Result<Option<DocumentRow>, AppError> {
    sqlx::query_as!(
        DocumentRow,
        "SELECT d.id, d.user_id, d.document_type, d.canonical_url, d.original_url, d.content_hash, \
                d.title, d.author, d.excerpt, d.published_at, d.language, d.domain, \
                d.lead_image_url, d.thumbnail_url, d.word_count, d.reading_time_minutes, d.created_at, d.updated_at \
         FROM documents d \
         JOIN document_origins o ON o.document_id = d.id AND o.user_id = d.user_id \
         WHERE o.user_id = $1 AND o.origin_type = $2 AND o.origin_id = $3",
        user_id.into_uuid(),
        origin_type_to_str(origin_type),
        origin_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_document_error)
}

/// Column-scoped sender linkage write. Idempotent: skips the write when the column already holds
/// the target sender so re-ingest does not bump `updated_at` needlessly.
async fn set_document_sender_tx(
    tx: &mut PgTx<'_>,
    user_id: UserId,
    document_id: DocumentId,
    sender_id: EmailSenderId,
) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE documents SET sender_id = $3, updated_at = now() \
         WHERE id = $1 AND user_id = $2 AND sender_id IS DISTINCT FROM $3",
        document_id.into_uuid(),
        user_id.into_uuid(),
        sender_id.into_uuid(),
    )
    .execute(&mut **tx)
    .await
    .map_err(map_document_error)?;
    Ok(())
}

async fn insert_origin(
    tx: &mut PgTx<'_>,
    user_id: UserId,
    document_id: Uuid,
    origin_type: DocumentOriginType,
    origin_id: Uuid,
) -> Result<bool, AppError> {
    let inserted = sqlx::query_scalar!(
        "INSERT INTO document_origins (user_id, document_id, origin_type, origin_id) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (user_id, origin_type, origin_id) DO NOTHING \
         RETURNING document_id",
        user_id.into_uuid(),
        document_id,
        origin_type_to_str(origin_type),
        origin_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_origin_error)?;

    Ok(inserted.is_some())
}
