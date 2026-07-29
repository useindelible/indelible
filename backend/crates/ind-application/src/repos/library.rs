use crate::error::AppError;
use crate::repos::event::MutationSideEffects;
use crate::repos::{Cursor, Page};
use ind_domain::{
    DocumentId, DocumentType, LibraryEntry, LibraryEntryId, LibraryEntryWithDocument, TriageState,
    UserId,
};

/// Saved entries of one document type within a library scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryItemTypeCount {
    pub item_type: DocumentType,
    pub count: i64,
}

/// Aggregate counts for one library scope (all saved entries, or one triage state).
///
/// Read-state buckets mirror `user_document_state`: `done` is a set `finished_at`, `reading` is
/// any recorded progress without a finish, and everything else — including documents that were
/// never opened and so have no state row — is `unread`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LibraryScopeCounts {
    pub unread: i64,
    pub reading: i64,
    pub done: i64,
    pub by_item_type: Vec<LibraryItemTypeCount>,
}

impl LibraryScopeCounts {
    pub fn total(&self) -> i64 {
        self.unread + self.reading + self.done
    }
}

/// Repository for saved-document membership.
/// See docs/document-feed-library-architecture.md (library_entries).
///
/// Reads always join `documents` and never scan the feed firehose.
/// `insert_entry` is a standalone, self-committing persistence primitive, not the
/// user-facing save path and not transaction-composable. The atomic save flow (hide
/// deliveries, outbox, domain events) is implemented by the document lifecycle transaction.
#[async_trait::async_trait]
pub trait LibraryRepository: Send + Sync {
    async fn insert_entry(&self, entry: LibraryEntry) -> Result<LibraryEntry, AppError>;

    async fn find_by_id(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
    ) -> Result<Option<LibraryEntryWithDocument>, AppError>;

    async fn find_active_by_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<LibraryEntry>, AppError>;

    async fn find_active_by_canonical_url(
        &self,
        user_id: UserId,
        canonical_url: &str,
    ) -> Result<Option<LibraryEntryWithDocument>, AppError>;

    async fn list_by_user(
        &self,
        user_id: UserId,
        triage: Option<TriageState>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError>;

    /// Targeted triage transition (no whole-row write). Errors if the entry is missing or
    /// soft-deleted. Builds and commits `library_entry.triaged`/`.archived` from the returned row
    /// in-tx; `effects` appends any extra caller-supplied events/outbox.
    async fn set_triage_state(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        state: TriageState,
        effects: MutationSideEffects,
    ) -> Result<LibraryEntry, AppError>;

    /// Flip `is_favorite` in place. Builds and commits `library_entry.favorited` (carrying the
    /// resulting `is_favorite`) from the returned row in-tx.
    async fn toggle_favorite(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<LibraryEntry, AppError>;

    /// Flip `is_shortlisted` in place. Emits no domain event by design (the catalog has no
    /// shortlist event); `effects` is still applied for any caller-supplied side effects.
    async fn toggle_shortlist(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<LibraryEntry, AppError>;

    /// Soft-delete an active entry (`deleted_at = now()`). No-op if already deleted/missing.
    /// Builds and commits `library_entry.trashed` from the deleted row in-tx.
    async fn soft_delete(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<(), AppError>;

    /// Restore a soft-deleted entry. Conflict-safe: if an active entry already exists for the same
    /// `(user_id, document_id)`, that entry is returned instead of restoring into a unique-index
    /// violation. Builds and commits `library_entry.restored` from the resulting row in-tx.
    async fn restore(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<LibraryEntry, AppError>;

    /// Permanently delete a library entry. Never deletes the underlying document or its authored
    /// capabilities; collection/tag membership for the entry cascades away. Builds and commits
    /// `library_entry.permanently_deleted` from the deleted row in-tx.
    async fn purge(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<(), AppError>;

    /// Permanently delete every trashed (`deleted_at IS NOT NULL`) entry for one user in one
    /// transaction. Never deletes the underlying documents or their authored capabilities;
    /// collection/tag membership cascades away. Builds and commits one
    /// `library_entry.permanently_deleted` event per purged row. Returns the number purged.
    async fn purge_all_trashed(&self, user_id: UserId) -> Result<u64, AppError>;

    /// Permanently delete trashed library entries whose retention window has expired. The
    /// underlying documents and authored capabilities remain intact. Commits one
    /// `library_entry.permanently_deleted` event and search reindex outbox row per purged entry.
    async fn purge_expired_trash(&self, retention_days: i64) -> Result<u64, AppError>;

    /// Trashed (soft-deleted) entries joined to their documents, newest-saved first.
    async fn list_trashed(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError>;

    /// Count of the user's active (`deleted_at IS NULL`) library entries. Quotas/counts use
    /// this, never a document or feed-delivery count.
    async fn count_active(&self, user_id: UserId) -> Result<i64, AppError>;

    /// Count of the user's trashed (`deleted_at IS NOT NULL`) library entries.
    async fn count_trashed(&self, user_id: UserId) -> Result<i64, AppError>;

    /// Read-state and item-type breakdown of the user's active entries, optionally narrowed to
    /// one triage state. Mirrors the `list_by_user` visibility predicate.
    async fn scope_counts(
        &self,
        user_id: UserId,
        triage: Option<TriageState>,
    ) -> Result<LibraryScopeCounts, AppError>;
}
