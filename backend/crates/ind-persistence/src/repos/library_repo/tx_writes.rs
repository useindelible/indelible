//! Transaction-scoped library-entry write primitive.
//!
//! Composed by `PgDocumentLifecycle::save_to_library` inside the caller-owned save
//! transaction so the document materialization, delivery back-link, library membership,
//! retained state, and outbox all commit atomically. See
//! docs/document-feed-library-architecture.md (User saves a feed-delivered document).

use ind_application::AppError;
use ind_application::repos::document_lifecycle::LibraryRestorePolicy;
use ind_domain::{
    ContentSource, DocumentId, DomainError, FeedDeliveryId, LibraryEntry, LibraryEntryId, UserId,
};

use super::super::document_repo::tx_writes::PgTx;
use super::rows::{LibraryEntryRow, map_library_error};

/// Outcome of insert-or-restore: the resolved entry, whether a soft-deleted row was revived,
/// and whether an active row already existed (idempotent save).
pub(crate) struct LibraryEntryUpsert {
    pub entry: LibraryEntry,
    pub restored: bool,
    pub already_active: bool,
    pub skipped_restore: bool,
}

/// Insert, restore, or idempotently return the user's active `library_entries` row for
/// `document_id`. Locks any existing row (`FOR UPDATE`, active first); `INSERT ... ON CONFLICT`
/// makes the no-existing-row path race-safe against the partial unique index
/// `uq_library_entries_user_document_active`.
pub(crate) async fn insert_or_restore_library_entry_tx(
    tx: &mut PgTx<'_>,
    user_id: UserId,
    document_id: DocumentId,
    source: ContentSource,
    source_delivery_id: Option<FeedDeliveryId>,
    restore_policy: LibraryRestorePolicy,
) -> Result<LibraryEntryUpsert, AppError> {
    if let Some(delivery_id) = source_delivery_id {
        validate_source_delivery(tx, user_id, document_id, delivery_id).await?;
    }

    let existing = sqlx::query_as!(
        LibraryEntryRow,
        "SELECT id, user_id, document_id, saved_at, triage_state, is_favorite, is_shortlisted, \
                deleted_at, source, source_delivery_id, created_at, updated_at \
         FROM library_entries \
         WHERE user_id = $1 AND document_id = $2 \
         ORDER BY (deleted_at IS NULL) DESC, updated_at DESC \
         LIMIT 1 FOR UPDATE",
        user_id.into_uuid(),
        document_id.into_uuid(),
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_library_error)?;

    let delivery_uuid = source_delivery_id.map(|id| id.into_uuid());

    match existing {
        Some(row) if row.deleted_at.is_none() => {
            let updated = sqlx::query_as!(
                LibraryEntryRow,
                "UPDATE library_entries \
                 SET source_delivery_id = COALESCE(source_delivery_id, $2), updated_at = now() \
                 WHERE id = $1 \
                 RETURNING id, user_id, document_id, saved_at, triage_state, is_favorite, \
                           is_shortlisted, deleted_at, source, source_delivery_id, created_at, \
                           updated_at",
                row.id,
                delivery_uuid,
            )
            .fetch_one(&mut **tx)
            .await
            .map_err(map_library_error)?;

            Ok(LibraryEntryUpsert {
                entry: updated.into_entry()?,
                restored: false,
                already_active: true,
                skipped_restore: false,
            })
        }
        Some(row)
            if matches!(
                restore_policy,
                LibraryRestorePolicy::SkipIfDeletedAfter(cutoff)
                    if row.deleted_at.as_ref().is_some_and(|deleted_at| *deleted_at > cutoff)
            ) =>
        {
            Ok(LibraryEntryUpsert {
                entry: row.into_entry()?,
                restored: false,
                already_active: false,
                skipped_restore: true,
            })
        }
        Some(row) => {
            let restored = sqlx::query_as!(
                LibraryEntryRow,
                "UPDATE library_entries \
                 SET deleted_at = NULL, saved_at = now(), source = $2, \
                     source_delivery_id = COALESCE($3, source_delivery_id), updated_at = now() \
                 WHERE id = $1 \
                 RETURNING id, user_id, document_id, saved_at, triage_state, is_favorite, \
                           is_shortlisted, deleted_at, source, source_delivery_id, created_at, \
                           updated_at",
                row.id,
                source.as_str(),
                delivery_uuid,
            )
            .fetch_one(&mut **tx)
            .await
            .map_err(map_library_error)?;

            Ok(LibraryEntryUpsert {
                entry: restored.into_entry()?,
                restored: true,
                already_active: false,
                skipped_restore: false,
            })
        }
        None => {
            // No row existed at lock time. `DO NOTHING` returns the inserted row on success and
            // nothing if a concurrent first-save won the partial-unique race; that lets us report
            // `already_active` truthfully instead of treating the loser as a fresh save.
            let inserted = sqlx::query_as!(
                LibraryEntryRow,
                "INSERT INTO library_entries \
                    (id, user_id, document_id, saved_at, triage_state, is_favorite, \
                     is_shortlisted, deleted_at, source, source_delivery_id) \
                 VALUES ($1, $2, $3, now(), 'inbox', false, false, NULL, $4, $5) \
                 ON CONFLICT (user_id, document_id) WHERE deleted_at IS NULL DO NOTHING \
                 RETURNING id, user_id, document_id, saved_at, triage_state, is_favorite, \
                           is_shortlisted, deleted_at, source, source_delivery_id, created_at, \
                           updated_at",
                LibraryEntryId::new().into_uuid(),
                user_id.into_uuid(),
                document_id.into_uuid(),
                source.as_str(),
                delivery_uuid,
            )
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_library_error)?;

            match inserted {
                Some(row) => Ok(LibraryEntryUpsert {
                    entry: row.into_entry()?,
                    restored: false,
                    already_active: false,
                    skipped_restore: false,
                }),
                None => {
                    let existing = sqlx::query_as!(
                        LibraryEntryRow,
                        "UPDATE library_entries \
                         SET source_delivery_id = COALESCE(source_delivery_id, $3), \
                             updated_at = now() \
                         WHERE user_id = $1 AND document_id = $2 AND deleted_at IS NULL \
                         RETURNING id, user_id, document_id, saved_at, triage_state, is_favorite, \
                                   is_shortlisted, deleted_at, source, source_delivery_id, \
                                   created_at, updated_at",
                        user_id.into_uuid(),
                        document_id.into_uuid(),
                        delivery_uuid,
                    )
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(map_library_error)?;

                    Ok(LibraryEntryUpsert {
                        entry: existing.into_entry()?,
                        restored: false,
                        already_active: true,
                        skipped_restore: false,
                    })
                }
            }
        }
    }
}

/// The provenance link must be truthful: the delivery belongs to this user and is already
/// linked to the document being saved. Back-linking runs before this in the save tx.
async fn validate_source_delivery(
    tx: &mut PgTx<'_>,
    user_id: UserId,
    document_id: DocumentId,
    delivery_id: FeedDeliveryId,
) -> Result<(), AppError> {
    let valid = sqlx::query_scalar!(
        "SELECT EXISTS ( \
             SELECT 1 FROM feed_deliveries \
             WHERE id = $1 AND user_id = $2 AND document_id = $3 \
         )",
        delivery_id.into_uuid(),
        user_id.into_uuid(),
        document_id.into_uuid(),
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(map_library_error)?;

    if valid != Some(true) {
        return Err(AppError::Domain(DomainError::Validation {
            field: "source_delivery_id".into(),
            message: "source delivery must belong to the owner and be linked to the saved document"
                .into(),
        }));
    }

    Ok(())
}
