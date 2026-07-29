use chrono::Utc;

use super::types::{RecentSearchRow, map_sqlx_error};
use super::*;

impl PgSearchRepository {
    pub(super) async fn upsert_recent_search_impl(
        &self,
        user_id: UserId,
        raw_query: &str,
        normalized_query: &str,
        max_entries: i64,
    ) -> Result<RecentSearch, AppError> {
        let now = Utc::now();
        let id = RecentSearchId::new();
        let row = sqlx::query_as!(
            RecentSearchRow,
            r#"
            INSERT INTO recent_searches (
                id,
                user_id,
                raw_query,
                normalized_query,
                last_searched_at,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $5, $5)
            ON CONFLICT (user_id, normalized_query) DO UPDATE SET
                raw_query = EXCLUDED.raw_query,
                last_searched_at = EXCLUDED.last_searched_at,
                updated_at = EXCLUDED.updated_at
            RETURNING
                id,
                user_id,
                raw_query,
                normalized_query,
                last_searched_at,
                created_at,
                updated_at
            "#,
            id.as_uuid(),
            user_id.into_uuid(),
            raw_query,
            normalized_query,
            now,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query!(
            r#"
            DELETE FROM recent_searches
            WHERE user_id = $1
              AND id NOT IN (
                    SELECT id
                    FROM recent_searches
                    WHERE user_id = $1
                    ORDER BY last_searched_at DESC, updated_at DESC
                    LIMIT $2
              )
            "#,
            user_id.into_uuid(),
            max_entries,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(RecentSearch::from(row))
    }
    pub(super) async fn list_recent_searches_impl(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<RecentSearch>, AppError> {
        let rows = sqlx::query_as!(
            RecentSearchRow,
            r#"
            SELECT
                id,
                user_id,
                raw_query,
                normalized_query,
                last_searched_at,
                created_at,
                updated_at
            FROM recent_searches
            WHERE user_id = $1
            ORDER BY last_searched_at DESC, updated_at DESC
            LIMIT $2
            "#,
            user_id.into_uuid(),
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(RecentSearch::from).collect())
    }
    pub(super) async fn suggest_recent_searches_impl(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<RecentSearch>, AppError> {
        let prefix = format!("{}%", prefix.to_lowercase());
        let rows = sqlx::query_as!(
            RecentSearchRow,
            r#"
            SELECT
                id,
                user_id,
                raw_query,
                normalized_query,
                last_searched_at,
                created_at,
                updated_at
            FROM recent_searches
            WHERE user_id = $1
              AND normalized_query LIKE $2
            ORDER BY last_searched_at DESC, updated_at DESC
            LIMIT $3
            "#,
            user_id.into_uuid(),
            prefix,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(RecentSearch::from).collect())
    }
    pub(super) async fn delete_recent_search_impl(
        &self,
        user_id: UserId,
        recent_search_id: RecentSearchId,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM recent_searches WHERE id = $1 AND user_id = $2",
            recent_search_id.as_uuid(),
            user_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
    pub(super) async fn clear_recent_searches_impl(&self, user_id: UserId) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM recent_searches WHERE user_id = $1",
            user_id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
