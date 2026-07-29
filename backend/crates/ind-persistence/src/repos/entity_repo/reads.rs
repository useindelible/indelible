use ind_application::AppError;
use ind_application::repos::{Cursor, Page};
use ind_domain::{DomainError, Entity, EntityDetail, EntityId, EntitySummary, EntityType, UserId};

use crate::cursor::{clamp_limit, decode_cursor_entity, encode_cursor_entity};

use super::rows::{EntityCoOccurrenceRow, EntityRow, EntitySummaryRow, format_entity_type};
use super::{PgEntityRepository, map_sqlx_error};

impl PgEntityRepository {
    pub(super) async fn find_by_id_for_user_impl(
        &self,
        id: EntityId,
        user_id: UserId,
    ) -> Result<Option<Entity>, AppError> {
        let row = sqlx::query_as!(
            EntityRow,
            r#"
            SELECT id, user_id, name, entity_type, description, created_at
            FROM entities
            WHERE id = $1 AND user_id = $2
            "#,
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into).transpose()
    }

    pub(super) async fn list_summaries_impl(
        &self,
        user_id: UserId,
        entity_type: Option<EntityType>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<EntitySummary>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        let (cursor_mentions, cursor_items, cursor_name, cursor_id) = match cursor.as_ref() {
            Some(cursor) => {
                let (mentions, items, name, id) = decode_cursor_entity(cursor)?;
                (Some(mentions), Some(items), Some(name), Some(id))
            }
            None => (None, None, None, None),
        };

        let rows = sqlx::query_as!(
            EntitySummaryRow,
            r#"
            WITH entity_stats AS (
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
                LEFT JOIN entity_mentions em ON em.entity_id = e.id
                LEFT JOIN library_entries le
                    ON le.document_id = em.document_id AND le.user_id = e.user_id AND le.deleted_at IS NULL
                WHERE e.user_id = $1
                  AND ($2::text IS NULL OR e.entity_type = $2)
                GROUP BY e.id, e.user_id, e.name, e.entity_type, e.description, e.created_at
                HAVING COUNT(DISTINCT em.document_id) > 0
            )
            SELECT
                id AS "id!",
                user_id AS "user_id!",
                name AS "name!",
                entity_type AS "entity_type!",
                description,
                created_at AS "created_at!",
                total_mentions AS "total_mentions!",
                item_count AS "item_count!",
                first_seen_at AS "first_seen_at!",
                last_seen_at AS "last_seen_at!"
            FROM entity_stats
            WHERE (
                    $3::bigint IS NULL
                    OR total_mentions < $3
                    OR (total_mentions = $3 AND item_count < $4)
                    OR (total_mentions = $3 AND item_count = $4 AND lower(name) > lower($5))
                    OR (total_mentions = $3 AND item_count = $4 AND lower(name) = lower($5) AND name > $5)
                    OR (total_mentions = $3 AND item_count = $4 AND lower(name) = lower($5) AND name = $5 AND id > $6)
                  )
            ORDER BY total_mentions DESC, item_count DESC, lower(name) ASC, name ASC, id ASC
            LIMIT $7
            "#,
            user_id.into_uuid(),
            entity_type.map(format_entity_type),
            cursor_mentions,
            cursor_items,
            cursor_name.as_deref(),
            cursor_id,
            fetch_limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let has_more = rows.len() as i64 > limit;
        let take = if has_more { limit as usize } else { rows.len() };

        let items: Vec<EntitySummary> = rows
            .into_iter()
            .take(take)
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;

        let next_cursor = if has_more {
            items.last().map(|summary| {
                encode_cursor_entity(
                    summary.total_mentions,
                    summary.item_count,
                    &summary.entity.name,
                    summary.entity.id.into_uuid(),
                )
            })
        } else {
            None
        };

        Ok(Page { items, next_cursor })
    }

    pub(super) async fn get_detail_impl(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        co_occurrence_limit: u32,
    ) -> Result<EntityDetail, AppError> {
        let row = sqlx::query_as!(
            EntitySummaryRow,
            r#"
            SELECT
                e.id,
                e.user_id,
                e.name,
                e.entity_type,
                e.description,
                e.created_at,
                COALESCE(SUM(em.mention_count), 0)::int8 AS "total_mentions!",
                COUNT(DISTINCT em.document_id)::int8 AS "item_count!",
                COALESCE(MIN(em.first_seen_at), e.created_at) AS "first_seen_at!",
                COALESCE(MAX(le.saved_at), e.created_at) AS "last_seen_at!"
            FROM entities e
            LEFT JOIN entity_mentions em ON em.entity_id = e.id
            LEFT JOIN library_entries le
                ON le.document_id = em.document_id AND le.user_id = e.user_id AND le.deleted_at IS NULL
            WHERE e.user_id = $1
              AND e.id = $2
            GROUP BY e.id, e.user_id, e.name, e.entity_type, e.description, e.created_at
            "#,
            user_id.into_uuid(),
            entity_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "entity",
                id: entity_id.to_string(),
            })
        })?;

        let summary = EntitySummary::try_from(row)?;
        let co_occurring = sqlx::query_as!(
            EntityCoOccurrenceRow,
            r#"
            WITH source_documents AS (
                SELECT em.document_id
                FROM entity_mentions em
                JOIN library_entries le
                    ON le.document_id = em.document_id AND le.user_id = $2 AND le.deleted_at IS NULL
                WHERE em.entity_id = $1
                  AND em.document_id IS NOT NULL
            ),
            shared_entities AS (
                SELECT
                    e.id,
                    e.user_id,
                    e.name,
                    e.entity_type,
                    e.description,
                    e.created_at,
                    COUNT(DISTINCT em.document_id)::int8 AS shared_item_count
                FROM entity_mentions em
                JOIN source_documents sd ON sd.document_id = em.document_id
                JOIN entities e ON e.id = em.entity_id
                WHERE e.user_id = $2
                  AND e.id <> $1
                GROUP BY e.id, e.user_id, e.name, e.entity_type, e.description, e.created_at
            ),
            totals AS (
                SELECT
                    em.entity_id,
                    COALESCE(SUM(em.mention_count), 0)::int8 AS total_mentions
                FROM entity_mentions em
                JOIN library_entries le
                    ON le.document_id = em.document_id AND le.user_id = $2 AND le.deleted_at IS NULL
                GROUP BY em.entity_id
            )
            SELECT
                se.id AS "id!",
                se.user_id AS "user_id!",
                se.name AS "name!",
                se.entity_type AS "entity_type!",
                se.description,
                se.created_at AS "created_at!",
                se.shared_item_count AS "shared_item_count!",
                COALESCE(t.total_mentions, 0) AS "total_mentions!"
            FROM shared_entities se
            LEFT JOIN totals t ON t.entity_id = se.id
            ORDER BY se.shared_item_count DESC, COALESCE(t.total_mentions, 0) DESC, lower(se.name), se.name, se.id
            LIMIT $3
            "#,
            entity_id.into_uuid(),
            user_id.into_uuid(),
            clamp_limit(co_occurrence_limit),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>, _>>()?;

        Ok(EntityDetail {
            entity: summary.entity,
            total_mentions: summary.total_mentions,
            item_count: summary.item_count,
            first_seen_at: summary.first_seen_at,
            last_seen_at: summary.last_seen_at,
            co_occurring,
        })
    }
}
