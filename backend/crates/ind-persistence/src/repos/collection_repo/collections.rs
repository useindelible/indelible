use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::{Cursor, Page};
use ind_domain::{Collection, CollectionId, DomainError, UserId};

use crate::cursor::{clamp_limit, decode_cursor_collection, encode_cursor_collection};

use super::rows::CollectionRow;
use super::{PgCollectionRepository, map_sqlx_error};

pub(super) struct CollectionFieldUpdate<'a> {
    pub(super) user_id: UserId,
    pub(super) id: CollectionId,
    pub(super) name: Option<&'a str>,
    pub(super) description: Option<Option<&'a str>>,
    pub(super) icon: Option<Option<&'a str>>,
    pub(super) color: Option<Option<&'a str>>,
    pub(super) sort_order: Option<i32>,
    pub(super) parent_id: Option<Option<CollectionId>>,
}

impl PgCollectionRepository {
    pub(super) async fn find_by_id_query(
        &self,
        user_id: UserId,
        id: CollectionId,
    ) -> Result<Option<Collection>, AppError> {
        let row = sqlx::query_as!(
            CollectionRow,
            "SELECT id, user_id, parent_id, name, description, icon, color, \
             sort_order, is_pinned, rss_token, created_at, updated_at \
             FROM collections WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Collection::from))
    }

    pub(super) async fn find_by_parent_and_name_query(
        &self,
        user_id: UserId,
        parent_id: Option<CollectionId>,
        name: &str,
    ) -> Result<Option<Collection>, AppError> {
        let row = sqlx::query_as!(
            CollectionRow,
            "SELECT id, user_id, parent_id, name, description, icon, color, \
             sort_order, is_pinned, rss_token, created_at, updated_at \
             FROM collections \
             WHERE user_id = $1 \
               AND (($2::uuid IS NULL AND parent_id IS NULL) OR parent_id = $2) \
               AND lower(name) = lower($3) \
             ORDER BY created_at ASC \
             LIMIT 1",
            user_id.into_uuid(),
            parent_id.map(|id| id.into_uuid()) as Option<Uuid>,
            name,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Collection::from))
    }

    pub(super) async fn create_collection(
        &self,
        collection: Collection,
    ) -> Result<Collection, AppError> {
        self.ensure_parent_exists(collection.user_id, collection.parent_id)
            .await?;

        let row = sqlx::query_as!(
            CollectionRow,
            "INSERT INTO collections (id, user_id, parent_id, name, description, icon, color, \
             sort_order, is_pinned, rss_token, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) \
             RETURNING id, user_id, parent_id, name, description, icon, color, \
             sort_order, is_pinned, rss_token, created_at, updated_at",
            collection.id.into_uuid(),
            collection.user_id.into_uuid(),
            collection.parent_id.map(|id| id.into_uuid()) as Option<Uuid>,
            &collection.name,
            collection.description.as_deref(),
            collection.icon.as_deref(),
            collection.color.as_deref(),
            collection.sort_order,
            collection.is_pinned,
            collection.rss_token.as_deref(),
            collection.created_at,
            collection.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(Collection::from(row))
    }

    pub(super) async fn delete_collection(
        &self,
        user_id: UserId,
        id: CollectionId,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            "DELETE FROM collections WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "collection",
                id: id.to_string(),
            }));
        }

        Ok(())
    }

    pub(super) async fn list_by_user_query(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<Collection>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = cursor {
            let (cursor_sort, cursor_name, cursor_id) = decode_cursor_collection(cursor)?;
            sqlx::query_as!(
                CollectionRow,
                "SELECT id, user_id, parent_id, name, description, icon, color, \
                 sort_order, is_pinned, rss_token, created_at, updated_at \
                 FROM collections \
                 WHERE user_id = $1 \
                 AND (sort_order, name, id) > ($2, $3, $4) \
                 ORDER BY sort_order ASC, name ASC, id ASC \
                 LIMIT $5",
                user_id.into_uuid(),
                cursor_sort,
                &cursor_name,
                cursor_id,
                fetch_limit
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        } else {
            sqlx::query_as!(
                CollectionRow,
                "SELECT id, user_id, parent_id, name, description, icon, color, \
                 sort_order, is_pinned, rss_token, created_at, updated_at \
                 FROM collections \
                 WHERE user_id = $1 \
                 ORDER BY sort_order ASC, name ASC, id ASC \
                 LIMIT $2",
                user_id.into_uuid(),
                fetch_limit
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        };

        Ok(collection_page(rows, limit))
    }

    pub(super) async fn list_children_query(
        &self,
        user_id: UserId,
        parent_id: CollectionId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<Collection>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = cursor {
            let (cursor_sort, cursor_name, cursor_id) = decode_cursor_collection(cursor)?;
            sqlx::query_as!(
                CollectionRow,
                "SELECT id, user_id, parent_id, name, description, icon, color, \
                 sort_order, is_pinned, rss_token, created_at, updated_at \
                 FROM collections \
                 WHERE user_id = $1 AND parent_id = $2 \
                 AND (sort_order, name, id) > ($3, $4, $5) \
                 ORDER BY sort_order ASC, name ASC, id ASC \
                 LIMIT $6",
                user_id.into_uuid(),
                parent_id.into_uuid(),
                cursor_sort,
                &cursor_name,
                cursor_id,
                fetch_limit
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        } else {
            sqlx::query_as!(
                CollectionRow,
                "SELECT id, user_id, parent_id, name, description, icon, color, \
                 sort_order, is_pinned, rss_token, created_at, updated_at \
                 FROM collections \
                 WHERE user_id = $1 AND parent_id = $2 \
                 ORDER BY sort_order ASC, name ASC, id ASC \
                 LIMIT $3",
                user_id.into_uuid(),
                parent_id.into_uuid(),
                fetch_limit
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        };

        Ok(collection_page(rows, limit))
    }

    pub(super) async fn update_collection_fields(
        &self,
        update: CollectionFieldUpdate<'_>,
    ) -> Result<Collection, AppError> {
        if let Some(Some(pid)) = update.parent_id {
            self.validate_parent_move(update.user_id, update.id, pid)
                .await?;
        }

        let row = sqlx::query_as!(
            CollectionRow,
            "UPDATE collections SET \
             name = COALESCE($3, name), \
             description = CASE WHEN $4 THEN $5 ELSE description END, \
             icon = CASE WHEN $6 THEN $7 ELSE icon END, \
             color = CASE WHEN $8 THEN $9 ELSE color END, \
             sort_order = COALESCE($10, sort_order), \
             parent_id = CASE WHEN $11 THEN $12 ELSE parent_id END, \
             updated_at = now() \
             WHERE id = $1 AND user_id = $2 \
             RETURNING id, user_id, parent_id, name, description, icon, color, \
             sort_order, is_pinned, rss_token, created_at, updated_at",
            update.id.into_uuid(),
            update.user_id.into_uuid(),
            update.name,
            update.description.is_some(),
            update.description.flatten(),
            update.icon.is_some(),
            update.icon.flatten(),
            update.color.is_some(),
            update.color.flatten(),
            update.sort_order,
            update.parent_id.is_some(),
            update.parent_id.flatten().map(|p| p.into_uuid()),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "collection",
                id: update.id.to_string(),
            })
        })?;

        Ok(Collection::from(row))
    }

    async fn ensure_parent_exists(
        &self,
        user_id: UserId,
        parent_id: Option<CollectionId>,
    ) -> Result<(), AppError> {
        let Some(parent_id) = parent_id else {
            return Ok(());
        };

        let parent_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM collections WHERE id = $1 AND user_id = $2)",
            parent_id.into_uuid(),
            user_id.into_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if !parent_exists.unwrap_or(false) {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "collection",
                id: parent_id.to_string(),
            }));
        }

        Ok(())
    }

    async fn validate_parent_move(
        &self,
        user_id: UserId,
        id: CollectionId,
        parent_id: CollectionId,
    ) -> Result<(), AppError> {
        if parent_id == id {
            return Err(AppError::Domain(DomainError::Validation {
                field: "parent_id".into(),
                message: "a collection cannot be its own parent".into(),
            }));
        }

        self.ensure_parent_exists(user_id, Some(parent_id)).await?;

        let creates_cycle = sqlx::query_scalar!(
            "WITH RECURSIVE descendants AS ( \
                 SELECT id FROM collections WHERE id = $1 AND user_id = $2 \
                 UNION ALL \
                 SELECT c.id FROM collections c \
                 JOIN descendants d ON c.parent_id = d.id \
                 WHERE c.user_id = $2 \
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
                message: "a collection cannot be moved under one of its descendants".into(),
            }));
        }

        Ok(())
    }
}

fn collection_page(rows: Vec<CollectionRow>, limit: i64) -> Page<Collection> {
    let has_more = rows.len() as i64 > limit;
    let take = if has_more { limit as usize } else { rows.len() };
    let items: Vec<Collection> = rows.into_iter().take(take).map(Collection::from).collect();
    let next_cursor = if has_more {
        items
            .last()
            .map(|c| encode_cursor_collection(c.sort_order, &c.name, c.id.into_uuid()))
    } else {
        None
    };

    Page { items, next_cursor }
}
