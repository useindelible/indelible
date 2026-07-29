use ind_application::AppError;
use ind_application::repos::{Cursor, Page};
use ind_domain::{Collection, CollectionId, UserId};

use crate::cursor::{clamp_limit, decode_cursor_collection, encode_cursor_collection};

use super::rows::CollectionWithCount;
use super::{PgCollectionRepository, map_sqlx_error};

impl PgCollectionRepository {
    pub(super) async fn list_by_user_with_counts_query(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<(Collection, i64)>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = cursor {
            let (cursor_sort, cursor_name, cursor_id) = decode_cursor_collection(cursor)?;
            sqlx::query_as!(
                CollectionWithCount,
                r#"WITH RECURSIVE subtree AS (
                       SELECT id AS root_id, id AS node_id
                       FROM collections
                       WHERE user_id = $1
                   UNION ALL
                       SELECT st.root_id, c.id
                       FROM collections c
                       JOIN subtree st ON c.parent_id = st.node_id
                       WHERE c.user_id = $1
                   ),
                   recursive_counts AS (
                       SELECT st.root_id, COUNT(le.id) AS total
                       FROM subtree st
                       LEFT JOIN collection_entries ce ON ce.collection_id = st.node_id
                       LEFT JOIN library_entries le
                           ON le.id = ce.library_entry_id AND le.deleted_at IS NULL
                       GROUP BY st.root_id
                   )
                   SELECT c.id, c.user_id, c.parent_id, c.name, c.description, c.icon, c.color,
                          c.sort_order, c.is_pinned, c.rss_token, c.created_at, c.updated_at,
                          COALESCE(rc.total, 0) AS item_count
                   FROM collections c
                   LEFT JOIN recursive_counts rc ON rc.root_id = c.id
                   WHERE c.user_id = $1
                   AND (c.sort_order, c.name, c.id) > ($2, $3, $4)
                   ORDER BY c.sort_order ASC, c.name ASC, c.id ASC
                   LIMIT $5"#,
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
                CollectionWithCount,
                r#"WITH RECURSIVE subtree AS (
                       SELECT id AS root_id, id AS node_id
                       FROM collections
                       WHERE user_id = $1
                   UNION ALL
                       SELECT st.root_id, c.id
                       FROM collections c
                       JOIN subtree st ON c.parent_id = st.node_id
                       WHERE c.user_id = $1
                   ),
                   recursive_counts AS (
                       SELECT st.root_id, COUNT(le.id) AS total
                       FROM subtree st
                       LEFT JOIN collection_entries ce ON ce.collection_id = st.node_id
                       LEFT JOIN library_entries le
                           ON le.id = ce.library_entry_id AND le.deleted_at IS NULL
                       GROUP BY st.root_id
                   )
                   SELECT c.id, c.user_id, c.parent_id, c.name, c.description, c.icon, c.color,
                          c.sort_order, c.is_pinned, c.rss_token, c.created_at, c.updated_at,
                          COALESCE(rc.total, 0) AS item_count
                   FROM collections c
                   LEFT JOIN recursive_counts rc ON rc.root_id = c.id
                   WHERE c.user_id = $1
                   ORDER BY c.sort_order ASC, c.name ASC, c.id ASC
                   LIMIT $2"#,
                user_id.into_uuid(),
                fetch_limit
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        };

        Ok(collection_with_counts_page(rows, limit))
    }

    pub(super) async fn list_children_with_counts_query(
        &self,
        user_id: UserId,
        parent_id: CollectionId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<(Collection, i64)>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = cursor {
            let (cursor_sort, cursor_name, cursor_id) = decode_cursor_collection(cursor)?;
            sqlx::query_as!(
                CollectionWithCount,
                r#"WITH RECURSIVE subtree AS (
                       SELECT id AS root_id, id AS node_id
                       FROM collections
                       WHERE user_id = $1 AND parent_id = $2
                   UNION ALL
                       SELECT st.root_id, c.id
                       FROM collections c
                       JOIN subtree st ON c.parent_id = st.node_id
                       WHERE c.user_id = $1
                   ),
                   recursive_counts AS (
                       SELECT st.root_id, COUNT(le.id) AS total
                       FROM subtree st
                       LEFT JOIN collection_entries ce ON ce.collection_id = st.node_id
                       LEFT JOIN library_entries le
                           ON le.id = ce.library_entry_id AND le.deleted_at IS NULL
                       GROUP BY st.root_id
                   )
                   SELECT c.id, c.user_id, c.parent_id, c.name, c.description, c.icon, c.color,
                          c.sort_order, c.is_pinned, c.rss_token, c.created_at, c.updated_at,
                          COALESCE(rc.total, 0) AS item_count
                   FROM collections c
                   LEFT JOIN recursive_counts rc ON rc.root_id = c.id
                   WHERE c.user_id = $1 AND c.parent_id = $2
                   AND (c.sort_order, c.name, c.id) > ($3, $4, $5)
                   ORDER BY c.sort_order ASC, c.name ASC, c.id ASC
                   LIMIT $6"#,
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
                CollectionWithCount,
                r#"WITH RECURSIVE subtree AS (
                       SELECT id AS root_id, id AS node_id
                       FROM collections
                       WHERE user_id = $1 AND parent_id = $2
                   UNION ALL
                       SELECT st.root_id, c.id
                       FROM collections c
                       JOIN subtree st ON c.parent_id = st.node_id
                       WHERE c.user_id = $1
                   ),
                   recursive_counts AS (
                       SELECT st.root_id, COUNT(le.id) AS total
                       FROM subtree st
                       LEFT JOIN collection_entries ce ON ce.collection_id = st.node_id
                       LEFT JOIN library_entries le
                           ON le.id = ce.library_entry_id AND le.deleted_at IS NULL
                       GROUP BY st.root_id
                   )
                   SELECT c.id, c.user_id, c.parent_id, c.name, c.description, c.icon, c.color,
                          c.sort_order, c.is_pinned, c.rss_token, c.created_at, c.updated_at,
                          COALESCE(rc.total, 0) AS item_count
                   FROM collections c
                   LEFT JOIN recursive_counts rc ON rc.root_id = c.id
                   WHERE c.user_id = $1 AND c.parent_id = $2
                   ORDER BY c.sort_order ASC, c.name ASC, c.id ASC
                   LIMIT $3"#,
                user_id.into_uuid(),
                parent_id.into_uuid(),
                fetch_limit
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        };

        Ok(collection_with_counts_page(rows, limit))
    }
}

fn collection_with_counts_page(
    rows: Vec<CollectionWithCount>,
    limit: i64,
) -> Page<(Collection, i64)> {
    let has_more = rows.len() as i64 > limit;
    let take = if has_more { limit as usize } else { rows.len() };
    let items: Vec<(Collection, i64)> = rows
        .into_iter()
        .take(take)
        .map(CollectionWithCount::into_pair)
        .collect();
    let next_cursor = if has_more {
        items
            .last()
            .map(|(c, _)| encode_cursor_collection(c.sort_order, &c.name, c.id.into_uuid()))
    } else {
        None
    };

    Page { items, next_cursor }
}
