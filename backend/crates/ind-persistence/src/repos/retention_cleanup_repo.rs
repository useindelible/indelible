use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::{
    FeedDeliveryPruneCounts, FeedDeliveryRetentionWindows, RetentionCleanupRepository,
};

pub struct PgRetentionCleanupRepository {
    pool: PgPool,
}

impl PgRetentionCleanupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("retention_cleanup", "retention cleanup conflict", err)
}

#[async_trait::async_trait]
impl RetentionCleanupRepository for PgRetentionCleanupRepository {
    async fn prune_feed_deliveries(
        &self,
        windows: FeedDeliveryRetentionWindows,
    ) -> Result<FeedDeliveryPruneCounts, AppError> {
        let dismissed = sqlx::query!(
            r#"
            DELETE FROM feed_deliveries
            WHERE dismissed_at IS NOT NULL
              AND dismissed_at < now() - ($1::int * interval '1 day')
            "#,
            windows.dismissed_days as i32,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?
        .rows_affected();

        let seen = sqlx::query!(
            r#"
            DELETE FROM feed_deliveries
            WHERE dismissed_at IS NULL
              AND seen_at IS NOT NULL
              AND seen_at < now() - ($1::int * interval '1 day')
            "#,
            windows.seen_days as i32,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?
        .rows_affected();

        let unseen = sqlx::query!(
            r#"
            DELETE FROM feed_deliveries
            WHERE dismissed_at IS NULL
              AND seen_at IS NULL
              AND delivered_at < now() - ($1::int * interval '1 day')
            "#,
            windows.unseen_days as i32,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?
        .rows_affected();

        Ok(FeedDeliveryPruneCounts {
            unseen,
            seen,
            dismissed,
        })
    }

    async fn compact_orphaned_feed_source_entries(
        &self,
        older_than_days: i64,
    ) -> Result<u64, AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM feed_source_entries fse
            WHERE fse.discovered_at < now() - ($1::int * interval '1 day')
              AND NOT EXISTS (
                  SELECT 1
                  FROM feed_deliveries fd
                  WHERE fd.source_entry_id = fse.id
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM document_origins origin
                  WHERE origin.origin_type = 'feed_source_entry'
                    AND origin.origin_id = fse.id
              )
            "#,
            older_than_days as i32,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(result.rows_affected())
    }

    async fn delete_disposable_documents(
        &self,
        windows: FeedDeliveryRetentionWindows,
        document_grace_days: i64,
    ) -> Result<u64, AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM documents d
            WHERE d.created_at < now() - ($1::int * interval '1 day')
              AND NOT EXISTS (
                  SELECT 1
                  FROM library_entries le
                  WHERE le.document_id = d.id
                    AND le.user_id = d.user_id
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM highlights h
                  WHERE h.document_id = d.id
                    AND h.user_id = d.user_id
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM item_notes n
                  WHERE n.document_id = d.id
                    AND n.user_id = d.user_id
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM mila_sessions ms
                  WHERE ms.document_id = d.id
                    AND ms.user_id = d.user_id
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM feed_deliveries fd
                  WHERE fd.document_id = d.id
                    AND fd.user_id = d.user_id
                    AND (
                        (
                            fd.dismissed_at IS NOT NULL
                            AND fd.dismissed_at >= now() - ($4::int * interval '1 day')
                        )
                        OR (
                            fd.dismissed_at IS NULL
                            AND fd.seen_at IS NOT NULL
                            AND fd.seen_at >= now() - ($3::int * interval '1 day')
                        )
                        OR (
                            fd.dismissed_at IS NULL
                            AND fd.seen_at IS NULL
                            AND fd.delivered_at >= now() - ($2::int * interval '1 day')
                        )
                    )
              )
            "#,
            document_grace_days as i32,
            windows.unseen_days as i32,
            windows.seen_days as i32,
            windows.dismissed_days as i32,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(result.rows_affected())
    }
}
