use super::types::map_sqlx_error;
use super::*;

impl PgSearchRepository {
    pub(super) async fn suggest_tags_impl(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        let prefix = format!("{}%", prefix.to_lowercase());
        sqlx::query_scalar!(
            r#"
            SELECT candidate AS "candidate!"
            FROM (
                SELECT t.name::text AS candidate
                FROM tags t
                WHERE t.user_id = $1
                  AND lower(t.name) LIKE $2

                UNION

                SELECT ta.alias::text AS candidate
                FROM tag_aliases ta
                JOIN tags t ON t.id = ta.tag_id
                WHERE t.user_id = $1
                  AND lower(ta.alias) LIKE $2
            ) suggestions
            ORDER BY lower(candidate), candidate
            LIMIT $3
            "#,
            user_id.into_uuid(),
            prefix,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)
    }
    pub(super) async fn suggest_collections_impl(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        let prefix = format!("{}%", prefix.to_lowercase());
        sqlx::query_scalar!(
            r#"
            SELECT name
            FROM collections
            WHERE user_id = $1
              AND lower(name) LIKE $2
            ORDER BY lower(name)
            LIMIT $3
            "#,
            user_id.into_uuid(),
            prefix,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)
    }

    pub(super) async fn suggest_senders_impl(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        let prefix = format!("{}%", prefix.to_lowercase());
        sqlx::query_scalar!(
            r#"
            SELECT es.canonical_addr AS "canonical_addr!"
            FROM email_senders es
            JOIN documents d ON d.sender_id = es.id AND d.user_id = $1
            WHERE es.user_id = $1
              AND lower(es.canonical_addr) LIKE $2
            GROUP BY es.canonical_addr
            ORDER BY count(d.id) DESC, lower(es.canonical_addr)
            LIMIT $3
            "#,
            user_id.into_uuid(),
            prefix,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)
    }

    pub(super) async fn suggest_sender_domains_impl(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        let prefix = format!("{}%", prefix.to_lowercase());
        sqlx::query_scalar!(
            r#"
            SELECT split_part(es.canonical_addr, '@', 2) AS "domain!"
            FROM email_senders es
            JOIN documents d ON d.sender_id = es.id AND d.user_id = $1
            WHERE es.user_id = $1
              AND split_part(es.canonical_addr, '@', 2) <> ''
              AND lower(split_part(es.canonical_addr, '@', 2)) LIKE $2
            GROUP BY split_part(es.canonical_addr, '@', 2)
            ORDER BY count(d.id) DESC, lower(split_part(es.canonical_addr, '@', 2))
            LIMIT $3
            "#,
            user_id.into_uuid(),
            prefix,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)
    }

    pub(super) async fn suggest_list_ids_impl(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        let prefix = format!("{}%", prefix.to_lowercase());
        sqlx::query_scalar!(
            r#"
            SELECT es.list_id AS "list_id!"
            FROM email_senders es
            JOIN documents d ON d.sender_id = es.id AND d.user_id = $1
            WHERE es.user_id = $1
              AND es.list_id IS NOT NULL
              AND lower(es.list_id) LIKE $2
            GROUP BY es.list_id
            ORDER BY count(d.id) DESC, lower(es.list_id)
            LIMIT $3
            "#,
            user_id.into_uuid(),
            prefix,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)
    }

    pub(super) async fn suggest_authors_impl(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        let prefix = format!("{}%", prefix.to_lowercase());
        sqlx::query_scalar!(
            r#"
            SELECT d.author AS "author!"
            FROM documents d
            WHERE d.user_id = $1
              AND d.author IS NOT NULL
              AND d.author <> ''
              AND lower(d.author) LIKE $2
            GROUP BY d.author
            ORDER BY count(*) DESC, lower(d.author)
            LIMIT $3
            "#,
            user_id.into_uuid(),
            prefix,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)
    }
}
