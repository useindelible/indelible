use chrono::Utc;

use ind_application::AppError;
use ind_application::repos::event::MutationSideEffects;
use ind_domain::{HighlightId, TagId, UserId};

use crate::repos::write_helpers::apply_mutation_side_effects_tx;

use super::{PgTagRepository, map_sqlx_error};

impl PgTagRepository {
    pub(super) async fn replace_for_highlight_impl(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
        tag_ids: &[TagId],
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        sqlx::query!(
            "DELETE FROM highlight_tags \
             WHERE highlight_id = $1 \
               AND EXISTS (SELECT 1 FROM highlights WHERE id = $1 AND user_id = $2)",
            highlight_id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        let now = Utc::now();
        for tag_id in tag_ids {
            sqlx::query!(
                "INSERT INTO highlight_tags (highlight_id, tag_id, added_at) \
                 VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
                highlight_id.into_uuid(),
                tag_id.into_uuid(),
                now,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        apply_mutation_side_effects_tx(&mut tx, effects).await?;

        tx.commit()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(())
    }
}
