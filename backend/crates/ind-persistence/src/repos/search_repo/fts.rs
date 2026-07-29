use ind_domain::CanonicalAddress;

use super::types::{SearchHitRow, map_sqlx_error};
use super::*;

fn canonicalize_sender_filter(value: &str) -> String {
    if value.contains('@') {
        CanonicalAddress::new(value).to_string()
    } else {
        value.trim().to_lowercase()
    }
}

impl PgSearchRepository {
    // Durable search is keyed by `documents`; feed discovery is a live query over unprepared
    // feed_deliveries + feed_source_entries. Authored capabilities never search legacy item/feed
    // rows in the document-era model.
    pub(super) async fn search_fts_impl(
        &self,
        query: &SearchFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        let normalized_senders: Vec<String> = query
            .sender_values
            .iter()
            .map(|v| canonicalize_sender_filter(v))
            .collect();
        let normalized_negated_senders: Vec<String> = query
            .negated_sender_values
            .iter()
            .map(|v| canonicalize_sender_filter(v))
            .collect();
        let rows = sqlx::query_as!(
            SearchHitRow,
            r#"
            WITH documents_matched AS (
                SELECT
                    sd.document_id AS result_id,
                    sd.document_id AS document_id,
                    NULL::uuid AS delivery_id,
                    NULL::uuid AS source_entry_id,
                    'document'::text AS result_kind,
                    d.title AS item_title,
                    CASE
                        WHEN $2::text IS NULL THEN
                            substr(
                                trim(
                                    COALESCE(
                                        NULLIF(sd.highlight_text, ''),
                                        NULLIF(sd.body_text, ''),
                                        NULLIF(sd.metadata_text, ''),
                                        NULLIF(sd.section_title, ''),
                                        NULLIF(sd.title, ''),
                                        d.title
                                    )
                                ),
                                1,
                                240
                            )
                        ELSE
                            ts_headline(
                                sd.search_config,
                                COALESCE(
                                    NULLIF(sd.highlight_text, ''),
                                    NULLIF(sd.body_text, ''),
                                    NULLIF(sd.metadata_text, ''),
                                    NULLIF(sd.section_title, ''),
                                    NULLIF(sd.title, ''),
                                    d.title
                                ),
                                websearch_to_tsquery('english'::regconfig, $2) || websearch_to_tsquery('simple'::regconfig, $2),
                                'StartSel=<mark>,StopSel=</mark>,MaxFragments=2,MinWords=4,MaxWords=18'
                            )
                    END AS snippet,
                    (
                        (
                            CASE
                                WHEN $2::text IS NULL THEN 0::double precision
                                ELSE ts_rank_cd(
                                    sd.document_tsv,
                                    websearch_to_tsquery('english'::regconfig, $2) || websearch_to_tsquery('simple'::regconfig, $2),
                                    32
                                )::double precision
                            END
                        ) * 0.85
                    ) + (
                        (
                            1.0 / (
                                1.0 + GREATEST(
                                    0.0,
                                    EXTRACT(EPOCH FROM ($50::timestamptz - COALESCE(le.saved_at, d.created_at))) / 86400.0
                                ) * 0.01
                            )
                        ) * 0.15
                    ) AS final_score,
                    d.document_type AS item_type,
                    COALESCE(d.canonical_url, d.original_url) AS url,
                    COALESCE(le.saved_at, d.created_at) AS saved_at,
                    sd.updated_at,
                    CASE WHEN sd.section_key = '' THEN NULL ELSE sd.document_kind END AS section_kind,
                    CASE WHEN sd.section_key = '' THEN NULL ELSE sd.section_key END AS section_key,
                    CASE WHEN sd.section_key = '' THEN NULL ELSE sd.section_title END AS section_title,
                    d.sender_id AS sender_id
                FROM search_documents sd
                JOIN documents d ON d.id = sd.document_id AND d.user_id = $1
                LEFT JOIN library_entries le
                    ON le.document_id = d.id AND le.user_id = $1 AND le.deleted_at IS NULL
                LEFT JOIN user_document_state uds
                    ON uds.document_id = d.id AND uds.user_id = $1
                WHERE sd.user_id = $1
                  AND sd.document_id IS NOT NULL
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
                  AND ($2::text IS NOT NULL OR sd.section_key = '')
                  AND (
                        $2::text IS NULL
                        OR sd.document_tsv @@ (websearch_to_tsquery('english'::regconfig, $2) || websearch_to_tsquery('simple'::regconfig, $2))
                  )
                  AND (
                        cardinality($3::text[]) = 0
                        OR EXISTS (
                            SELECT 1
                            FROM library_entry_tags letag
                            JOIN tags t ON t.id = letag.tag_id
                            LEFT JOIN tag_aliases ta ON ta.tag_id = t.id
                            WHERE letag.library_entry_id = le.id
                              AND (
                                    lower(t.name) = ANY($3::text[])
                                    OR lower(ta.alias) = ANY($3::text[])
                              )
                        )
                  )
                  AND (
                        cardinality($4::text[]) = 0
                        OR NOT EXISTS (
                            SELECT 1
                            FROM library_entry_tags letag
                            JOIN tags t ON t.id = letag.tag_id
                            LEFT JOIN tag_aliases ta ON ta.tag_id = t.id
                            WHERE letag.library_entry_id = le.id
                              AND (
                                    lower(t.name) = ANY($4::text[])
                                    OR lower(ta.alias) = ANY($4::text[])
                              )
                        )
                  )
                  AND (
                        cardinality($5::text[]) = 0
                        OR EXISTS (
                            SELECT 1
                            FROM collection_entries ce
                            JOIN collections c ON c.id = ce.collection_id
                            WHERE ce.library_entry_id = le.id
                              AND (
                                    lower(c.name) = ANY($5::text[])
                                    OR regexp_replace(
                                        regexp_replace(lower(c.name), '[^a-z0-9]+', '-', 'g'),
                                        '(^-|-$)',
                                        '',
                                        'g'
                                    ) = ANY($5::text[])
                              )
                        )
                  )
                  AND (
                        cardinality($6::text[]) = 0
                        OR NOT EXISTS (
                            SELECT 1
                            FROM collection_entries ce
                            JOIN collections c ON c.id = ce.collection_id
                            WHERE ce.library_entry_id = le.id
                              AND (
                                    lower(c.name) = ANY($6::text[])
                                    OR regexp_replace(
                                        regexp_replace(lower(c.name), '[^a-z0-9]+', '-', 'g'),
                                        '(^-|-$)',
                                        '',
                                        'g'
                                    ) = ANY($6::text[])
                              )
                        )
                  )
                  AND (cardinality($7::text[]) = 0 OR lower(d.document_type) = ANY($7::text[]))
                  AND (cardinality($8::text[]) = 0 OR lower(d.document_type) <> ALL($8::text[]))
                  AND (
                        cardinality($9::text[]) = 0
                        OR EXISTS (
                            SELECT 1 FROM unnest($9::text[]) AS needle(value)
                            WHERE lower(COALESCE(d.author, '')) LIKE '%' || needle.value || '%'
                        )
                  )
                  AND (
                        cardinality($10::text[]) = 0
                        OR NOT EXISTS (
                            SELECT 1 FROM unnest($10::text[]) AS needle(value)
                            WHERE lower(COALESCE(d.author, '')) LIKE '%' || needle.value || '%'
                        )
                  )
                  AND (
                        cardinality($11::text[]) = 0
                        OR EXISTS (
                            SELECT 1 FROM unnest($11::text[]) AS needle(value)
                            WHERE lower(COALESCE(d.canonical_url, '')) LIKE '%' || needle.value || '%'
                               OR lower(COALESCE(d.original_url, '')) LIKE '%' || needle.value || '%'
                               OR lower(COALESCE(d.domain, '')) LIKE '%' || needle.value || '%'
                        )
                  )
                  AND (
                        cardinality($12::text[]) = 0
                        OR NOT EXISTS (
                            SELECT 1 FROM unnest($12::text[]) AS needle(value)
                            WHERE lower(COALESCE(d.canonical_url, '')) LIKE '%' || needle.value || '%'
                               OR lower(COALESCE(d.original_url, '')) LIKE '%' || needle.value || '%'
                               OR lower(COALESCE(d.domain, '')) LIKE '%' || needle.value || '%'
                        )
                  )
                  AND (
                        cardinality($13::text[]) = 0
                        OR EXISTS (
                            SELECT 1
                            FROM entity_mentions em
                            JOIN entities e ON e.id = em.entity_id
                            WHERE em.document_id = d.id
                              AND e.user_id = $1
                              AND lower(e.name) = ANY($13::text[])
                        )
                  )
                  AND (
                        cardinality($14::text[]) = 0
                        OR NOT EXISTS (
                            SELECT 1
                            FROM entity_mentions em
                            JOIN entities e ON e.id = em.entity_id
                            WHERE em.document_id = d.id
                              AND e.user_id = $1
                              AND lower(e.name) = ANY($14::text[])
                        )
                  )
                  AND ($15::timestamptz IS NULL OR COALESCE(le.saved_at, d.created_at) < $15)
                  AND ($16::timestamptz IS NULL OR COALESCE(le.saved_at, d.created_at) >= $16)
                  AND (
                        NOT $17
                        OR COALESCE(uds.max_progress_percent, 0) >= 100
                        OR uds.finished_at IS NOT NULL
                  )
                  AND (
                        NOT $18
                        OR NOT (
                            COALESCE(uds.max_progress_percent, 0) >= 100
                            OR uds.finished_at IS NOT NULL
                        )
                  )
                  AND (NOT $19 OR uds.document_id IS NULL OR uds.last_read_at IS NULL)
                  AND (
                        NOT $20
                        OR NOT (uds.document_id IS NULL OR uds.last_read_at IS NULL)
                  )
                  AND (NOT $21 OR le.triage_state = 'archive')
                  AND (NOT $22 OR le.triage_state IS DISTINCT FROM 'archive')
                  AND (NOT $23 OR COALESCE(le.is_favorite, false))
                  AND (NOT $24 OR NOT COALESCE(le.is_favorite, false))
                  AND (
                        NOT $25
                        OR EXISTS (SELECT 1 FROM highlights h WHERE h.document_id = d.id)
                  )
                  AND (
                        NOT $26
                        OR NOT EXISTS (SELECT 1 FROM highlights h WHERE h.document_id = d.id)
                  )
                  AND (
                        NOT $27
                        OR EXISTS (SELECT 1 FROM item_notes n WHERE n.document_id = d.id)
                        OR EXISTS (
                            SELECT 1
                            FROM highlights h
                            JOIN highlight_notes hn ON hn.highlight_id = h.id
                            WHERE h.document_id = d.id
                        )
                  )
                  AND (
                        NOT $28
                        OR (
                            NOT EXISTS (SELECT 1 FROM item_notes n WHERE n.document_id = d.id)
                            AND NOT EXISTS (
                                SELECT 1
                                FROM highlights h
                                JOIN highlight_notes hn ON hn.highlight_id = h.id
                                WHERE h.document_id = d.id
                            )
                        )
                  )
                  AND (NOT $29 OR COALESCE(le.is_shortlisted, false))
                  AND (NOT $30 OR NOT COALESCE(le.is_shortlisted, false))
                  -- source:feed includes a document iff it has a linked delivery (prepared-but-in-feed);
                  -- source:library ($39) includes it iff it has an active library entry.
                  AND (
                        NOT $36
                        OR EXISTS (
                            SELECT 1 FROM feed_deliveries fd
                            WHERE fd.document_id = d.id
                              AND fd.user_id = $1
                              AND fd.hidden_at IS NULL
                        )
                  )
                  AND (NOT $39 OR le.id IS NOT NULL)
                  -- Email-sender filters resolve through documents.sender_id -> email_senders.
                  -- Require variants demand a matching linked sender; negated variants exclude
                  -- documents whose linked sender matches.
                  AND (
                        cardinality($37::text[]) = 0
                        OR EXISTS (
                            SELECT 1 FROM email_senders es
                            WHERE es.id = d.sender_id AND es.user_id = $1
                              AND lower(es.canonical_addr) = ANY($37::text[])
                        )
                  )
                  AND (
                        cardinality($38::text[]) = 0
                        OR NOT EXISTS (
                            SELECT 1 FROM email_senders es
                            WHERE es.id = d.sender_id AND es.user_id = $1
                              AND lower(es.canonical_addr) = ANY($38::text[])
                        )
                  )
                  AND (
                        cardinality($40::text[]) = 0
                        OR EXISTS (
                            SELECT 1 FROM email_senders es
                            WHERE es.id = d.sender_id AND es.user_id = $1
                              AND lower(split_part(es.canonical_addr, '@', 2)) = ANY($40::text[])
                        )
                  )
                  AND (
                        cardinality($41::text[]) = 0
                        OR NOT EXISTS (
                            SELECT 1 FROM email_senders es
                            WHERE es.id = d.sender_id AND es.user_id = $1
                              AND lower(split_part(es.canonical_addr, '@', 2)) = ANY($41::text[])
                        )
                  )
                  AND (
                        cardinality($42::text[]) = 0
                        OR EXISTS (
                            SELECT 1 FROM email_senders es
                            WHERE es.id = d.sender_id AND es.user_id = $1
                              AND lower(COALESCE(es.list_id, '')) = ANY($42::text[])
                        )
                  )
                  AND (
                        cardinality($43::text[]) = 0
                        OR NOT EXISTS (
                            SELECT 1 FROM email_senders es
                            WHERE es.id = d.sender_id AND es.user_id = $1
                              AND lower(COALESCE(es.list_id, '')) = ANY($43::text[])
                        )
                  )
                  AND (
                        cardinality($44::text[]) = 0
                        OR EXISTS (
                            SELECT 1 FROM unnest($44::text[]) AS needle(value)
                            WHERE lower(d.title) LIKE '%' || needle.value || '%'
                        )
                  )
                  AND (
                        cardinality($45::text[]) = 0
                        OR NOT EXISTS (
                            SELECT 1 FROM unnest($45::text[]) AS needle(value)
                            WHERE lower(d.title) LIKE '%' || needle.value || '%'
                        )
                  )
                  AND (
                        NOT $46
                        OR EXISTS (
                            SELECT 1 FROM email_unsubscribe_targets eut
                            WHERE eut.sender_id = d.sender_id
                        )
                  )
                  AND (
                        NOT $47
                        OR NOT EXISTS (
                            SELECT 1 FROM email_unsubscribe_targets eut
                            WHERE eut.sender_id = d.sender_id
                        )
                  )
                  AND (
                        NOT $48
                        OR EXISTS (
                            SELECT 1 FROM email_senders es
                            WHERE es.id = d.sender_id AND es.user_id = $1
                              AND es.blocked_at IS NOT NULL
                        )
                  )
                  AND (
                        NOT $49
                        OR NOT EXISTS (
                            SELECT 1 FROM email_senders es
                            WHERE es.id = d.sender_id AND es.user_id = $1
                              AND es.blocked_at IS NOT NULL
                        )
                  )
            ),
            feed_preview_matched AS (
                SELECT
                    fd.id AS result_id,
                    NULL::uuid AS document_id,
                    fd.id AS delivery_id,
                    fse.id AS source_entry_id,
                    'feed_preview'::text AS result_kind,
                    fse.title AS item_title,
                    CASE
                        WHEN $2::text IS NULL THEN
                            substr(
                                trim(
                                    COALESCE(
                                        NULLIF(fse.excerpt, ''),
                                        NULLIF(fse.content_html, ''),
                                        fse.title
                                    )
                                ),
                                1,
                                240
                            )
                        ELSE
                            ts_headline(
                                public.fts_config_for_language(fse.language),
                                COALESCE(
                                    NULLIF(fse.excerpt, ''),
                                    NULLIF(fse.content_html, ''),
                                    fse.title
                                ),
                                websearch_to_tsquery('english'::regconfig, $2) || websearch_to_tsquery('simple'::regconfig, $2),
                                'StartSel=<mark>,StopSel=</mark>,MaxFragments=2,MinWords=4,MaxWords=18'
                            )
                    END AS snippet,
                    (
                        (
                            CASE
                                WHEN $2::text IS NULL THEN 0::double precision
                                ELSE ts_rank(
                                    fse.search_tsv,
                                    websearch_to_tsquery('english'::regconfig, $2) || websearch_to_tsquery('simple'::regconfig, $2)
                                )::double precision
                            END
                        ) * 0.85
                    ) + (
                        (
                            1.0 / (
                                1.0 + GREATEST(
                                    0.0,
                                    EXTRACT(EPOCH FROM ($50::timestamptz - fd.delivered_at)) / 86400.0
                                ) * 0.01
                            )
                        ) * 0.15
                    ) AS final_score,
                    CASE fs.feed_type
                        WHEN 'youtube' THEN 'video'
                        WHEN 'podcast' THEN 'podcast'
                        WHEN 'twitter' THEN 'tweet'
                        ELSE 'article'
                    END AS item_type,
                    fse.url AS url,
                    fd.delivered_at AS saved_at,
                    fd.updated_at,
                    NULL::text AS section_kind,
                    NULL::text AS section_key,
                    NULL::text AS section_title,
                    NULL::uuid AS sender_id
                FROM feed_deliveries fd
                JOIN feed_source_entries fse ON fse.id = fd.source_entry_id
                JOIN feed_sources fs ON fs.id = fd.source_id
                WHERE fd.user_id = $1
                  AND fd.hidden_at IS NULL
                  -- Search previews are pre-materialization only. Once a delivery links to a
                  -- document, the document branch above owns recall and lifecycle filters.
                  AND fd.document_id IS NULL
                  AND (
                        $2::text IS NULL
                        OR fse.search_tsv @@ (websearch_to_tsquery('english'::regconfig, $2) || websearch_to_tsquery('simple'::regconfig, $2))
                  )
                  -- A preview row cannot satisfy tag/collection/entity/status/has/pinned/sender
                  -- require filters because it has not been prepared into a document.
                  AND cardinality($3::text[]) = 0
                  AND cardinality($4::text[]) >= 0
                  AND cardinality($5::text[]) = 0
                  AND cardinality($6::text[]) >= 0
                  AND cardinality($13::text[]) = 0
                  AND cardinality($14::text[]) >= 0
                  AND (
                        cardinality($7::text[]) = 0
                        OR CASE fs.feed_type
                            WHEN 'youtube' THEN 'video'
                            WHEN 'podcast' THEN 'podcast'
                            WHEN 'twitter' THEN 'tweet'
                            ELSE 'article'
                        END = ANY($7::text[])
                  )
                  AND (
                        cardinality($8::text[]) = 0
                        OR CASE fs.feed_type
                            WHEN 'youtube' THEN 'video'
                            WHEN 'podcast' THEN 'podcast'
                            WHEN 'twitter' THEN 'tweet'
                            ELSE 'article'
                        END <> ALL($8::text[])
                  )
                  AND (
                        cardinality($9::text[]) = 0
                        OR EXISTS (
                            SELECT 1 FROM unnest($9::text[]) AS needle(value)
                            WHERE lower(COALESCE(fse.author, '')) LIKE '%' || needle.value || '%'
                        )
                  )
                  AND (
                        cardinality($10::text[]) = 0
                        OR NOT EXISTS (
                            SELECT 1 FROM unnest($10::text[]) AS needle(value)
                            WHERE lower(COALESCE(fse.author, '')) LIKE '%' || needle.value || '%'
                        )
                  )
                  AND (
                        cardinality($11::text[]) = 0
                        OR EXISTS (
                            SELECT 1 FROM unnest($11::text[]) AS needle(value)
                            WHERE lower(COALESCE(fse.url, '')) LIKE '%' || needle.value || '%'
                        )
                  )
                  AND (
                        cardinality($12::text[]) = 0
                        OR NOT EXISTS (
                            SELECT 1 FROM unnest($12::text[]) AS needle(value)
                            WHERE lower(COALESCE(fse.url, '')) LIKE '%' || needle.value || '%'
                        )
                  )
                  AND ($15::timestamptz IS NULL OR fd.delivered_at < $15)
                  AND ($16::timestamptz IS NULL OR fd.delivered_at >= $16)
                  AND NOT $17 AND NOT $19 AND NOT $21 AND NOT $23
                  AND NOT $25 AND NOT $27 AND NOT $29
                  AND (
                        cardinality($44::text[]) = 0
                        OR EXISTS (
                            SELECT 1 FROM unnest($44::text[]) AS needle(value)
                            WHERE lower(COALESCE(fse.title, '')) LIKE '%' || needle.value || '%'
                        )
                  )
                  AND (
                        cardinality($45::text[]) = 0
                        OR NOT EXISTS (
                            SELECT 1 FROM unnest($45::text[]) AS needle(value)
                            WHERE lower(COALESCE(fse.title, '')) LIKE '%' || needle.value || '%'
                        )
                  )
                  AND cardinality($37::text[]) = 0
                  AND cardinality($38::text[]) >= 0
                  AND cardinality($40::text[]) = 0
                  AND cardinality($41::text[]) >= 0
                  AND cardinality($42::text[]) = 0
                  AND cardinality($43::text[]) >= 0
                  AND NOT $46
                  AND (NOT $47 OR TRUE)
                  AND NOT $48
                  AND (NOT $49 OR TRUE)
                  -- source:library ($39) excludes previews (they are never in Library).
                  AND NOT $39
            ),
            matched AS (
                SELECT * FROM documents_matched
                UNION ALL
                SELECT * FROM feed_preview_matched
            ),
            ranked AS (
                SELECT *
                FROM matched
                WHERE (
                    $31::double precision IS NULL
                    OR final_score < $31
                    OR (final_score = $31 AND saved_at < $32)
                    OR (final_score = $31 AND saved_at = $32 AND result_id < $33)
                    OR (
                        final_score = $31
                        AND saved_at = $32
                        AND result_id = $33
                        AND COALESCE(section_key, '') < COALESCE($34, '')
                    )
                )
            )
            SELECT
                document_id AS "document_id?",
                delivery_id AS "delivery_id?",
                source_entry_id AS "source_entry_id?",
                result_kind AS "result_kind!",
                item_title AS "item_title!",
                snippet AS "snippet!",
                final_score AS "final_score!",
                item_type AS "item_type!",
                url AS "url?",
                saved_at AS "saved_at!",
                updated_at AS "updated_at!",
                section_kind AS "section_kind?",
                section_key AS "section_key?",
                section_title AS "section_title?",
                sender_id AS "sender_id?"
            FROM ranked
            ORDER BY final_score DESC, saved_at DESC, result_id DESC, COALESCE(section_key, '') DESC
            LIMIT $35
            "#,
            query.user_id.into_uuid(),
            query.text_query.as_deref(),
            &query.tag_values,
            &query.negated_tag_values,
            &query.collection_values,
            &query.negated_collection_values,
            &query.type_values,
            &query.negated_type_values,
            &query.author_values,
            &query.negated_author_values,
            &query.url_values,
            &query.negated_url_values,
            &query.entity_values,
            &query.negated_entity_values,
            query.before_saved_at,
            query.after_saved_at,
            query.require_read,
            query.exclude_read,
            query.require_unread,
            query.exclude_unread,
            query.require_archived,
            query.exclude_archived,
            query.require_favorited,
            query.exclude_favorited,
            query.require_has_highlights,
            query.exclude_has_highlights,
            query.require_has_notes,
            query.exclude_has_notes,
            query.require_pinned,
            query.exclude_pinned,
            query.cursor_score,
            query.cursor_saved_at,
            query.cursor_result_id,
            query.cursor_section_key.as_deref(),
            query.limit,
            query.require_feed_only,
            &normalized_senders,
            &normalized_negated_senders,
            query.exclude_feed_only,
            &query.sender_domain_values,
            &query.negated_sender_domain_values,
            &query.list_values,
            &query.negated_list_values,
            &query.subject_values,
            &query.negated_subject_values,
            query.require_has_unsubscribe,
            query.exclude_has_unsubscribe,
            query.require_sender_blocked,
            query.exclude_sender_blocked,
            query.score_reference_at,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(SearchHit::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
