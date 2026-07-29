use crate::error::AppError;
use crate::repos::{Cursor, Page};
use ind_domain::{Collection, CollectionId, LibraryEntryId, LibraryEntryWithDocument, UserId};

#[allow(clippy::too_many_arguments)]
#[async_trait::async_trait]
pub trait CollectionRepository: Send + Sync {
    async fn find_by_id(
        &self,
        user_id: UserId,
        id: CollectionId,
    ) -> Result<Option<Collection>, AppError>;
    async fn find_by_parent_and_name(
        &self,
        user_id: UserId,
        parent_id: Option<CollectionId>,
        name: &str,
    ) -> Result<Option<Collection>, AppError>;
    async fn create(&self, collection: Collection) -> Result<Collection, AppError>;
    async fn delete(&self, user_id: UserId, id: CollectionId) -> Result<(), AppError>;
    async fn list_by_user(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<Collection>, AppError>;
    async fn list_children(
        &self,
        user_id: UserId,
        parent_id: CollectionId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<Collection>, AppError>;
    async fn update_fields(
        &self,
        user_id: UserId,
        id: CollectionId,
        name: Option<&str>,
        description: Option<Option<&str>>,
        icon: Option<Option<&str>>,
        color: Option<Option<&str>>,
        sort_order: Option<i32>,
        parent_id: Option<Option<CollectionId>>,
    ) -> Result<Collection, AppError>;

    async fn add_library_entry_to_collection(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        library_entry_id: LibraryEntryId,
    ) -> Result<(), AppError>;

    async fn remove_library_entry_from_collection(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        library_entry_id: LibraryEntryId,
    ) -> Result<(), AppError>;

    /// Collection contents as saved library entries joined to their documents, newest-added first.
    async fn list_collection_entries(
        &self,
        collection_id: CollectionId,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError>;

    /// Count of active library entries in the collection (the canonical Library membership count).
    async fn count_items(&self, collection_id: CollectionId) -> Result<i64, AppError>;

    async fn list_by_user_with_counts(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<(Collection, i64)>, AppError>;

    async fn list_children_with_counts(
        &self,
        user_id: UserId,
        parent_id: CollectionId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<(Collection, i64)>, AppError>;
}
