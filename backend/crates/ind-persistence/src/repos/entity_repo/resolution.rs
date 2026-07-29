use chrono::Utc;

use ind_application::AppError;
use ind_domain::{Entity, EntityId, EntityType, UserId};

use super::rows::{EntityRow, format_entity_type};
use super::{PgEntityRepository, map_sqlx_error};

impl PgEntityRepository {
    pub(super) async fn find_for_resolution_impl(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
    ) -> Result<Option<Entity>, AppError> {
        let row = sqlx::query_as!(
            EntityRow,
            r#"
            SELECT
                id AS "id!",
                user_id AS "user_id!",
                name AS "name!",
                entity_type AS "entity_type!",
                description,
                created_at AS "created_at!"
            FROM (
                SELECT 0 AS pref, e.id, e.user_id, e.name, e.entity_type, e.description, e.created_at
                FROM entities e
                WHERE e.user_id = $1 AND e.entity_type = $2 AND e.name = $3
                UNION ALL
                SELECT 1 AS pref, e.id, e.user_id, e.name, e.entity_type, e.description, e.created_at
                FROM entity_aliases a
                JOIN entities e ON e.id = a.entity_id
                WHERE a.user_id = $1 AND e.user_id = $1 AND a.entity_type = $2 AND a.name = $3
            ) candidates
            ORDER BY pref
            LIMIT 1
            "#,
            user_id.into_uuid(),
            format_entity_type(entity_type),
            name,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into).transpose()
    }

    pub(super) async fn block_candidates_impl(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        limit: i64,
    ) -> Result<Vec<Entity>, AppError> {
        let rows = sqlx::query_as!(
            EntityRow,
            r#"
            SELECT id, user_id, name, entity_type, description, created_at
            FROM entities b
            WHERE b.user_id = $1 AND b.entity_type = $2 AND b.name <> $3
              AND (
                    lower(btrim(b.name)) = lower(btrim($3))
                 OR similarity(b.name, $3) >= 0.45
                 OR regexp_split_to_array(lower($3), '\s+')   <@ regexp_split_to_array(lower(b.name), '\s+')
                 OR regexp_split_to_array(lower(b.name), '\s+') <@ regexp_split_to_array(lower($3), '\s+')
              )
            ORDER BY similarity(b.name, $3) DESC
            LIMIT $4
            "#,
            user_id.into_uuid(),
            format_entity_type(entity_type),
            name,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Entity>, AppError>>()
    }

    pub(super) async fn insert_canonical_impl(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        description: Option<&str>,
    ) -> Result<Entity, AppError> {
        let row = sqlx::query_as!(
            EntityRow,
            r#"
            INSERT INTO entities (id, user_id, name, entity_type, description, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id, name, entity_type)
            DO UPDATE SET description = COALESCE(EXCLUDED.description, entities.description)
            RETURNING id, user_id, name, entity_type, description, created_at
            "#,
            EntityId::new().into_uuid(),
            user_id.into_uuid(),
            name,
            format_entity_type(entity_type),
            description,
            Utc::now(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    pub(super) async fn insert_alias_impl(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        entity_id: EntityId,
    ) -> Result<(), AppError> {
        // Defense in depth: only create/repoint the alias when the target entity
        // belongs to the same user, so a caller passing a foreign EntityId cannot
        // forge a cross-tenant alias edge.
        sqlx::query!(
            r#"
            INSERT INTO entity_aliases (user_id, entity_type, name, entity_id)
            SELECT $1, $2, $3, $4
            WHERE EXISTS (
                SELECT 1 FROM entities e WHERE e.id = $4 AND e.user_id = $1
            )
            ON CONFLICT (user_id, entity_type, name) DO UPDATE SET entity_id = EXCLUDED.entity_id
            "#,
            user_id.into_uuid(),
            format_entity_type(entity_type),
            name,
            entity_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    pub(super) async fn register_alias_if_absent_impl(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        entity_id: EntityId,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO entity_aliases (user_id, entity_type, name, entity_id)
            SELECT $1, $2, $3, $4
            WHERE NOT EXISTS (
                SELECT 1 FROM entities e
                WHERE e.user_id = $1 AND e.entity_type = $2 AND e.name = $3
            )
              AND NOT EXISTS (
                SELECT 1 FROM entity_aliases a
                WHERE a.user_id = $1 AND a.entity_type = $2 AND a.name = $3
            )
            ON CONFLICT DO NOTHING
            "#,
            user_id.into_uuid(),
            format_entity_type(entity_type),
            name,
            entity_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }
}
