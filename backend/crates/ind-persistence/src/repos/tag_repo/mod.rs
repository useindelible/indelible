use std::collections::HashMap;

use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::tag::{TagRepository, TaggedHighlight};
use ind_application::repos::{Cursor, Page};
use ind_domain::{
    HighlightId, LibraryEntryId, LibraryEntryWithDocument, Tag, TagAlias, TagId, TagSource, UserId,
};

mod base;
mod library_relations;
mod listing;
mod merge;
mod relations;
mod rows;

pub struct PgTagRepository {
    pub(super) pool: PgPool,
}

impl PgTagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub(super) fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("tag", "duplicate tag", err)
}

#[async_trait::async_trait]
impl TagRepository for PgTagRepository {
    async fn find_by_id(&self, id: TagId) -> Result<Option<Tag>, AppError> {
        PgTagRepository::find_by_id_impl(self, id).await
    }

    async fn create(&self, tag: Tag) -> Result<Tag, AppError> {
        PgTagRepository::create_impl(self, tag).await
    }

    async fn delete(&self, id: TagId) -> Result<(), AppError> {
        PgTagRepository::delete_impl(self, id).await
    }

    async fn list_by_user(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<Tag>, AppError> {
        PgTagRepository::list_by_user_impl(self, user_id, cursor, limit).await
    }

    async fn find_by_name(&self, user_id: UserId, name: &str) -> Result<Option<Tag>, AppError> {
        PgTagRepository::find_by_name_impl(self, user_id, name).await
    }

    async fn find_or_create_by_name(&self, user_id: UserId, name: &str) -> Result<Tag, AppError> {
        PgTagRepository::find_or_create_by_name_impl(self, user_id, name).await
    }

    async fn replace_for_highlight(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
        tag_ids: &[TagId],
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        PgTagRepository::replace_for_highlight_impl(self, user_id, highlight_id, tag_ids, effects)
            .await
    }

    async fn list_by_library_entry(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
    ) -> Result<Vec<Tag>, AppError> {
        PgTagRepository::list_by_library_entry_impl(self, user_id, library_entry_id).await
    }

    async fn replace_for_library_entry(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
        tag_ids: &[TagId],
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        PgTagRepository::replace_for_library_entry_impl(
            self,
            user_id,
            library_entry_id,
            tag_ids,
            effects,
        )
        .await
    }

    async fn replace_for_library_entry_with_source(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
        tag_ids: &[TagId],
        source: TagSource,
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        PgTagRepository::replace_for_library_entry_with_source_impl(
            self,
            user_id,
            library_entry_id,
            tag_ids,
            source,
            effects,
        )
        .await
    }

    async fn list_tag_library_entries(
        &self,
        tag_id: TagId,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        PgTagRepository::list_tag_library_entries_impl(self, tag_id, user_id, cursor, limit).await
    }

    async fn find_by_id_for_user(
        &self,
        id: TagId,
        user_id: UserId,
    ) -> Result<Option<Tag>, AppError> {
        PgTagRepository::find_by_id_for_user_impl(self, id, user_id).await
    }

    async fn delete_for_user(&self, id: TagId, user_id: UserId) -> Result<(), AppError> {
        PgTagRepository::delete_for_user_impl(self, id, user_id).await
    }

    async fn update_name(&self, id: TagId, user_id: UserId, name: &str) -> Result<Tag, AppError> {
        PgTagRepository::update_name_impl(self, id, user_id, name).await
    }

    async fn update_color(
        &self,
        id: TagId,
        user_id: UserId,
        color: Option<&str>,
    ) -> Result<Tag, AppError> {
        PgTagRepository::update_color_impl(self, id, user_id, color).await
    }

    async fn update_parent(
        &self,
        id: TagId,
        user_id: UserId,
        parent_id: Option<TagId>,
    ) -> Result<Tag, AppError> {
        PgTagRepository::update_parent_impl(self, id, user_id, parent_id).await
    }

    async fn merge_tags(
        &self,
        source_ids: &[TagId],
        target_id: TagId,
        user_id: UserId,
    ) -> Result<Tag, AppError> {
        PgTagRepository::merge_tags_impl(self, source_ids, target_id, user_id).await
    }

    async fn list_with_counts(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
        scope: Option<&str>,
    ) -> Result<Page<(Tag, i64, i64)>, AppError> {
        PgTagRepository::list_with_counts_impl(self, user_id, cursor, limit, scope).await
    }

    async fn list_aliases(&self, tag_id: TagId) -> Result<Vec<TagAlias>, AppError> {
        PgTagRepository::list_aliases_impl(self, tag_id).await
    }

    async fn count_items_for_tag(&self, tag_id: TagId) -> Result<i64, AppError> {
        PgTagRepository::count_items_for_tag_impl(self, tag_id).await
    }

    async fn count_highlights_for_tag(&self, tag_id: TagId) -> Result<i64, AppError> {
        PgTagRepository::count_highlights_for_tag_impl(self, tag_id).await
    }

    async fn list_tag_highlights(
        &self,
        tag_id: TagId,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<TaggedHighlight>, AppError> {
        PgTagRepository::list_tag_highlights_impl(self, tag_id, user_id, cursor, limit).await
    }

    async fn list_aliases_for_tags(
        &self,
        tag_ids: &[TagId],
    ) -> Result<HashMap<TagId, Vec<String>>, AppError> {
        PgTagRepository::list_aliases_for_tags_impl(self, tag_ids).await
    }
}
