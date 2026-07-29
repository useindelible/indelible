use std::collections::HashMap;

use uuid::Uuid;

use super::types::{EntitySuggestionRow, SearchEntityCardRow, SearchEntityChipRow, map_sqlx_error};
use super::*;

impl PgSearchRepository {
    pub(super) async fn list_entity_chips_for_documents_impl(
        &self,
        user_id: UserId,
        document_ids: &[DocumentId],
    ) -> Result<HashMap<DocumentId, Vec<SearchEntityChip>>, AppError> {
        if document_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let document_ids: Vec<Uuid> = document_ids
            .iter()
            .map(|document_id| document_id.into_uuid())
            .collect();
        let rows = sqlx::query_as!(
            SearchEntityChipRow,
            r#"
            SELECT
                em.document_id AS "document_id!",
                e.id AS entity_id,
                e.name,
                e.entity_type,
                em.mention_count
            FROM entity_mentions em
            JOIN entities e ON e.id = em.entity_id
            JOIN documents d ON d.id = em.document_id
            WHERE e.user_id = $1
              AND d.user_id = $1
              AND em.document_id = ANY($2)
            ORDER BY em.document_id, em.mention_count DESC, lower(e.name), e.name
            "#,
            user_id.into_uuid(),
            &document_ids,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut result = HashMap::new();
        for row in rows {
            let document_id = DocumentId::from_uuid(row.document_id);
            let chip = SearchEntityChip::try_from(row)?;
            result
                .entry(document_id)
                .or_insert_with(Vec::new)
                .push(chip);
        }

        Ok(result)
    }
    pub(super) async fn suggest_entities_impl(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<SearchEntityChip>, AppError> {
        let prefix = format!("{}%", prefix.to_lowercase());
        let rows = sqlx::query_as!(
            EntitySuggestionRow,
            r#"
            SELECT
                e.id AS entity_id,
                e.name,
                e.entity_type,
                COALESCE(sum(em.mention_count), 0)::int8 AS "mention_count!"
            FROM entities e
            JOIN entity_mentions em ON em.entity_id = e.id
            JOIN documents d ON d.id = em.document_id
            WHERE e.user_id = $1
              AND d.user_id = $1
              AND lower(e.name) LIKE $2
            GROUP BY e.id, e.name, e.entity_type
            ORDER BY COALESCE(sum(em.mention_count), 0) DESC, lower(e.name), e.name
            LIMIT $3
            "#,
            user_id.into_uuid(),
            prefix,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(SearchEntityChip::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
    pub(super) async fn find_entity_card_impl(
        &self,
        user_id: UserId,
        query: &str,
    ) -> Result<Option<SearchEntityCard>, AppError> {
        let normalized = query.trim().to_lowercase();
        if normalized.len() < 3 {
            return Ok(None);
        }

        let prefix = format!("{normalized}%");
        let row = sqlx::query_as!(
            SearchEntityCardRow,
            r#"
            WITH ranked_entities AS (
                SELECT
                    e.id AS entity_id,
                    e.name,
                    e.entity_type,
                    sum(em.mention_count)::int8 AS mention_count,
                    min(em.first_seen_at) AS first_seen_at,
                    max(COALESCE(le.saved_at, d.created_at)) AS last_seen_at,
                    CASE
                        WHEN lower(e.name) = $2 THEN 0
                        WHEN lower(e.name) LIKE $3 THEN 1
                        WHEN strpos(lower(e.name), $2) > 0 THEN 2
                        ELSE 3
                    END AS match_rank
                FROM entities e
                JOIN entity_mentions em ON em.entity_id = e.id
                JOIN documents d ON d.id = em.document_id
                LEFT JOIN library_entries le
                    ON le.document_id = d.id AND le.user_id = $1 AND le.deleted_at IS NULL
                WHERE e.user_id = $1
                  AND d.user_id = $1
                  AND (
                        lower(e.name) = $2
                        OR lower(e.name) LIKE $3
                        OR strpos(lower(e.name), $2) > 0
                  )
                GROUP BY e.id, e.name, e.entity_type
            )
            SELECT
                entity_id,
                name,
                entity_type,
                mention_count AS "mention_count!",
                first_seen_at AS "first_seen_at!",
                last_seen_at AS "last_seen_at!"
            FROM ranked_entities
            ORDER BY match_rank ASC, mention_count DESC, last_seen_at DESC, lower(name), name
            LIMIT 1
            "#,
            user_id.into_uuid(),
            normalized,
            prefix,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(SearchEntityCard::try_from).transpose()
    }
}
