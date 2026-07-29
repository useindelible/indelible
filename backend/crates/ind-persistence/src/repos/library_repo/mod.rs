mod mutations;
mod queries;
pub(crate) mod rows;
pub(crate) mod tx_writes;

use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::library::{LibraryRepository, LibraryScopeCounts};
use ind_application::repos::{Cursor, Page};
use ind_domain::{
    DocumentId, LibraryEntry, LibraryEntryId, LibraryEntryWithDocument, TriageState, UserId,
};

pub struct PgLibraryRepository {
    pool: PgPool,
}

impl PgLibraryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl LibraryRepository for PgLibraryRepository {
    async fn insert_entry(&self, entry: LibraryEntry) -> Result<LibraryEntry, AppError> {
        self.insert_entry_impl(entry).await
    }

    async fn find_by_id(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
    ) -> Result<Option<LibraryEntryWithDocument>, AppError> {
        self.find_by_id_impl(id, user_id).await
    }

    async fn find_active_by_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<LibraryEntry>, AppError> {
        self.find_active_by_document_impl(user_id, document_id)
            .await
    }

    async fn find_active_by_canonical_url(
        &self,
        user_id: UserId,
        canonical_url: &str,
    ) -> Result<Option<LibraryEntryWithDocument>, AppError> {
        self.find_active_by_canonical_url_impl(user_id, canonical_url)
            .await
    }

    async fn list_by_user(
        &self,
        user_id: UserId,
        triage: Option<TriageState>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        self.list_by_user_impl(user_id, triage, cursor, limit).await
    }

    async fn set_triage_state(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        state: TriageState,
        effects: MutationSideEffects,
    ) -> Result<LibraryEntry, AppError> {
        self.set_triage_state_impl(id, user_id, state, effects)
            .await
    }

    async fn toggle_favorite(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<LibraryEntry, AppError> {
        self.toggle_favorite_impl(id, user_id, effects).await
    }

    async fn toggle_shortlist(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<LibraryEntry, AppError> {
        self.toggle_shortlist_impl(id, user_id, effects).await
    }

    async fn soft_delete(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        self.soft_delete_impl(id, user_id, effects).await
    }

    async fn restore(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<LibraryEntry, AppError> {
        self.restore_impl(id, user_id, effects).await
    }

    async fn purge(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        self.purge_impl(id, user_id, effects).await
    }

    async fn purge_all_trashed(&self, user_id: UserId) -> Result<u64, AppError> {
        self.purge_all_trashed_impl(user_id).await
    }

    async fn purge_expired_trash(&self, retention_days: i64) -> Result<u64, AppError> {
        self.purge_expired_trash_impl(retention_days).await
    }

    async fn list_trashed(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        self.list_trashed_impl(user_id, cursor, limit).await
    }

    async fn count_active(&self, user_id: UserId) -> Result<i64, AppError> {
        self.count_active_impl(user_id).await
    }

    async fn count_trashed(&self, user_id: UserId) -> Result<i64, AppError> {
        self.count_trashed_impl(user_id).await
    }

    async fn scope_counts(
        &self,
        user_id: UserId,
        triage: Option<TriageState>,
    ) -> Result<LibraryScopeCounts, AppError> {
        self.scope_counts_impl(user_id, triage).await
    }
}
