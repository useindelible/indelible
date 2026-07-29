use std::collections::HashMap;

use crate::error::AppError;
use crate::repos::event::MutationSideEffects;
use crate::repos::{Cursor, Page};
use ind_domain::{
    HighlightId, LibraryEntryId, LibraryEntryWithDocument, Tag, TagAlias, TagId, TagSource, UserId,
};

#[async_trait::async_trait]
pub trait TagRepository: Send + Sync {
    async fn find_by_id(&self, id: TagId) -> Result<Option<Tag>, AppError>;
    async fn create(&self, tag: Tag) -> Result<Tag, AppError>;
    async fn delete(&self, id: TagId) -> Result<(), AppError>;
    async fn list_by_user(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<Tag>, AppError>;
    async fn find_by_name(&self, user_id: UserId, name: &str) -> Result<Option<Tag>, AppError>;
    async fn find_or_create_by_name(&self, user_id: UserId, name: &str) -> Result<Tag, AppError>;
    async fn replace_for_highlight(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
        tag_ids: &[TagId],
        effects: MutationSideEffects,
    ) -> Result<(), AppError>;

    async fn list_by_library_entry(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
    ) -> Result<Vec<Tag>, AppError>;

    async fn replace_for_library_entry(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
        tag_ids: &[TagId],
        effects: MutationSideEffects,
    ) -> Result<(), AppError>;

    async fn replace_for_library_entry_with_source(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
        tag_ids: &[TagId],
        source: TagSource,
        effects: MutationSideEffects,
    ) -> Result<(), AppError>;

    async fn list_tag_library_entries(
        &self,
        tag_id: TagId,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError>;

    async fn find_by_id_for_user(
        &self,
        id: TagId,
        user_id: UserId,
    ) -> Result<Option<Tag>, AppError>;
    async fn delete_for_user(&self, id: TagId, user_id: UserId) -> Result<(), AppError>;
    async fn update_name(&self, id: TagId, user_id: UserId, name: &str) -> Result<Tag, AppError>;
    async fn update_color(
        &self,
        id: TagId,
        user_id: UserId,
        color: Option<&str>,
    ) -> Result<Tag, AppError>;
    async fn update_parent(
        &self,
        id: TagId,
        user_id: UserId,
        parent_id: Option<TagId>,
    ) -> Result<Tag, AppError>;
    async fn merge_tags(
        &self,
        source_ids: &[TagId],
        target_id: TagId,
        user_id: UserId,
    ) -> Result<Tag, AppError>;
    async fn list_with_counts(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
        scope: Option<&str>,
    ) -> Result<Page<(Tag, i64, i64)>, AppError>;
    async fn list_aliases(&self, tag_id: TagId) -> Result<Vec<TagAlias>, AppError>;
    async fn count_items_for_tag(&self, tag_id: TagId) -> Result<i64, AppError>;
    async fn count_highlights_for_tag(&self, tag_id: TagId) -> Result<i64, AppError>;
    async fn list_tag_highlights(
        &self,
        tag_id: TagId,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<ind_domain::Highlight>, AppError>;
    async fn list_aliases_for_tags(
        &self,
        tag_ids: &[TagId],
    ) -> Result<HashMap<TagId, Vec<String>>, AppError>;
}
