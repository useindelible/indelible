use ind_application::AppError;
use ind_application::repos::content_vector::CollectionDocumentFtsQuery;
use ind_domain::SearchHit;

use super::super::PgContentVectorRepository;
use super::super::types::{FtsHitRow, map_sqlx_error};

impl PgContentVectorRepository {
    pub(in crate::repos::content_vector_repo) async fn fts_collection_document_impl(
        &self,
        query: &CollectionDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        let rows = sqlx::query_as!(
            FtsHitRow,
            r#"
            WITH RECURSIVE collection_scope(id) AS (
                SELECT c.id
                FROM collections c
                WHERE c.id = $3
                  AND c.user_id = $1
                UNION ALL
                SELECT child.id
                FROM collections child
                JOIN collection_scope parent ON parent.id = child.parent_id
                WHERE child.user_id = $1
                  AND $5::boolean
            ),
            matches AS (
                SELECT
                    cv.id AS chunk_id,
                    cv.document_id,
                    cv.content AS snippet,
                    (
                        ts_rank_cd(cv.content_tsv, fts_relaxed_query('english'::regconfig, $2), 32)::double precision
                        + CASE
                            WHEN cv.content_tsv @@ plainto_tsquery('english'::regconfig, $2)
                                THEN 0.25::double precision
                            ELSE 0.0::double precision
                          END
                    ) AS fts_rank,
                    CASE WHEN cv.section_key = '' THEN NULL ELSE cv.section_kind END AS section_kind,
                    CASE WHEN cv.section_key = '' THEN NULL ELSE cv.section_key END AS section_key,
                    NULL::text AS section_title
                FROM content_vectors cv
                WHERE cv.user_id = $1
                  AND cv.search_config = 'english'::regconfig
                  AND cv.content_tsv @@ fts_relaxed_query('english'::regconfig, $2)

                UNION ALL

                SELECT
                    cv.id AS chunk_id,
                    cv.document_id,
                    cv.content AS snippet,
                    (
                        ts_rank_cd(cv.content_tsv, fts_relaxed_query('simple'::regconfig, $2), 32)::double precision
                        + CASE
                            WHEN cv.content_tsv @@ plainto_tsquery('simple'::regconfig, $2)
                                THEN 0.25::double precision
                            ELSE 0.0::double precision
                          END
                    ) AS fts_rank,
                    CASE WHEN cv.section_key = '' THEN NULL ELSE cv.section_kind END AS section_kind,
                    CASE WHEN cv.section_key = '' THEN NULL ELSE cv.section_key END AS section_key,
                    NULL::text AS section_title
                FROM content_vectors cv
                WHERE cv.user_id = $1
                  AND cv.search_config = 'simple'::regconfig
                  AND cv.content_tsv @@ fts_relaxed_query('simple'::regconfig, $2)

                UNION ALL

                SELECT
                    NULL::uuid AS chunk_id,
                    sd.document_id,
                    ts_headline(
                        'english'::regconfig,
                        concat_ws(
                            E'\n',
                            NULLIF(sd.title, ''),
                            NULLIF(sd.section_title, ''),
                            NULLIF(sd.highlight_text, ''),
                            NULLIF(sd.metadata_text, ''),
                            NULLIF(sd.body_text, '')
                        ),
                        fts_relaxed_query('english'::regconfig, $2),
                        'StartSel=<<,StopSel=>>,MaxFragments=1,MinWords=8,MaxWords=40'
                    ) AS snippet,
                    (
                        ts_rank_cd(sd.document_tsv, fts_relaxed_query('english'::regconfig, $2), 32)::double precision
                        + CASE
                            WHEN sd.document_tsv @@ plainto_tsquery('english'::regconfig, $2)
                                THEN 0.25::double precision
                            ELSE 0.0::double precision
                          END
                    ) AS fts_rank,
                    CASE WHEN sd.section_key = '' THEN NULL ELSE sd.document_kind END AS section_kind,
                    CASE WHEN sd.section_key = '' THEN NULL ELSE sd.section_key END AS section_key,
                    CASE WHEN sd.section_key = '' THEN NULL ELSE sd.section_title END AS section_title
                FROM search_documents sd
                WHERE sd.user_id = $1
                  AND sd.search_config = 'english'::regconfig
                  AND sd.document_tsv @@ fts_relaxed_query('english'::regconfig, $2)
                  AND NOT EXISTS (
                      SELECT 1
                      FROM content_vectors existing
                      WHERE existing.user_id = $1
                        AND existing.document_id = sd.document_id
                  )

                UNION ALL

                SELECT
                    NULL::uuid AS chunk_id,
                    sd.document_id,
                    ts_headline(
                        'simple'::regconfig,
                        concat_ws(
                            E'\n',
                            NULLIF(sd.title, ''),
                            NULLIF(sd.section_title, ''),
                            NULLIF(sd.highlight_text, ''),
                            NULLIF(sd.metadata_text, ''),
                            NULLIF(sd.body_text, '')
                        ),
                        fts_relaxed_query('simple'::regconfig, $2),
                        'StartSel=<<,StopSel=>>,MaxFragments=1,MinWords=8,MaxWords=40'
                    ) AS snippet,
                    (
                        ts_rank_cd(sd.document_tsv, fts_relaxed_query('simple'::regconfig, $2), 32)::double precision
                        + CASE
                            WHEN sd.document_tsv @@ plainto_tsquery('simple'::regconfig, $2)
                                THEN 0.25::double precision
                            ELSE 0.0::double precision
                          END
                    ) AS fts_rank,
                    CASE WHEN sd.section_key = '' THEN NULL ELSE sd.document_kind END AS section_kind,
                    CASE WHEN sd.section_key = '' THEN NULL ELSE sd.section_key END AS section_key,
                    CASE WHEN sd.section_key = '' THEN NULL ELSE sd.section_title END AS section_title
                FROM search_documents sd
                WHERE sd.user_id = $1
                  AND sd.search_config = 'simple'::regconfig
                  AND sd.document_tsv @@ fts_relaxed_query('simple'::regconfig, $2)
                  AND NOT EXISTS (
                      SELECT 1
                      FROM content_vectors existing
                      WHERE existing.user_id = $1
                        AND existing.document_id = sd.document_id
                  )
            )
            SELECT
                COALESCE(matches.chunk_id, matches.document_id) AS "chunk_id!",
                (matches.chunk_id IS NULL) AS "coarse_fallback!",
                matches.document_id AS "document_id!",
                d.title AS item_title,
                matches.snippet AS "snippet!",
                matches.fts_rank AS "fts_rank!",
                d.document_type AS item_type,
                COALESCE(d.canonical_url, d.original_url) AS url,
                COALESCE(le.saved_at, d.created_at) AS "saved_at!",
                d.updated_at,
                matches.section_kind,
                matches.section_key,
                matches.section_title
            FROM matches
            JOIN documents d ON d.id = matches.document_id AND d.user_id = $1
            LEFT JOIN library_entries le
                ON le.document_id = d.id AND le.user_id = $1 AND le.deleted_at IS NULL
            WHERE EXISTS (
                SELECT 1
                FROM collection_entries ce
                JOIN collection_scope cs ON cs.id = ce.collection_id
                JOIN library_entries le2
                  ON le2.id = ce.library_entry_id AND le2.deleted_at IS NULL
                WHERE le2.document_id = matches.document_id
                  AND le2.user_id = $1
            )
            ORDER BY matches.fts_rank DESC, COALESCE(le.saved_at, d.created_at) DESC
            LIMIT $4
            "#,
            query.user_id.into_uuid(),
            &query.text_query,
            query.collection_id.into_uuid(),
            query.limit,
            query.include_descendants,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(SearchHit::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
