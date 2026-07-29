use ind_application::AppError;
use ind_application::repos::content_vector::{
    CollectionDocumentVectorQuery, CrossDocumentVectorQuery, SingleDocumentVectorQuery,
};
use ind_domain::SearchHit;

use super::PgContentVectorRepository;
use super::types::*;

impl PgContentVectorRepository {
    pub(super) async fn search_single_document_impl(
        &self,
        query: &SingleDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        let embedding_literal = build_vector_literal(&query.query_embedding);
        let section_kind = query.section_kind.map(search_section_kind_to_str);
        let rows = sqlx::query_as!(
            SearchHitRow,
            r#"
            SELECT
                cv.id AS chunk_id,
                cv.document_id AS document_id,
                d.title AS item_title,
                cv.content AS snippet,
                (1.0 - (cv.embedding <=> ($2::text)::vector)) AS "final_score!",
                d.document_type AS item_type,
                COALESCE(d.canonical_url, d.original_url) AS url,
                COALESCE(le.saved_at, d.created_at) AS "saved_at!",
                d.updated_at,
                CASE WHEN cv.section_key = '' THEN NULL ELSE cv.section_kind END AS section_kind,
                CASE WHEN cv.section_key = '' THEN NULL ELSE cv.section_key END AS section_key,
                NULL::text AS section_title
            FROM content_vectors cv
            JOIN documents d ON d.id = cv.document_id AND d.user_id = $1
            LEFT JOIN library_entries le
                ON le.document_id = d.id AND le.user_id = $1 AND le.deleted_at IS NULL
            WHERE cv.user_id = $1
              AND cv.document_id = $5
              AND cv.embedding_model = $6
              AND cv.embedding_dim = $7
              AND ($3::text IS NULL OR cv.section_kind = $3)
            ORDER BY
                cv.embedding <=> ($2::text)::vector ASC,
                COALESCE(le.saved_at, d.created_at) DESC,
                cv.document_id DESC,
                COALESCE(cv.section_key, '') DESC,
                cv.chunk_index ASC
            LIMIT $4
            "#,
            query.user_id.into_uuid(),
            &embedding_literal,
            section_kind,
            query.limit,
            query.document_id.into_uuid(),
            &query.embedding_model,
            query.embedding_dim,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(SearchHit::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    pub(super) async fn search_cross_document_impl(
        &self,
        query: &CrossDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        let embedding_literal = build_vector_literal(&query.query_embedding);
        let section_kind = query.section_kind.map(search_section_kind_to_str);
        let rows = sqlx::query_as!(
            SearchHitRow,
            r#"
            SELECT
                cv.id AS chunk_id,
                cv.document_id AS document_id,
                d.title AS item_title,
                cv.content AS snippet,
                (1.0 - (cv.embedding <=> ($2::text)::vector)) AS "final_score!",
                d.document_type AS item_type,
                COALESCE(d.canonical_url, d.original_url) AS url,
                COALESCE(le.saved_at, d.created_at) AS "saved_at!",
                d.updated_at,
                CASE WHEN cv.section_key = '' THEN NULL ELSE cv.section_kind END AS section_kind,
                CASE WHEN cv.section_key = '' THEN NULL ELSE cv.section_key END AS section_key,
                NULL::text AS section_title
            FROM content_vectors cv
            JOIN documents d ON d.id = cv.document_id AND d.user_id = $1
            LEFT JOIN library_entries le
                ON le.document_id = d.id AND le.user_id = $1 AND le.deleted_at IS NULL
            WHERE cv.user_id = $1
              AND cv.embedding_model = $5
              AND cv.embedding_dim = $6
              AND ($3::text IS NULL OR cv.section_kind = $3)
              AND (
                  le.id IS NOT NULL
                  OR EXISTS (
                      SELECT 1
                      FROM feed_deliveries fd_visible
                      WHERE fd_visible.document_id = d.id
                        AND fd_visible.user_id = $1
                        AND fd_visible.hidden_at IS NULL
                  )
              )
            ORDER BY
                cv.embedding <=> ($2::text)::vector ASC,
                COALESCE(le.saved_at, d.created_at) DESC,
                cv.document_id DESC,
                COALESCE(cv.section_key, '') DESC,
                cv.chunk_index ASC
            LIMIT $4
            "#,
            query.user_id.into_uuid(),
            &embedding_literal,
            section_kind,
            query.limit,
            &query.embedding_model,
            query.embedding_dim,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(SearchHit::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    pub(super) async fn search_collection_document_impl(
        &self,
        query: &CollectionDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        let embedding_literal = build_vector_literal(&query.query_embedding);
        let section_kind = query.section_kind.map(search_section_kind_to_str);
        let rows = sqlx::query_as!(
            SearchHitRow,
            r#"
            WITH RECURSIVE collection_scope(id) AS (
                SELECT c.id
                FROM collections c
                WHERE c.id = $5
                  AND c.user_id = $1
                UNION ALL
                SELECT child.id
                FROM collections child
                JOIN collection_scope parent ON parent.id = child.parent_id
                WHERE child.user_id = $1
                  AND $6::boolean
            )
            SELECT
                cv.id AS chunk_id,
                cv.document_id AS document_id,
                d.title AS item_title,
                cv.content AS snippet,
                (1.0 - (cv.embedding <=> ($2::text)::vector)) AS "final_score!",
                d.document_type AS item_type,
                COALESCE(d.canonical_url, d.original_url) AS url,
                COALESCE(le.saved_at, d.created_at) AS "saved_at!",
                d.updated_at,
                CASE WHEN cv.section_key = '' THEN NULL ELSE cv.section_kind END AS section_kind,
                CASE WHEN cv.section_key = '' THEN NULL ELSE cv.section_key END AS section_key,
                NULL::text AS section_title
            FROM content_vectors cv
            JOIN documents d ON d.id = cv.document_id AND d.user_id = $1
            LEFT JOIN library_entries le
                ON le.document_id = d.id AND le.user_id = $1 AND le.deleted_at IS NULL
            WHERE cv.user_id = $1
              AND cv.embedding_model = $7
              AND cv.embedding_dim = $8
              AND ($3::text IS NULL OR cv.section_kind = $3)
              AND EXISTS (
                  SELECT 1
                  FROM collection_entries ce
                  JOIN collection_scope cs ON cs.id = ce.collection_id
                  JOIN library_entries le2
                    ON le2.id = ce.library_entry_id AND le2.deleted_at IS NULL
                  WHERE le2.document_id = cv.document_id AND le2.user_id = $1
              )
            ORDER BY
                cv.embedding <=> ($2::text)::vector ASC,
                COALESCE(le.saved_at, d.created_at) DESC,
                cv.document_id DESC,
                COALESCE(cv.section_key, '') DESC,
                cv.chunk_index ASC
            LIMIT $4
            "#,
            query.user_id.into_uuid(),
            &embedding_literal,
            section_kind,
            query.limit,
            query.collection_id.into_uuid(),
            query.include_descendants,
            &query.embedding_model,
            query.embedding_dim,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(SearchHit::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
