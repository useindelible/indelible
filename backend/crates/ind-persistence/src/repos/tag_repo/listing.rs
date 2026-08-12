use chrono::{DateTime, Utc};
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::tag::TaggedHighlight;
use ind_application::repos::{Cursor, Page};
use ind_domain::{DocumentId, Tag, TagId, UserId};

use crate::cursor::{
    clamp_limit, decode_cursor_name, decode_cursor_ts, encode_cursor_name, encode_cursor_ts,
};

use super::{PgTagRepository, map_sqlx_error};

impl PgTagRepository {
    pub(super) async fn list_with_counts_impl(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
        scope: Option<&str>,
    ) -> Result<Page<(Tag, i64, i64)>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        #[derive(sqlx::FromRow)]
        struct TagWithCounts {
            id: Uuid,
            user_id: Uuid,
            name: String,
            color: Option<String>,
            parent_id: Option<Uuid>,
            created_at: DateTime<Utc>,
            item_count: Option<i64>,
            highlight_count: Option<i64>,
        }

        // The scope filter uses HAVING to only return tags that have at
        // least one association in the requested category. The $5 parameter
        // encodes 0 = no filter, 1 = document-only, 2 = highlight-only.
        let scope_flag: i32 = match scope {
            Some("document") => 1,
            Some("highlight") => 2,
            _ => 0,
        };

        let rows = if let Some(ref cursor) = cursor {
            let (cursor_name, cursor_id) = decode_cursor_name(cursor)?;
            sqlx::query_as!(
                TagWithCounts,
                "SELECT t.id, t.user_id, t.name, t.color, t.parent_id, t.created_at, \
                 COUNT(DISTINCT le.id) AS item_count, \
                 COUNT(DISTINCT ht.highlight_id) AS highlight_count \
                 FROM tags t \
                 LEFT JOIN library_entry_tags let ON let.tag_id = t.id \
                 LEFT JOIN library_entries le \
                   ON le.id = let.library_entry_id AND le.deleted_at IS NULL \
                 LEFT JOIN highlight_tags ht ON ht.tag_id = t.id \
                 WHERE t.user_id = $1 AND (t.name, t.id) > ($2, $3) \
                 GROUP BY t.id \
                 HAVING ($5 = 0) \
                    OR ($5 = 1 AND COUNT(DISTINCT le.id) > 0) \
                    OR ($5 = 2 AND COUNT(DISTINCT ht.highlight_id) > 0) \
                 ORDER BY t.name ASC, t.id ASC \
                 LIMIT $4",
                user_id.into_uuid(),
                &cursor_name,
                cursor_id,
                fetch_limit,
                scope_flag,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        } else {
            sqlx::query_as!(
                TagWithCounts,
                "SELECT t.id, t.user_id, t.name, t.color, t.parent_id, t.created_at, \
                 COUNT(DISTINCT le.id) AS item_count, \
                 COUNT(DISTINCT ht.highlight_id) AS highlight_count \
                 FROM tags t \
                 LEFT JOIN library_entry_tags let ON let.tag_id = t.id \
                 LEFT JOIN library_entries le \
                   ON le.id = let.library_entry_id AND le.deleted_at IS NULL \
                 LEFT JOIN highlight_tags ht ON ht.tag_id = t.id \
                 WHERE t.user_id = $1 \
                 GROUP BY t.id \
                 HAVING ($3 = 0) \
                    OR ($3 = 1 AND COUNT(DISTINCT le.id) > 0) \
                    OR ($3 = 2 AND COUNT(DISTINCT ht.highlight_id) > 0) \
                 ORDER BY t.name ASC, t.id ASC \
                 LIMIT $2",
                user_id.into_uuid(),
                fetch_limit,
                scope_flag,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        };

        let has_more = rows.len() as i64 > limit;
        let take = if has_more { limit as usize } else { rows.len() };

        let items: Vec<(Tag, i64, i64)> = rows
            .into_iter()
            .take(take)
            .map(|r| {
                let tag = Tag {
                    id: TagId::from_uuid(r.id),
                    user_id: UserId::from_uuid(r.user_id),
                    name: r.name,
                    color: r.color,
                    parent_id: r.parent_id.map(TagId::from_uuid),
                    created_at: r.created_at,
                };
                (
                    tag,
                    r.item_count.unwrap_or(0),
                    r.highlight_count.unwrap_or(0),
                )
            })
            .collect();

        let next_cursor = if has_more {
            items
                .last()
                .map(|(t, _, _)| encode_cursor_name(&t.name, t.id.into_uuid()))
        } else {
            None
        };

        Ok(Page { items, next_cursor })
    }

    pub(super) async fn list_tag_highlights_impl(
        &self,
        tag_id: TagId,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<TaggedHighlight>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        #[derive(sqlx::FromRow)]
        struct HighlightRow {
            link_added_at: DateTime<Utc>,
            id: Uuid,
            document_id: Uuid,
            user_id: Uuid,
            color: String,
            text_content: String,
            locator: Option<serde_json::Value>,
            source_locator: Option<serde_json::Value>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
            item_title: String,
            item_domain: Option<String>,
            item_type: String,
            note: Option<String>,
        }

        let rows = if let Some(ref cursor) = cursor {
            let (cursor_ts, cursor_id) = decode_cursor_ts(cursor)?;
            sqlx::query_as!(
                HighlightRow,
                "SELECT ht.added_at AS link_added_at, \
                 h.id, h.document_id AS \"document_id!\", h.user_id, h.color, h.text_content, \
                 h.locator, h.source_locator, h.created_at, h.updated_at, \
                 d.title AS item_title, d.domain AS item_domain, \
                 d.document_type AS item_type, hn.body AS note \
                 FROM highlights h \
                 JOIN highlight_tags ht ON ht.highlight_id = h.id \
                 JOIN documents d ON d.id = h.document_id AND d.user_id = h.user_id \
                 LEFT JOIN highlight_notes hn ON hn.highlight_id = h.id \
                 WHERE ht.tag_id = $1 AND h.user_id = $2 AND h.document_id IS NOT NULL \
                 AND (ht.added_at, h.id) < ($3, $4) \
                 ORDER BY ht.added_at DESC, h.id DESC \
                 LIMIT $5",
                tag_id.into_uuid(),
                user_id.into_uuid(),
                cursor_ts,
                cursor_id,
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        } else {
            sqlx::query_as!(
                HighlightRow,
                "SELECT ht.added_at AS link_added_at, \
                 h.id, h.document_id AS \"document_id!\", h.user_id, h.color, h.text_content, \
                 h.locator, h.source_locator, h.created_at, h.updated_at, \
                 d.title AS item_title, d.domain AS item_domain, \
                 d.document_type AS item_type, hn.body AS note \
                 FROM highlights h \
                 JOIN highlight_tags ht ON ht.highlight_id = h.id \
                 JOIN documents d ON d.id = h.document_id AND d.user_id = h.user_id \
                 LEFT JOIN highlight_notes hn ON hn.highlight_id = h.id \
                 WHERE ht.tag_id = $1 AND h.user_id = $2 AND h.document_id IS NOT NULL \
                 ORDER BY ht.added_at DESC, h.id DESC \
                 LIMIT $3",
                tag_id.into_uuid(),
                user_id.into_uuid(),
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        };

        let has_more = rows.len() as i64 > limit;
        let take = if has_more { limit as usize } else { rows.len() };

        let next_cursor = if has_more {
            rows.get(take - 1)
                .map(|r| encode_cursor_ts(r.link_added_at, r.id))
        } else {
            None
        };

        let items: Vec<TaggedHighlight> = rows
            .into_iter()
            .take(take)
            .map(|r| {
                let locator: Option<ind_domain::HighlightLocator> = r
                    .locator
                    .map(serde_json::from_value)
                    .transpose()
                    .map_err(|e| AppError::Repository(Box::new(e)))?;
                let source_locator: Option<ind_domain::HighlightSourceLocator> = r
                    .source_locator
                    .map(serde_json::from_value)
                    .transpose()
                    .map_err(|e| AppError::Repository(Box::new(e)))?;
                Ok(TaggedHighlight {
                    highlight: ind_domain::Highlight {
                        id: ind_domain::HighlightId::from_uuid(r.id),
                        document_id: DocumentId::from_uuid(r.document_id),
                        user_id: UserId::from_uuid(r.user_id),
                        color: r.color,
                        text_content: r.text_content,
                        locator,
                        source_locator,
                        created_at: r.created_at,
                        updated_at: r.updated_at,
                    },
                    item_title: r.item_title,
                    item_domain: r.item_domain,
                    item_type: r.item_type,
                    note: r.note,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok(Page { items, next_cursor })
    }
}
