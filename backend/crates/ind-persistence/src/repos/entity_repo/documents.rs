use ind_application::AppError;
use ind_application::repos::entity::EntityDocument;
use ind_application::repos::{Cursor, Page};
use ind_domain::{DocumentId, EntityId, EntitySummary, UserId};

use crate::cursor::{clamp_limit, decode_cursor_ts, encode_cursor_ts};

use super::rows::{EntityDocumentRow, EntitySummaryRow};
use super::{PgEntityRepository, map_sqlx_error};

impl PgEntityRepository {
    pub(super) async fn list_entity_documents_impl(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<EntityDocument>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = cursor {
            let (saved_at, document_id) = decode_cursor_ts(cursor)?;
            sqlx::query_as!(
                EntityDocumentRow,
                r#"
                SELECT le.document_id, d.title, d.author, d.excerpt, d.domain, le.saved_at
                FROM entity_mentions em
                JOIN library_entries le
                    ON le.document_id = em.document_id AND le.user_id = $2 AND le.deleted_at IS NULL
                JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id
                WHERE em.entity_id = $1
                  AND em.document_id IS NOT NULL
                  AND (le.saved_at, le.document_id) < ($3, $4)
                ORDER BY le.saved_at DESC, le.document_id DESC
                LIMIT $5
                "#,
                entity_id.into_uuid(),
                user_id.into_uuid(),
                saved_at,
                document_id,
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        } else {
            sqlx::query_as!(
                EntityDocumentRow,
                r#"
                SELECT le.document_id, d.title, d.author, d.excerpt, d.domain, le.saved_at
                FROM entity_mentions em
                JOIN library_entries le
                    ON le.document_id = em.document_id AND le.user_id = $2 AND le.deleted_at IS NULL
                JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id
                WHERE em.entity_id = $1
                  AND em.document_id IS NOT NULL
                ORDER BY le.saved_at DESC, le.document_id DESC
                LIMIT $3
                "#,
                entity_id.into_uuid(),
                user_id.into_uuid(),
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
        };

        let has_more = rows.len() as i64 > limit;
        let take = if has_more { limit as usize } else { rows.len() };
        let documents: Vec<EntityDocument> = rows.into_iter().take(take).map(Into::into).collect();

        let next_cursor = if has_more {
            documents
                .last()
                .map(|doc| encode_cursor_ts(doc.saved_at, doc.document_id.into_uuid()))
        } else {
            None
        };

        Ok(Page {
            items: documents,
            next_cursor,
        })
    }

    pub(super) async fn list_document_ids_for_entity_impl(
        &self,
        user_id: UserId,
        entity_id: EntityId,
    ) -> Result<Vec<DocumentId>, AppError> {
        let document_ids = sqlx::query_scalar!(
            r#"
            SELECT le.document_id
            FROM entity_mentions em
            JOIN library_entries le
                ON le.document_id = em.document_id AND le.user_id = $2 AND le.deleted_at IS NULL
            WHERE em.entity_id = $1
              AND em.document_id IS NOT NULL
            ORDER BY le.saved_at DESC, le.document_id DESC
            "#,
            entity_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(document_ids
            .into_iter()
            .map(DocumentId::from_uuid)
            .collect())
    }

    pub(super) async fn list_entities_for_document_impl(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Vec<EntitySummary>, AppError> {
        let rows = sqlx::query_as!(
            EntitySummaryRow,
            r#"
            WITH item_entity_mentions AS (
                SELECT entity_id, mention_count
                FROM entity_mentions
                WHERE document_id = $2
            ),
            entity_stats AS (
                SELECT
                    e.id,
                    e.user_id,
                    e.name,
                    e.entity_type,
                    e.description,
                    e.created_at,
                    COALESCE(SUM(em.mention_count), 0)::int8 AS total_mentions,
                    COUNT(DISTINCT em.document_id)::int8 AS item_count,
                    COALESCE(MIN(em.first_seen_at), e.created_at) AS first_seen_at,
                    COALESCE(MAX(le.saved_at), e.created_at) AS last_seen_at
                FROM entities e
                JOIN item_entity_mentions iem ON iem.entity_id = e.id
                LEFT JOIN entity_mentions em ON em.entity_id = e.id
                LEFT JOIN library_entries le
                    ON le.document_id = em.document_id AND le.user_id = e.user_id AND le.deleted_at IS NULL
                WHERE e.user_id = $1
                GROUP BY e.id, e.user_id, e.name, e.entity_type, e.description, e.created_at
            )
            SELECT
                es.id AS "id!",
                es.user_id AS "user_id!",
                es.name AS "name!",
                es.entity_type AS "entity_type!",
                es.description,
                es.created_at AS "created_at!",
                es.total_mentions AS "total_mentions!",
                es.item_count AS "item_count!",
                es.first_seen_at AS "first_seen_at!",
                es.last_seen_at AS "last_seen_at!"
            FROM entity_stats es
            JOIN item_entity_mentions iem ON iem.entity_id = es.id
            ORDER BY iem.mention_count DESC, lower(es.name) ASC
            LIMIT 100
            "#,
            user_id.into_uuid(),
            document_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(TryInto::try_into).collect()
    }
}
