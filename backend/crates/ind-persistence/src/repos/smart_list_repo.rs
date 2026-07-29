use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::smart_list::SmartListRepository;
use ind_application::repos::{Cursor, Page};
use ind_domain::{
    DomainError, FilterNode, LibraryEntryWithDocument, SmartList, SmartListId, UserId,
};

use super::library_query::{LibraryListFilter, query_library_entries_page};
use crate::cursor::{clamp_limit, decode_cursor_name, encode_cursor_name};

pub struct PgSmartListRepository {
    pool: PgPool,
}

impl PgSmartListRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SmartListRow {
    id: Uuid,
    user_id: Uuid,
    name: String,
    icon: Option<String>,
    color: Option<String>,
    is_pinned: bool,
    filter_expression: Json<FilterNode>,
    default_sort: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SmartListRow> for SmartList {
    fn from(row: SmartListRow) -> Self {
        SmartList {
            id: SmartListId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            name: row.name,
            icon: row.icon,
            color: row.color,
            is_pinned: row.is_pinned,
            filter_expression: row.filter_expression.0,
            default_sort: row.default_sort,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("smart_list", "duplicate smart list", err)
}

// -- SmartListRepository implementation --

#[async_trait::async_trait]
impl SmartListRepository for PgSmartListRepository {
    async fn find_by_id(
        &self,
        id: SmartListId,
        user_id: UserId,
    ) -> Result<Option<SmartList>, AppError> {
        let row = sqlx::query_as!(
            SmartListRow,
            r#"SELECT id, user_id, name, icon, color, is_pinned,
             filter_expression as "filter_expression: Json<FilterNode>",
             default_sort, created_at, updated_at
             FROM smart_lists WHERE id = $1 AND user_id = $2"#,
            id.into_uuid(),
            user_id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(SmartList::from))
    }

    async fn create(&self, smart_list: SmartList) -> Result<SmartList, AppError> {
        let filter_expression = Json(smart_list.filter_expression.clone());
        let row = sqlx::query_as!(
            SmartListRow,
            r#"INSERT INTO smart_lists (id, user_id, name, icon, color, is_pinned,
             filter_expression, default_sort, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id, user_id, name, icon, color, is_pinned,
             filter_expression as "filter_expression: Json<FilterNode>",
             default_sort, created_at, updated_at"#,
            smart_list.id.into_uuid(),
            smart_list.user_id.into_uuid(),
            &smart_list.name,
            smart_list.icon.as_deref(),
            smart_list.color.as_deref(),
            smart_list.is_pinned,
            filter_expression as _,
            smart_list.default_sort.as_deref(),
            smart_list.created_at,
            smart_list.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(SmartList::from(row))
    }

    async fn update_name(
        &self,
        id: SmartListId,
        user_id: UserId,
        name: &str,
    ) -> Result<SmartList, AppError> {
        let row = sqlx::query_as!(
            SmartListRow,
            r#"UPDATE smart_lists SET name = $3, updated_at = now()
             WHERE id = $1 AND user_id = $2
             RETURNING id, user_id, name, icon, color, is_pinned,
             filter_expression as "filter_expression: Json<FilterNode>",
             default_sort, created_at, updated_at"#,
            id.into_uuid(),
            user_id.into_uuid(),
            name,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "smart_list",
                id: id.to_string(),
            })
        })?;

        Ok(SmartList::from(row))
    }

    async fn update_filter(
        &self,
        id: SmartListId,
        user_id: UserId,
        filter: &FilterNode,
    ) -> Result<SmartList, AppError> {
        let filter_expression = Json(filter.clone());
        let row = sqlx::query_as!(
            SmartListRow,
            r#"UPDATE smart_lists SET filter_expression = $3, updated_at = now()
             WHERE id = $1 AND user_id = $2
             RETURNING id, user_id, name, icon, color, is_pinned,
             filter_expression as "filter_expression: Json<FilterNode>",
             default_sort, created_at, updated_at"#,
            id.into_uuid(),
            user_id.into_uuid(),
            filter_expression as _,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "smart_list",
                id: id.to_string(),
            })
        })?;

        Ok(SmartList::from(row))
    }

    async fn update_pin(
        &self,
        id: SmartListId,
        user_id: UserId,
        is_pinned: bool,
    ) -> Result<SmartList, AppError> {
        let row = sqlx::query_as!(
            SmartListRow,
            r#"UPDATE smart_lists SET is_pinned = $3, updated_at = now()
             WHERE id = $1 AND user_id = $2
             RETURNING id, user_id, name, icon, color, is_pinned,
             filter_expression as "filter_expression: Json<FilterNode>",
             default_sort, created_at, updated_at"#,
            id.into_uuid(),
            user_id.into_uuid(),
            is_pinned,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "smart_list",
                id: id.to_string(),
            })
        })?;

        Ok(SmartList::from(row))
    }

    async fn update_metadata(
        &self,
        id: SmartListId,
        user_id: UserId,
        icon: Option<&str>,
        color: Option<&str>,
        default_sort: Option<&str>,
    ) -> Result<SmartList, AppError> {
        let row = sqlx::query_as!(
            SmartListRow,
            r#"UPDATE smart_lists SET icon = $3, color = $4, default_sort = $5, updated_at = now()
             WHERE id = $1 AND user_id = $2
             RETURNING id, user_id, name, icon, color, is_pinned,
             filter_expression as "filter_expression: Json<FilterNode>",
             default_sort, created_at, updated_at"#,
            id.into_uuid(),
            user_id.into_uuid(),
            icon,
            color,
            default_sort,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "smart_list",
                id: id.to_string(),
            })
        })?;

        Ok(SmartList::from(row))
    }

    async fn delete(&self, id: SmartListId, user_id: UserId) -> Result<(), AppError> {
        let result = sqlx::query!(
            "DELETE FROM smart_lists WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "smart_list",
                id: id.to_string(),
            }));
        }

        Ok(())
    }

    async fn list_by_user(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<SmartList>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = cursor {
            let (cursor_name, cursor_id) = decode_cursor_name(cursor)?;
            sqlx::query_as!(
                SmartListRow,
                r#"SELECT id, user_id, name, icon, color, is_pinned,
                 filter_expression as "filter_expression: Json<FilterNode>",
                 default_sort, created_at, updated_at
                 FROM smart_lists
                 WHERE user_id = $1 AND (name, id) > ($2, $3)
                 ORDER BY name ASC, id ASC
                 LIMIT $4"#,
                user_id.into_uuid(),
                &cursor_name,
                cursor_id,
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        } else {
            sqlx::query_as!(
                SmartListRow,
                r#"SELECT id, user_id, name, icon, color, is_pinned,
                 filter_expression as "filter_expression: Json<FilterNode>",
                 default_sort, created_at, updated_at
                 FROM smart_lists
                 WHERE user_id = $1
                 ORDER BY name ASC, id ASC
                 LIMIT $2"#,
                user_id.into_uuid(),
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        };

        let has_more = rows.len() as i64 > limit;
        let take = if has_more { limit as usize } else { rows.len() };

        let items: Vec<SmartList> = rows.into_iter().take(take).map(SmartList::from).collect();

        let next_cursor = if has_more {
            items
                .last()
                .map(|s| encode_cursor_name(&s.name, s.id.into_uuid()))
        } else {
            None
        };

        Ok(Page { items, next_cursor })
    }

    async fn list_pinned(&self, user_id: UserId) -> Result<Vec<SmartList>, AppError> {
        let rows = sqlx::query_as!(
            SmartListRow,
            r#"SELECT id, user_id, name, icon, color, is_pinned,
             filter_expression as "filter_expression: Json<FilterNode>",
             default_sort, created_at, updated_at
             FROM smart_lists
             WHERE user_id = $1 AND is_pinned = true
             ORDER BY name ASC"#,
            user_id.into_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(SmartList::from).collect())
    }

    async fn evaluate_filter(
        &self,
        user_id: UserId,
        filter: &FilterNode,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        // Smart lists describe the Library, so evaluation runs over library_entries JOIN documents
        // (AC#4); prepared-but-unsaved feed documents have no library entry and are excluded (AC#5).
        query_library_entries_page(
            &self.pool,
            user_id,
            &LibraryListFilter {
                filter_expression: Some(filter.clone()),
                trashed_only: false,
            },
            cursor,
            limit.min(200),
        )
        .await
    }
}
