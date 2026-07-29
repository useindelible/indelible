use std::sync::Arc;

use chrono::Utc;
use futures::future::BoxFuture;
use ind_domain::{
    Collection, CollectionId, DomainError, LibraryEntryId, LibraryEntryWithDocument, UserId,
};

use crate::AppError;
use crate::ports::{CollectionOperations, CreateCollectionRequest, UpdateCollectionRequest};
use crate::repos::collection::CollectionRepository;
use crate::repos::{Cursor, Page};

#[derive(Debug, Clone)]
pub struct CollectionWithCount {
    pub collection: Collection,
    pub item_count: i64,
}

pub struct CreateCollectionInput {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub parent_id: Option<CollectionId>,
}

pub struct UpdateCollectionInput {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub sort_order: Option<i32>,
    pub parent_id: Option<Option<CollectionId>>,
}

pub struct CollectionService {
    collection_repo: Arc<dyn CollectionRepository>,
}

impl CollectionService {
    pub fn new(collection_repo: Arc<dyn CollectionRepository>) -> Self {
        Self { collection_repo }
    }

    pub async fn create(
        &self,
        user_id: UserId,
        input: CreateCollectionInput,
    ) -> Result<CollectionWithCount, AppError> {
        let now = Utc::now();
        let collection = Collection {
            id: CollectionId::new(),
            user_id,
            parent_id: input.parent_id,
            name: input.name,
            description: input.description,
            icon: input.icon,
            color: input.color,
            sort_order: input.sort_order.unwrap_or(0),
            rss_token: None,
            is_pinned: false,
            created_at: now,
            updated_at: now,
        };

        let created = self.collection_repo.create(collection).await?;
        Ok(CollectionWithCount {
            collection: created,
            item_count: 0,
        })
    }

    pub async fn get(
        &self,
        user_id: UserId,
        id: CollectionId,
    ) -> Result<CollectionWithCount, AppError> {
        let collection = self
            .collection_repo
            .find_by_id(user_id, id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "collection",
                    id: id.to_string(),
                })
            })?;

        let item_count = self.collection_repo.count_items(id).await?;

        Ok(CollectionWithCount {
            collection,
            item_count,
        })
    }

    pub async fn list(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<CollectionWithCount>, AppError> {
        let page = self
            .collection_repo
            .list_by_user_with_counts(user_id, cursor, limit)
            .await?;

        let items = page
            .items
            .into_iter()
            .map(|(collection, item_count)| CollectionWithCount {
                collection,
                item_count,
            })
            .collect();

        Ok(Page {
            items,
            next_cursor: page.next_cursor,
        })
    }

    pub async fn list_children(
        &self,
        user_id: UserId,
        parent_id: CollectionId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<CollectionWithCount>, AppError> {
        let page = self
            .collection_repo
            .list_children_with_counts(user_id, parent_id, cursor, limit)
            .await?;

        let items = page
            .items
            .into_iter()
            .map(|(collection, item_count)| CollectionWithCount {
                collection,
                item_count,
            })
            .collect();

        Ok(Page {
            items,
            next_cursor: page.next_cursor,
        })
    }

    pub async fn update_fields(
        &self,
        user_id: UserId,
        id: CollectionId,
        input: UpdateCollectionInput,
    ) -> Result<CollectionWithCount, AppError> {
        let updated = self
            .collection_repo
            .update_fields(
                user_id,
                id,
                input.name.as_deref(),
                input.description.as_ref().map(|d| d.as_deref()),
                input.icon.as_ref().map(|i| i.as_deref()),
                input.color.as_ref().map(|c| c.as_deref()),
                input.sort_order,
                input.parent_id,
            )
            .await?;

        let item_count = self.collection_repo.count_items(id).await?;

        Ok(CollectionWithCount {
            collection: updated,
            item_count,
        })
    }

    pub async fn delete(&self, user_id: UserId, id: CollectionId) -> Result<(), AppError> {
        self.collection_repo.delete(user_id, id).await
    }

    pub async fn add_entry(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        library_entry_id: LibraryEntryId,
    ) -> Result<(), AppError> {
        self.ensure_collection(user_id, collection_id).await?;
        self.collection_repo
            .add_library_entry_to_collection(user_id, collection_id, library_entry_id)
            .await
    }

    pub async fn remove_entry(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        library_entry_id: LibraryEntryId,
    ) -> Result<(), AppError> {
        self.ensure_collection(user_id, collection_id).await?;
        self.collection_repo
            .remove_library_entry_from_collection(user_id, collection_id, library_entry_id)
            .await
    }

    pub async fn list_entries(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        self.ensure_collection(user_id, collection_id).await?;
        self.collection_repo
            .list_collection_entries(collection_id, user_id, cursor, limit)
            .await
    }

    async fn ensure_collection(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
    ) -> Result<(), AppError> {
        self.collection_repo
            .find_by_id(user_id, collection_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "collection",
                    id: collection_id.to_string(),
                })
            })?;
        Ok(())
    }
}

impl CollectionOperations for CollectionService {
    fn create_collection(
        &self,
        user_id: UserId,
        request: CreateCollectionRequest,
    ) -> BoxFuture<'_, Result<CollectionWithCount, AppError>> {
        Box::pin(self.create(
            user_id,
            CreateCollectionInput {
                name: request.name,
                description: request.description,
                icon: request.icon,
                color: request.color,
                sort_order: request.sort_order,
                parent_id: request.parent_id,
            },
        ))
    }

    fn get_collection(
        &self,
        user_id: UserId,
        id: CollectionId,
    ) -> BoxFuture<'_, Result<CollectionWithCount, AppError>> {
        Box::pin(self.get(user_id, id))
    }

    fn list_collections(
        &self,
        user_id: UserId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<CollectionWithCount>, AppError>> {
        Box::pin(self.list(user_id, cursor.map(Cursor), limit.unwrap_or(50)))
    }

    fn list_children(
        &self,
        user_id: UserId,
        parent_id: CollectionId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<CollectionWithCount>, AppError>> {
        Box::pin(self.list_children(user_id, parent_id, cursor.map(Cursor), limit.unwrap_or(50)))
    }

    fn update_collection(
        &self,
        user_id: UserId,
        id: CollectionId,
        request: UpdateCollectionRequest,
    ) -> BoxFuture<'_, Result<CollectionWithCount, AppError>> {
        Box::pin(self.update_fields(
            user_id,
            id,
            UpdateCollectionInput {
                name: request.name,
                description: request.description,
                icon: request.icon,
                color: request.color,
                sort_order: request.sort_order,
                parent_id: request.parent_id,
            },
        ))
    }

    fn delete_collection(
        &self,
        user_id: UserId,
        id: CollectionId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.delete(user_id, id))
    }

    fn add_entry(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        library_entry_id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.add_entry(user_id, collection_id, library_entry_id))
    }

    fn remove_entry(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        library_entry_id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.remove_entry(user_id, collection_id, library_entry_id))
    }

    fn list_entries(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<LibraryEntryWithDocument>, AppError>> {
        Box::pin(self.list_entries(
            user_id,
            collection_id,
            cursor.map(Cursor),
            limit.unwrap_or(50),
        ))
    }
}
