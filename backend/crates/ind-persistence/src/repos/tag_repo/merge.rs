use uuid::Uuid;

use ind_application::AppError;
use ind_domain::{DomainError, Tag, TagId, UserId};

use super::rows::TagRow;
use super::{PgTagRepository, map_sqlx_error};

impl PgTagRepository {
    pub(super) async fn merge_tags_impl(
        &self,
        source_ids: &[TagId],
        target_id: TagId,
        user_id: UserId,
    ) -> Result<Tag, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        let source_uuids: Vec<Uuid> = source_ids.iter().map(|id| id.into_uuid()).collect();

        let target = sqlx::query_as!(
            TagRow,
            "SELECT id, user_id, name, color, parent_id, created_at \
             FROM tags WHERE id = $1 AND user_id = $2",
            target_id.into_uuid(),
            user_id.into_uuid()
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "tag",
                id: target_id.to_string(),
            })
        })?;

        let source_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM tags WHERE id = ANY($1) AND user_id = $2",
            &source_uuids,
            user_id.into_uuid()
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        if source_count.unwrap_or(0) != source_ids.len() as i64 {
            return Err(AppError::Domain(DomainError::Validation {
                field: "source_ids".into(),
                message: "one or more source tags not found".into(),
            }));
        }

        sqlx::query!(
            "INSERT INTO library_entry_tags (user_id, library_entry_id, tag_id, source, added_at) \
             SELECT user_id, library_entry_id, $2, source, added_at FROM library_entry_tags \
             WHERE tag_id = ANY($1) \
             ON CONFLICT (library_entry_id, tag_id) DO NOTHING",
            &source_uuids,
            target_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query!(
            "DELETE FROM library_entry_tags WHERE tag_id = ANY($1)",
            &source_uuids,
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query!(
            "INSERT INTO tag_aliases (id, tag_id, alias) \
             SELECT gen_random_uuid(), $2, alias FROM tag_aliases \
             WHERE tag_id = ANY($1) \
             ON CONFLICT (tag_id, alias) DO NOTHING",
            &source_uuids,
            target_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query!(
            "DELETE FROM tag_aliases WHERE tag_id = ANY($1)",
            &source_uuids,
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        for source_id in source_ids {
            sqlx::query!(
                "INSERT INTO tag_aliases (id, tag_id, alias) \
                 SELECT gen_random_uuid(), $2, name FROM tags \
                 WHERE id = $1 \
                 ON CONFLICT (tag_id, alias) DO NOTHING",
                source_id.into_uuid(),
                target_id.into_uuid(),
            )
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        sqlx::query!(
            "DELETE FROM tags WHERE id = ANY($1) AND user_id = $2",
            &source_uuids,
            user_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        tx.commit()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(Tag::from(target))
    }
}
