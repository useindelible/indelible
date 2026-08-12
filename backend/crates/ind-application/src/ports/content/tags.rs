use super::*;

pub struct CreateTagRequest {
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<TagId>,
}

pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub color: Option<Option<String>>,
    pub parent_id: Option<Option<TagId>>,
}

pub trait TagOperations: Send + Sync {
    fn create_tag(
        &self,
        user_id: UserId,
        req: CreateTagRequest,
    ) -> BoxFuture<'_, Result<TagWithMeta, AppError>>;

    fn get_tag(&self, user_id: UserId, id: TagId) -> BoxFuture<'_, Result<TagWithMeta, AppError>>;

    fn list_tags(
        &self,
        user_id: UserId,
        cursor: Option<String>,
        limit: Option<u32>,
        scope: Option<String>,
    ) -> BoxFuture<'_, Result<Page<TagWithMeta>, AppError>>;

    fn update_tag(
        &self,
        user_id: UserId,
        id: TagId,
        req: UpdateTagRequest,
    ) -> BoxFuture<'_, Result<TagWithMeta, AppError>>;

    fn delete_tag(&self, user_id: UserId, id: TagId) -> BoxFuture<'_, Result<(), AppError>>;

    fn merge_tags(
        &self,
        user_id: UserId,
        source_ids: Vec<TagId>,
        target_id: TagId,
    ) -> BoxFuture<'_, Result<TagWithMeta, AppError>>;

    fn list_tag_highlights(
        &self,
        user_id: UserId,
        tag_id: TagId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<TaggedHighlight>, AppError>>;

    fn set_library_entry_tags(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
        document_id: DocumentId,
        tag_names: Vec<String>,
    ) -> BoxFuture<'_, Result<Vec<Tag>, AppError>>;

    fn list_library_entry_tags(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<Vec<Tag>, AppError>>;

    fn list_tag_library_entries(
        &self,
        user_id: UserId,
        tag_id: TagId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<LibraryEntryWithDocument>, AppError>>;
}
