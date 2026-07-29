use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use ind_application::AppError;
use ind_domain::{DocumentId, DomainError, Entity, EntityId, UserId};

use super::rows::EntityRow;
use super::{PgEntityRepository, map_sqlx_error};

impl PgEntityRepository {
    pub(super) async fn update_fields_impl(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        name: Option<&str>,
        description: Option<Option<&str>>,
    ) -> Result<Entity, AppError> {
        let description_should_update = description.is_some();
        let description_value = description.flatten();

        let row = sqlx::query_as!(
            EntityRow,
            r#"
            UPDATE entities
            SET name = COALESCE($3::text, name),
                description = CASE WHEN $4::boolean THEN $5::text ELSE description END
            WHERE id = $1
              AND user_id = $2
            RETURNING id, user_id, name, entity_type, description, created_at
            "#,
            entity_id.into_uuid(),
            user_id.into_uuid(),
            name,
            description_should_update,
            description_value,
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

        row.try_into()
    }

    pub(super) async fn merge_entities_impl(
        &self,
        user_id: UserId,
        source_id: EntityId,
        target_id: EntityId,
    ) -> Result<Entity, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        sqlx::query_as!(
            EntityRow,
            r#"
            SELECT id, user_id, name, entity_type, description, created_at
            FROM entities
            WHERE id = $1 AND user_id = $2
            FOR UPDATE
            "#,
            source_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "entity",
                id: source_id.to_string(),
            })
        })?;

        let target = sqlx::query_as!(
            EntityRow,
            r#"
            SELECT id, user_id, name, entity_type, description, created_at
            FROM entities
            WHERE id = $1 AND user_id = $2
            FOR UPDATE
            "#,
            target_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "entity",
                id: target_id.to_string(),
            })
        })?;

        sqlx::query!(
            r#"
            INSERT INTO entity_mentions (entity_id, document_id, mention_count, first_seen_at)
            SELECT
                $1,
                em.document_id,
                em.mention_count,
                em.first_seen_at
            FROM entity_mentions em
            WHERE em.entity_id = $2 AND em.document_id IS NOT NULL
            ON CONFLICT (entity_id, document_id) WHERE document_id IS NOT NULL
            DO UPDATE SET
                mention_count = entity_mentions.mention_count + EXCLUDED.mention_count,
                first_seen_at = LEAST(entity_mentions.first_seen_at, EXCLUDED.first_seen_at)
            "#,
            target_id.into_uuid(),
            source_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query!(
            "DELETE FROM entities WHERE id = $1 AND user_id = $2",
            source_id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        target.try_into()
    }

    pub(super) async fn set_document_mentions_impl(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        mentions: &[(EntityId, i32)],
    ) -> Result<(), AppError> {
        let mut totals: HashMap<EntityId, i32> = HashMap::new();
        for (entity_id, count) in mentions {
            *totals.entry(*entity_id).or_insert(0) += (*count).max(1);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        let now = Utc::now();
        let mut entity_ids: Vec<Uuid> = Vec::with_capacity(totals.len());
        for (entity_id, count) in &totals {
            entity_ids.push(entity_id.into_uuid());
            sqlx::query!(
                r#"
                INSERT INTO entity_mentions (entity_id, document_id, mention_count, first_seen_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (entity_id, document_id) WHERE document_id IS NOT NULL
                DO UPDATE SET
                    mention_count = EXCLUDED.mention_count,
                    first_seen_at = LEAST(entity_mentions.first_seen_at, EXCLUDED.first_seen_at)
                "#,
                entity_id.into_uuid(),
                document_id.into_uuid(),
                *count,
                now,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        if entity_ids.is_empty() {
            sqlx::query!(
                "DELETE FROM entity_mentions WHERE document_id = $1",
                document_id.into_uuid(),
            )
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        } else {
            sqlx::query!(
                "DELETE FROM entity_mentions WHERE document_id = $1 AND NOT (entity_id = ANY($2))",
                document_id.into_uuid(),
                &entity_ids,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        self.cleanup_orphan_entities(&mut tx, user_id).await?;

        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(())
    }

    async fn cleanup_orphan_entities(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: UserId,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM entities e
            WHERE e.user_id = $1
              AND NOT EXISTS (
                    SELECT 1
                    FROM entity_mentions em
                    WHERE em.entity_id = e.id
              )
              AND NOT EXISTS (
                    SELECT 1
                    FROM entity_aliases a
                    WHERE a.entity_id = e.id
              )
            "#,
            user_id.into_uuid(),
        )
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }
}
