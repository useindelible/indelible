use chrono::Utc;
use ind_application::AppError;
use ind_application::event_intents::{
    library_entry_favorite_changed, library_entry_permanently_deleted, library_entry_restored,
    library_entry_trashed, library_entry_triaged,
};
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::lifecycle_outbox::search_reindex_document_outbox;
use ind_domain::{DocumentId, DomainError, LibraryEntry, LibraryEntryId, TriageState, UserId};

use super::PgLibraryRepository;
use super::rows::{LibraryEntryRow, map_library_error};
use crate::repos::write_helpers::apply_mutation_side_effects_tx;

fn not_found(id: LibraryEntryId) -> AppError {
    AppError::Domain(DomainError::NotFound {
        entity: "LibraryEntry",
        id: id.to_string(),
    })
}

fn permanent_deletion_effects(
    entries: impl IntoIterator<Item = (LibraryEntryId, UserId, DocumentId)>,
) -> MutationSideEffects {
    let mut effects = MutationSideEffects::none();
    for (entry_id, user_id, document_id) in entries {
        effects.events.push(library_entry_permanently_deleted(
            user_id,
            entry_id,
            document_id,
        ));
        effects
            .outbox
            .push(search_reindex_document_outbox(document_id, Utc::now()));
    }
    effects
}

impl PgLibraryRepository {
    pub(super) async fn set_triage_state_impl(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        state: TriageState,
        mut effects: MutationSideEffects,
    ) -> Result<LibraryEntry, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_library_error)?;
        let row = sqlx::query_as!(
            LibraryEntryRow,
            "UPDATE library_entries SET triage_state = $3, updated_at = now() \
             WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL \
             RETURNING id, user_id, document_id, saved_at, triage_state, is_favorite, \
                       is_shortlisted, deleted_at, source, source_delivery_id, created_at, \
                       updated_at",
            id.into_uuid(),
            user_id.into_uuid(),
            state.as_str(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_library_error)?
        .ok_or_else(|| not_found(id))?;

        let entry = row.into_entry()?;
        effects
            .events
            .insert(0, library_entry_triaged(user_id, &entry));
        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_library_error)?;
        Ok(entry)
    }

    pub(super) async fn toggle_favorite_impl(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        mut effects: MutationSideEffects,
    ) -> Result<LibraryEntry, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_library_error)?;
        let row = sqlx::query_as!(
            LibraryEntryRow,
            "UPDATE library_entries SET is_favorite = NOT is_favorite, updated_at = now() \
             WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL \
             RETURNING id, user_id, document_id, saved_at, triage_state, is_favorite, \
                       is_shortlisted, deleted_at, source, source_delivery_id, created_at, \
                       updated_at",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_library_error)?
        .ok_or_else(|| not_found(id))?;

        let entry = row.into_entry()?;
        effects
            .events
            .insert(0, library_entry_favorite_changed(user_id, &entry));
        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_library_error)?;
        Ok(entry)
    }

    pub(super) async fn toggle_shortlist_impl(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<LibraryEntry, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_library_error)?;
        let row = sqlx::query_as!(
            LibraryEntryRow,
            "UPDATE library_entries SET is_shortlisted = NOT is_shortlisted, updated_at = now() \
             WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL \
             RETURNING id, user_id, document_id, saved_at, triage_state, is_favorite, \
                       is_shortlisted, deleted_at, source, source_delivery_id, created_at, \
                       updated_at",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_library_error)?
        .ok_or_else(|| not_found(id))?;

        // Shortlist toggles emit no domain event by design; only caller-supplied effects apply.
        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_library_error)?;
        row.into_entry()
    }

    pub(super) async fn soft_delete_impl(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        mut effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_library_error)?;
        let document_id = sqlx::query_scalar!(
            "UPDATE library_entries SET deleted_at = now(), updated_at = now() \
             WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL \
             RETURNING document_id",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_library_error)?;

        // A missing, already-deleted, or another user's entry must surface as not-found, not a
        // silent success (avoids reporting a no-op delete as 204).
        let Some(document_id) = document_id else {
            return Err(not_found(id));
        };

        let document_id = DocumentId::from_uuid(document_id);
        effects
            .events
            .insert(0, library_entry_trashed(user_id, id, document_id));
        effects
            .outbox
            .push(search_reindex_document_outbox(document_id, Utc::now()));
        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_library_error)?;
        Ok(())
    }

    pub(super) async fn count_active_impl(&self, user_id: UserId) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM library_entries WHERE user_id = $1 AND deleted_at IS NULL",
            user_id.into_uuid(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_library_error)?;

        Ok(count.unwrap_or(0))
    }

    pub(super) async fn count_trashed_impl(&self, user_id: UserId) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM library_entries WHERE user_id = $1 AND deleted_at IS NOT NULL",
            user_id.into_uuid(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_library_error)?;

        Ok(count.unwrap_or(0))
    }

    pub(super) async fn purge_expired_trash_impl(
        &self,
        retention_days: i64,
    ) -> Result<u64, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_library_error)?;
        let purged = sqlx::query!(
            "DELETE FROM library_entries \
             WHERE deleted_at IS NOT NULL \
               AND deleted_at < now() - ($1::int * interval '1 day') \
             RETURNING id, user_id, document_id",
            retention_days as i32,
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(map_library_error)?;

        let effects = permanent_deletion_effects(purged.iter().map(|row| {
            (
                LibraryEntryId::from_uuid(row.id),
                UserId::from_uuid(row.user_id),
                DocumentId::from_uuid(row.document_id),
            )
        }));
        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_library_error)?;

        Ok(purged.len() as u64)
    }

    /// Restore a soft-deleted entry. The partial unique `uq_library_entries_user_document_active`
    /// permits trashed rows plus one active row for the same `(user_id, document_id)`, so a blind
    /// `deleted_at = NULL` could violate it. Lock the whole `(user_id, document_id)`
    /// group `FOR UPDATE` so a concurrent restore of a different trashed sibling — or a concurrent
    /// save — serializes; then if an active entry already exists (the target itself, or a newer save
    /// that superseded it) return that entry instead of restoring into a conflict; otherwise clear
    /// `deleted_at` on the target.
    pub(super) async fn restore_impl(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        mut effects: MutationSideEffects,
    ) -> Result<LibraryEntry, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_library_error)?;

        // Resolve the group key WITHOUT a row lock first; taking a lock on the target row before
        // the group lock would let two concurrent restores of different trashed siblings deadlock
        // (each holding its own row, both waiting on the group).
        let target = sqlx::query!(
            "SELECT document_id FROM library_entries WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_library_error)?
        .ok_or_else(|| not_found(id))?;

        // Lock every row in the group in a deterministic (id) order so concurrent restore/save for
        // the same (user_id, document_id) serialize without deadlock and cannot both clear
        // deleted_at into uq_library_entries_user_document_active.
        sqlx::query!(
            "SELECT id FROM library_entries \
             WHERE user_id = $1 AND document_id = $2 ORDER BY id FOR UPDATE",
            user_id.into_uuid(),
            target.document_id,
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(map_library_error)?;

        let active = sqlx::query_as!(
            LibraryEntryRow,
            "SELECT id, user_id, document_id, saved_at, triage_state, is_favorite, is_shortlisted, \
                    deleted_at, source, source_delivery_id, created_at, updated_at \
             FROM library_entries \
             WHERE user_id = $1 AND document_id = $2 AND deleted_at IS NULL \
             ORDER BY updated_at DESC LIMIT 1",
            user_id.into_uuid(),
            target.document_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_library_error)?;

        let entry = if let Some(active) = active {
            active.into_entry()?
        } else {
            // The target may have been purged between the lookup and the group lock; treat a
            // vanished target as not-found rather than a 500.
            sqlx::query_as!(
                LibraryEntryRow,
                "UPDATE library_entries SET deleted_at = NULL, saved_at = now(), updated_at = now() \
                 WHERE id = $1 AND user_id = $2 \
                 RETURNING id, user_id, document_id, saved_at, triage_state, is_favorite, \
                           is_shortlisted, deleted_at, source, source_delivery_id, created_at, \
                           updated_at",
                id.into_uuid(),
                user_id.into_uuid(),
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_library_error)?
            .ok_or_else(|| not_found(id))?
            .into_entry()?
        };

        effects
            .events
            .insert(0, library_entry_restored(user_id, &entry));
        effects.outbox.push(search_reindex_document_outbox(
            entry.document_id,
            Utc::now(),
        ));
        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_library_error)?;
        Ok(entry)
    }

    /// Permanently remove a library entry. The document is the parent of this FK, so purging an
    /// entry never deletes the document or its authored capabilities (highlights/notes/progress/
    /// Mila); collection/tag membership for this entry cascades away with it (AC#6).
    pub(super) async fn purge_impl(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        mut effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_library_error)?;
        let document_id = sqlx::query_scalar!(
            "DELETE FROM library_entries WHERE id = $1 AND user_id = $2 RETURNING document_id",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_library_error)?;

        let Some(document_id) = document_id else {
            return Err(not_found(id));
        };
        let document_id = DocumentId::from_uuid(document_id);

        effects.events.insert(
            0,
            library_entry_permanently_deleted(user_id, id, document_id),
        );
        effects
            .outbox
            .push(search_reindex_document_outbox(document_id, Utc::now()));
        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_library_error)?;
        Ok(())
    }

    /// Empty the user's trash: permanently remove every soft-deleted entry in one transaction.
    /// Same parent-direction FK semantics as `purge_impl` (documents and authored capabilities
    /// survive; membership join tables cascade), with one `library_entry.permanently_deleted`
    /// event fanned out per purged row.
    pub(super) async fn purge_all_trashed_impl(&self, user_id: UserId) -> Result<u64, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_library_error)?;
        let purged = sqlx::query!(
            "DELETE FROM library_entries \
             WHERE user_id = $1 AND deleted_at IS NOT NULL \
             RETURNING id, document_id",
            user_id.into_uuid(),
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(map_library_error)?;

        let effects = permanent_deletion_effects(purged.iter().map(|row| {
            (
                LibraryEntryId::from_uuid(row.id),
                user_id,
                DocumentId::from_uuid(row.document_id),
            )
        }));
        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_library_error)?;
        Ok(purged.len() as u64)
    }
}
