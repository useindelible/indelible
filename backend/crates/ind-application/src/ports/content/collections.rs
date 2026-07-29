use super::*;

pub struct CreateCollectionRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub parent_id: Option<CollectionId>,
}

pub struct UpdateCollectionRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub sort_order: Option<i32>,
    pub parent_id: Option<Option<CollectionId>>,
}

pub trait CollectionOperations: Send + Sync {
    fn create_collection(
        &self,
        user_id: UserId,
        req: CreateCollectionRequest,
    ) -> BoxFuture<'_, Result<CollectionWithCount, AppError>>;

    fn get_collection(
        &self,
        user_id: UserId,
        id: CollectionId,
    ) -> BoxFuture<'_, Result<CollectionWithCount, AppError>>;

    fn list_collections(
        &self,
        user_id: UserId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<CollectionWithCount>, AppError>>;

    fn list_children(
        &self,
        user_id: UserId,
        parent_id: CollectionId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<CollectionWithCount>, AppError>>;

    fn update_collection(
        &self,
        user_id: UserId,
        id: CollectionId,
        req: UpdateCollectionRequest,
    ) -> BoxFuture<'_, Result<CollectionWithCount, AppError>>;

    fn delete_collection(
        &self,
        user_id: UserId,
        id: CollectionId,
    ) -> BoxFuture<'_, Result<(), AppError>>;

    fn add_entry(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        library_entry_id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<(), AppError>>;

    fn remove_entry(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        library_entry_id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<(), AppError>>;

    fn list_entries(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<LibraryEntryWithDocument>, AppError>>;
}
