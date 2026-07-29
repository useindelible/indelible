use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::{Cursor, Page};
use ind_domain::{DomainError, Tag, TagAlias, TagAliasId, TagId, UserId};

use crate::cursor::{clamp_limit, decode_cursor_name, encode_cursor_name};

use super::rows::TagRow;
use super::{PgTagRepository, map_sqlx_error};

impl PgTagRepository {
    pub(super) async fn find_by_id_impl(&self, id: TagId) -> Result<Option<Tag>, AppError> {
        let row = sqlx::query_as!(
            TagRow,
            "SELECT id, user_id, name, color, parent_id, created_at \
             FROM tags WHERE id = $1",
            id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Tag::from))
    }

    pub(super) async fn create_impl(&self, tag: Tag) -> Result<Tag, AppError> {
        let row = sqlx::query_as!(
            TagRow,
            "INSERT INTO tags (id, user_id, name, color, parent_id, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, user_id, name, color, parent_id, created_at",
            tag.id.into_uuid(),
            tag.user_id.into_uuid(),
            &tag.name,
            tag.color.as_deref(),
            tag.parent_id.map(|p| p.into_uuid()),
            tag.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(Tag::from(row))
    }

    pub(super) async fn delete_impl(&self, id: TagId) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM tags WHERE id = $1", id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "tag",
                id: id.to_string(),
            }));
        }

        Ok(())
    }

    pub(super) async fn list_by_user_impl(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<Tag>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = cursor {
            let (cursor_name, cursor_id) = decode_cursor_name(cursor)?;
            sqlx::query_as!(
                TagRow,
                "SELECT id, user_id, name, color, parent_id, created_at \
                 FROM tags \
                 WHERE user_id = $1 \
                 AND (name, id) > ($2, $3) \
                 ORDER BY name ASC, id ASC \
                 LIMIT $4",
                user_id.into_uuid(),
                cursor_name,
                cursor_id,
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        } else {
            sqlx::query_as!(
                TagRow,
                "SELECT id, user_id, name, color, parent_id, created_at \
                 FROM tags \
                 WHERE user_id = $1 \
                 ORDER BY name ASC, id ASC \
                 LIMIT $2",
                user_id.into_uuid(),
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        };

        let has_more = rows.len() as i64 > limit;
        let take = if has_more { limit as usize } else { rows.len() };

        let tags: Vec<Tag> = rows.into_iter().take(take).map(Tag::from).collect();

        let next_cursor = if has_more {
            tags.last()
                .map(|t| encode_cursor_name(&t.name, t.id.into_uuid()))
        } else {
            None
        };

        Ok(Page {
            items: tags,
            next_cursor,
        })
    }

    pub(super) async fn find_by_name_impl(
        &self,
        user_id: UserId,
        name: &str,
    ) -> Result<Option<Tag>, AppError> {
        let row = sqlx::query_as!(
            TagRow,
            "SELECT id, user_id, name, color, parent_id, created_at \
             FROM tags \
             WHERE user_id = $1 AND lower(name) = lower($2)",
            user_id.into_uuid(),
            name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Tag::from))
    }

    pub(super) async fn find_or_create_by_name_impl(
        &self,
        user_id: UserId,
        name: &str,
    ) -> Result<Tag, AppError> {
        if let Some(existing) = self.find_by_name_impl(user_id, name).await? {
            return Ok(existing);
        }

        let tag = Tag {
            id: TagId::new(),
            user_id,
            name: name.to_owned(),
            color: None,
            parent_id: None,
            created_at: Utc::now(),
        };

        // ON CONFLICT uses the functional index uq_tags_user_lower_name to handle races
        // between the find_by_name check above and this insert.
        let row = sqlx::query_as!(
            TagRow,
            "INSERT INTO tags (id, user_id, name, color, parent_id, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (user_id, lower(name)) DO UPDATE SET name = tags.name \
             RETURNING id, user_id, name, color, parent_id, created_at",
            tag.id.into_uuid(),
            tag.user_id.into_uuid(),
            &tag.name,
            tag.color.as_deref(),
            tag.parent_id.map(|p| p.into_uuid()),
            tag.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(Tag::from(row))
    }

    pub(super) async fn find_by_id_for_user_impl(
        &self,
        id: TagId,
        user_id: UserId,
    ) -> Result<Option<Tag>, AppError> {
        let row = sqlx::query_as!(
            TagRow,
            "SELECT id, user_id, name, color, parent_id, created_at \
             FROM tags WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Tag::from))
    }

    pub(super) async fn delete_for_user_impl(
        &self,
        id: TagId,
        user_id: UserId,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            "DELETE FROM tags WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "tag",
                id: id.to_string(),
            }));
        }

        Ok(())
    }

    pub(super) async fn update_name_impl(
        &self,
        id: TagId,
        user_id: UserId,
        name: &str,
    ) -> Result<Tag, AppError> {
        let row = sqlx::query_as!(
            TagRow,
            "UPDATE tags SET name = $3 \
             WHERE id = $1 AND user_id = $2 \
             RETURNING id, user_id, name, color, parent_id, created_at",
            id.into_uuid(),
            user_id.into_uuid(),
            name,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "tag",
                id: id.to_string(),
            })
        })?;

        Ok(Tag::from(row))
    }

    pub(super) async fn update_color_impl(
        &self,
        id: TagId,
        user_id: UserId,
        color: Option<&str>,
    ) -> Result<Tag, AppError> {
        let row = sqlx::query_as!(
            TagRow,
            "UPDATE tags SET color = $3 \
             WHERE id = $1 AND user_id = $2 \
             RETURNING id, user_id, name, color, parent_id, created_at",
            id.into_uuid(),
            user_id.into_uuid(),
            color,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "tag",
                id: id.to_string(),
            })
        })?;

        Ok(Tag::from(row))
    }

    pub(super) async fn update_parent_impl(
        &self,
        id: TagId,
        user_id: UserId,
        parent_id: Option<TagId>,
    ) -> Result<Tag, AppError> {
        if let Some(parent_id) = parent_id {
            let creates_cycle = sqlx::query_scalar!(
                "WITH RECURSIVE descendants AS ( \
                     SELECT id FROM tags WHERE id = $1 AND user_id = $2 \
                     UNION ALL \
                     SELECT t.id FROM tags t \
                     JOIN descendants d ON t.parent_id = d.id \
                     WHERE t.user_id = $2 \
                 ) \
                 SELECT EXISTS(SELECT 1 FROM descendants WHERE id = $3)",
                id.into_uuid(),
                user_id.into_uuid(),
                parent_id.into_uuid(),
            )
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

            if creates_cycle.unwrap_or(false) {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "parent_id".into(),
                    message: "a tag cannot be moved under one of its descendants".into(),
                }));
            }
        }

        let row = sqlx::query_as!(
            TagRow,
            "UPDATE tags SET parent_id = $3 \
             WHERE id = $1 AND user_id = $2 \
             RETURNING id, user_id, name, color, parent_id, created_at",
            id.into_uuid(),
            user_id.into_uuid(),
            parent_id.map(|p| p.into_uuid()),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "tag",
                id: id.to_string(),
            })
        })?;

        Ok(Tag::from(row))
    }

    pub(super) async fn list_aliases_impl(&self, tag_id: TagId) -> Result<Vec<TagAlias>, AppError> {
        #[derive(sqlx::FromRow)]
        struct AliasRow {
            id: Uuid,
            tag_id: Uuid,
            alias: String,
        }

        let rows = sqlx::query_as!(
            AliasRow,
            "SELECT id, tag_id, alias FROM tag_aliases WHERE tag_id = $1 ORDER BY alias ASC",
            tag_id.into_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|r| TagAlias {
                id: TagAliasId::from_uuid(r.id),
                tag_id: TagId::from_uuid(r.tag_id),
                alias: r.alias,
            })
            .collect())
    }

    pub(super) async fn count_items_for_tag_impl(&self, tag_id: TagId) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM library_entry_tags let \
             JOIN library_entries le ON le.id = let.library_entry_id AND le.deleted_at IS NULL \
             WHERE let.tag_id = $1",
            tag_id.into_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(count.unwrap_or(0))
    }

    pub(super) async fn count_highlights_for_tag_impl(
        &self,
        tag_id: TagId,
    ) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM highlight_tags WHERE tag_id = $1",
            tag_id.into_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(count.unwrap_or(0))
    }

    pub(super) async fn list_aliases_for_tags_impl(
        &self,
        tag_ids: &[TagId],
    ) -> Result<HashMap<TagId, Vec<String>>, AppError> {
        let uuids: Vec<Uuid> = tag_ids.iter().map(|id| id.into_uuid()).collect();

        #[derive(sqlx::FromRow)]
        struct AliasEntry {
            tag_id: Uuid,
            alias: String,
        }

        let rows = sqlx::query_as!(
            AliasEntry,
            "SELECT tag_id, alias FROM tag_aliases WHERE tag_id = ANY($1) ORDER BY alias ASC",
            &uuids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut map: HashMap<TagId, Vec<String>> = HashMap::new();
        for row in rows {
            map.entry(TagId::from_uuid(row.tag_id))
                .or_default()
                .push(row.alias);
        }

        Ok(map)
    }
}
