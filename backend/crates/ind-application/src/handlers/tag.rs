use std::sync::Arc;

use chrono::Utc;
use futures::future::BoxFuture;
use ind_domain::{
    DocumentId, DomainError, Highlight, LibraryEntryId, LibraryEntryWithDocument, Tag, TagId,
    UserId,
};

use crate::AppError;
use crate::event_intents::library_entry_tagged;
use crate::ports::{CreateTagRequest, TagOperations, UpdateTagRequest};
use crate::repos::event::MutationSideEffects;
use crate::repos::tag::TagRepository;
use crate::repos::{Cursor, Page};

pub struct CreateTagInput {
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<TagId>,
}

pub struct UpdateTagInput {
    pub name: Option<String>,
    pub color: Option<Option<String>>,
    pub parent_id: Option<Option<TagId>>,
}

pub struct TagWithMeta {
    pub tag: Tag,
    pub item_count: i64,
    pub highlight_count: i64,
    pub aliases: Vec<String>,
}

pub struct TagService {
    tag_repo: Arc<dyn TagRepository>,
}

impl TagService {
    pub fn new(tag_repo: Arc<dyn TagRepository>) -> Self {
        Self { tag_repo }
    }

    pub async fn create(
        &self,
        user_id: UserId,
        input: CreateTagInput,
    ) -> Result<TagWithMeta, AppError> {
        let existing = self.tag_repo.find_by_name(user_id, &input.name).await?;
        if existing.is_some() {
            return Err(AppError::Domain(DomainError::Conflict {
                entity: "tag",
                message: format!("tag with name '{}' already exists", input.name),
            }));
        }

        if let Some(pid) = input.parent_id {
            self.tag_repo
                .find_by_id_for_user(pid, user_id)
                .await?
                .ok_or_else(|| {
                    AppError::Domain(DomainError::NotFound {
                        entity: "tag",
                        id: pid.to_string(),
                    })
                })?;
        }

        let tag = Tag {
            id: TagId::new(),
            user_id,
            name: input.name,
            color: input.color,
            parent_id: input.parent_id,
            created_at: Utc::now(),
        };

        let created = self.tag_repo.create(tag).await?;
        Ok(TagWithMeta {
            tag: created,
            item_count: 0,
            highlight_count: 0,
            aliases: vec![],
        })
    }

    pub async fn get(&self, user_id: UserId, id: TagId) -> Result<TagWithMeta, AppError> {
        let tag = self
            .tag_repo
            .find_by_id_for_user(id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "tag",
                    id: id.to_string(),
                })
            })?;

        let aliases = self.tag_repo.list_aliases(id).await?;
        let alias_names: Vec<String> = aliases.into_iter().map(|a| a.alias).collect();
        let item_count = self.tag_repo.count_items_for_tag(id).await?;
        let highlight_count = self.tag_repo.count_highlights_for_tag(id).await?;

        Ok(TagWithMeta {
            tag,
            item_count,
            highlight_count,
            aliases: alias_names,
        })
    }

    pub async fn list(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
        scope: Option<&str>,
    ) -> Result<Page<TagWithMeta>, AppError> {
        let page = self
            .tag_repo
            .list_with_counts(user_id, cursor, limit, scope)
            .await?;

        let tag_ids: Vec<TagId> = page.items.iter().map(|(tag, _, _)| tag.id).collect();
        let mut aliases_map = self.tag_repo.list_aliases_for_tags(&tag_ids).await?;

        let items = page
            .items
            .into_iter()
            .map(|(tag, item_count, highlight_count)| {
                let aliases = aliases_map.remove(&tag.id).unwrap_or_default();
                TagWithMeta {
                    tag,
                    item_count,
                    highlight_count,
                    aliases,
                }
            })
            .collect();

        Ok(Page {
            items,
            next_cursor: page.next_cursor,
        })
    }

    pub async fn update(
        &self,
        user_id: UserId,
        id: TagId,
        input: UpdateTagInput,
    ) -> Result<TagWithMeta, AppError> {
        let mut tag = self
            .tag_repo
            .find_by_id_for_user(id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "tag",
                    id: id.to_string(),
                })
            })?;

        if let Some(ref new_name) = input.name {
            tag = self.tag_repo.update_name(id, user_id, new_name).await?;
        }
        if let Some(ref new_color) = input.color {
            tag = self
                .tag_repo
                .update_color(id, user_id, new_color.as_deref())
                .await?;
        }
        if let Some(new_parent) = input.parent_id {
            if let Some(pid) = new_parent {
                if pid == id {
                    return Err(AppError::Domain(DomainError::Validation {
                        field: "parent_id".into(),
                        message: "a tag cannot be its own parent".into(),
                    }));
                }
                self.tag_repo
                    .find_by_id_for_user(pid, user_id)
                    .await?
                    .ok_or_else(|| {
                        AppError::Domain(DomainError::NotFound {
                            entity: "tag",
                            id: pid.to_string(),
                        })
                    })?;
            }
            tag = self.tag_repo.update_parent(id, user_id, new_parent).await?;
        }

        let aliases = self.tag_repo.list_aliases(id).await?;
        let alias_names: Vec<String> = aliases.into_iter().map(|a| a.alias).collect();
        let item_count = self.tag_repo.count_items_for_tag(id).await?;
        let highlight_count = self.tag_repo.count_highlights_for_tag(id).await?;

        Ok(TagWithMeta {
            tag,
            item_count,
            highlight_count,
            aliases: alias_names,
        })
    }

    pub async fn delete(&self, user_id: UserId, id: TagId) -> Result<(), AppError> {
        self.tag_repo.delete_for_user(id, user_id).await
    }

    pub async fn merge(
        &self,
        user_id: UserId,
        source_ids: Vec<TagId>,
        target_id: TagId,
    ) -> Result<TagWithMeta, AppError> {
        if source_ids.is_empty() {
            return Err(AppError::Domain(DomainError::Validation {
                field: "source_ids".into(),
                message: "must provide at least one source tag".into(),
            }));
        }
        if source_ids.contains(&target_id) {
            return Err(AppError::Domain(DomainError::Validation {
                field: "target_id".into(),
                message: "target tag must not be in source list".into(),
            }));
        }

        let tag = self
            .tag_repo
            .merge_tags(&source_ids, target_id, user_id)
            .await?;

        let aliases = self.tag_repo.list_aliases(target_id).await?;
        let alias_names: Vec<String> = aliases.into_iter().map(|a| a.alias).collect();
        let item_count = self.tag_repo.count_items_for_tag(target_id).await?;
        let highlight_count = self.tag_repo.count_highlights_for_tag(target_id).await?;

        Ok(TagWithMeta {
            tag,
            item_count,
            highlight_count,
            aliases: alias_names,
        })
    }

    pub async fn list_highlights(
        &self,
        user_id: UserId,
        tag_id: TagId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<Highlight>, AppError> {
        self.tag_repo
            .find_by_id_for_user(tag_id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "tag",
                    id: tag_id.to_string(),
                })
            })?;

        self.tag_repo
            .list_tag_highlights(tag_id, user_id, cursor, limit)
            .await
    }

    pub async fn set_library_entry_tags(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
        document_id: DocumentId,
        tag_names: Vec<String>,
    ) -> Result<Vec<Tag>, AppError> {
        let mut tag_ids = Vec::with_capacity(tag_names.len());
        let mut resolved_tags = Vec::with_capacity(tag_names.len());
        for name in &tag_names {
            let tag = self.tag_repo.find_or_create_by_name(user_id, name).await?;
            tag_ids.push(tag.id);
            resolved_tags.push(tag);
        }

        let effects = MutationSideEffects::with_events(vec![library_entry_tagged(
            user_id,
            library_entry_id,
            document_id,
            &resolved_tags,
        )]);

        self.tag_repo
            .replace_for_library_entry(user_id, library_entry_id, &tag_ids, effects)
            .await?;

        self.tag_repo
            .list_by_library_entry(user_id, library_entry_id)
            .await
    }

    pub async fn list_library_entry_tags(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
    ) -> Result<Vec<Tag>, AppError> {
        self.tag_repo
            .list_by_library_entry(user_id, library_entry_id)
            .await
    }

    pub async fn list_library_entries(
        &self,
        user_id: UserId,
        tag_id: TagId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        self.tag_repo
            .find_by_id_for_user(tag_id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "tag",
                    id: tag_id.to_string(),
                })
            })?;

        self.tag_repo
            .list_tag_library_entries(tag_id, user_id, cursor, limit)
            .await
    }
}

impl TagOperations for TagService {
    fn create_tag(
        &self,
        user_id: UserId,
        request: CreateTagRequest,
    ) -> BoxFuture<'_, Result<TagWithMeta, AppError>> {
        Box::pin(self.create(
            user_id,
            CreateTagInput {
                name: request.name,
                color: request.color,
                parent_id: request.parent_id,
            },
        ))
    }

    fn get_tag(&self, user_id: UserId, id: TagId) -> BoxFuture<'_, Result<TagWithMeta, AppError>> {
        Box::pin(self.get(user_id, id))
    }

    fn list_tags(
        &self,
        user_id: UserId,
        cursor: Option<String>,
        limit: Option<u32>,
        scope: Option<String>,
    ) -> BoxFuture<'_, Result<Page<TagWithMeta>, AppError>> {
        Box::pin(async move {
            self.list(
                user_id,
                cursor.map(Cursor),
                limit.unwrap_or(50),
                scope.as_deref(),
            )
            .await
        })
    }

    fn update_tag(
        &self,
        user_id: UserId,
        id: TagId,
        request: UpdateTagRequest,
    ) -> BoxFuture<'_, Result<TagWithMeta, AppError>> {
        Box::pin(self.update(
            user_id,
            id,
            UpdateTagInput {
                name: request.name,
                color: request.color,
                parent_id: request.parent_id,
            },
        ))
    }

    fn delete_tag(&self, user_id: UserId, id: TagId) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.delete(user_id, id))
    }

    fn merge_tags(
        &self,
        user_id: UserId,
        source_ids: Vec<TagId>,
        target_id: TagId,
    ) -> BoxFuture<'_, Result<TagWithMeta, AppError>> {
        Box::pin(self.merge(user_id, source_ids, target_id))
    }

    fn list_tag_highlights(
        &self,
        user_id: UserId,
        tag_id: TagId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<Highlight>, AppError>> {
        Box::pin(self.list_highlights(user_id, tag_id, cursor.map(Cursor), limit.unwrap_or(50)))
    }

    fn set_library_entry_tags(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
        document_id: DocumentId,
        tag_names: Vec<String>,
    ) -> BoxFuture<'_, Result<Vec<Tag>, AppError>> {
        Box::pin(self.set_library_entry_tags(user_id, library_entry_id, document_id, tag_names))
    }

    fn list_library_entry_tags(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<Vec<Tag>, AppError>> {
        Box::pin(self.list_library_entry_tags(user_id, library_entry_id))
    }

    fn list_tag_library_entries(
        &self,
        user_id: UserId,
        tag_id: TagId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<LibraryEntryWithDocument>, AppError>> {
        Box::pin(self.list_library_entries(
            user_id,
            tag_id,
            cursor.map(Cursor),
            limit.unwrap_or(50),
        ))
    }
}
