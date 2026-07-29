use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::collection::CollectionRepository;
use ind_application::repos::{Cursor, Page};
use ind_domain::{Collection, CollectionId, LibraryEntryId, LibraryEntryWithDocument, UserId};

mod collections;
mod counts;
mod entries;
mod rows;

use collections::CollectionFieldUpdate;

pub struct PgCollectionRepository {
    pool: PgPool,
}

impl PgCollectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("collection", "duplicate collection", err)
}

#[async_trait::async_trait]
impl CollectionRepository for PgCollectionRepository {
    async fn find_by_id(
        &self,
        user_id: UserId,
        id: CollectionId,
    ) -> Result<Option<Collection>, AppError> {
        self.find_by_id_query(user_id, id).await
    }

    async fn find_by_parent_and_name(
        &self,
        user_id: UserId,
        parent_id: Option<CollectionId>,
        name: &str,
    ) -> Result<Option<Collection>, AppError> {
        self.find_by_parent_and_name_query(user_id, parent_id, name)
            .await
    }

    async fn create(&self, collection: Collection) -> Result<Collection, AppError> {
        self.create_collection(collection).await
    }

    async fn delete(&self, user_id: UserId, id: CollectionId) -> Result<(), AppError> {
        self.delete_collection(user_id, id).await
    }

    async fn list_by_user(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<Collection>, AppError> {
        self.list_by_user_query(user_id, cursor, limit).await
    }

    async fn list_children(
        &self,
        user_id: UserId,
        parent_id: CollectionId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<Collection>, AppError> {
        self.list_children_query(user_id, parent_id, cursor, limit)
            .await
    }

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
    ) -> Result<Collection, AppError> {
        self.update_collection_fields(CollectionFieldUpdate {
            user_id,
            id,
            name,
            description,
            icon,
            color,
            sort_order,
            parent_id,
        })
        .await
    }

    async fn add_library_entry_to_collection(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        library_entry_id: LibraryEntryId,
    ) -> Result<(), AppError> {
        self.add_library_entry_link(user_id, collection_id, library_entry_id)
            .await
    }

    async fn remove_library_entry_from_collection(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        library_entry_id: LibraryEntryId,
    ) -> Result<(), AppError> {
        self.remove_library_entry_link(user_id, collection_id, library_entry_id)
            .await
    }

    async fn list_collection_entries(
        &self,
        collection_id: CollectionId,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        self.list_collection_entries_query(collection_id, user_id, cursor, limit)
            .await
    }

    async fn count_items(&self, collection_id: CollectionId) -> Result<i64, AppError> {
        self.count_library_entries_query(collection_id).await
    }

    async fn list_by_user_with_counts(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<(Collection, i64)>, AppError> {
        self.list_by_user_with_counts_query(user_id, cursor, limit)
            .await
    }

    async fn list_children_with_counts(
        &self,
        user_id: UserId,
        parent_id: CollectionId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<(Collection, i64)>, AppError> {
        self.list_children_with_counts_query(user_id, parent_id, cursor, limit)
            .await
    }
}
