use ind_application::AppError;
use ind_application::repos::library::{LibraryItemTypeCount, LibraryScopeCounts};
use ind_application::repos::{Cursor, Page};
use ind_domain::{
    DocumentId, DomainError, LibraryEntry, LibraryEntryId, LibraryEntryWithDocument, TriageState,
    UserId,
};

use super::PgLibraryRepository;
use super::rows::{LibraryEntryRow, LibraryWithDocRow, map_library_error, parse_document_type};
use crate::cursor::{clamp_limit, decode_cursor_ts, encode_cursor_ts};
use crate::repos::library_query::{LibraryListFilter, query_library_entries_page};

impl PgLibraryRepository {
    pub(super) async fn insert_entry_impl(
        &self,
        entry: LibraryEntry,
    ) -> Result<LibraryEntry, AppError> {
        // Provenance must be truthful: the delivery must belong to this user and already be
        // linked to the same document the entry saves. The source_delivery_id FK only proves
        // the row exists, so guard ownership and the document link explicitly. (document_id
        // ownership for the entry itself is enforced by the composite FK to documents.)
        if let Some(delivery_id) = entry.source_delivery_id {
            let valid = sqlx::query_scalar!(
                "SELECT EXISTS ( \
                     SELECT 1 FROM feed_deliveries \
                     WHERE id = $1 AND user_id = $2 AND document_id = $3 \
                 )",
                delivery_id.into_uuid(),
                entry.user_id.into_uuid(),
                entry.document_id.into_uuid(),
            )
            .fetch_one(&self.pool)
            .await
            .map_err(map_library_error)?;

            if valid != Some(true) {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "source_delivery_id".into(),
                    message: "source delivery must belong to the owner and be linked to the \
                              saved document"
                        .into(),
                }));
            }
        }

        let row = sqlx::query_as!(
            LibraryEntryRow,
            "INSERT INTO library_entries \
                (id, user_id, document_id, saved_at, triage_state, is_favorite, is_shortlisted, \
                 deleted_at, source, source_delivery_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             RETURNING id, user_id, document_id, saved_at, triage_state, is_favorite, \
                       is_shortlisted, deleted_at, source, source_delivery_id, created_at, \
                       updated_at",
            entry.id.into_uuid(),
            entry.user_id.into_uuid(),
            entry.document_id.into_uuid(),
            entry.saved_at,
            entry.triage_state.as_str(),
            entry.is_favorite,
            entry.is_shortlisted,
            entry.deleted_at,
            entry.source.as_str(),
            entry.source_delivery_id.map(|id| id.into_uuid()),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_library_error)?;

        row.into_entry()
    }

    pub(super) async fn find_active_by_document_impl(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<LibraryEntry>, AppError> {
        let row = sqlx::query_as!(
            LibraryEntryRow,
            "SELECT id, user_id, document_id, saved_at, triage_state, is_favorite, is_shortlisted, \
                    deleted_at, source, source_delivery_id, created_at, updated_at \
             FROM library_entries \
             WHERE user_id = $1 AND document_id = $2 AND deleted_at IS NULL",
            user_id.into_uuid(),
            document_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_library_error)?;

        row.map(LibraryEntryRow::into_entry).transpose()
    }

    pub(super) async fn find_by_id_impl(
        &self,
        id: LibraryEntryId,
        user_id: UserId,
    ) -> Result<Option<LibraryEntryWithDocument>, AppError> {
        let row = sqlx::query_as!(
            LibraryWithDocRow,
            "SELECT le.id, le.user_id, le.document_id, le.saved_at, le.triage_state, \
                    le.is_favorite, le.is_shortlisted, le.deleted_at, le.source, \
                    le.source_delivery_id, le.created_at, le.updated_at, \
                    d.document_type AS doc_document_type, d.canonical_url AS doc_canonical_url, \
                    d.original_url AS doc_original_url, d.content_hash AS doc_content_hash, \
                    d.title AS doc_title, d.author AS doc_author, d.excerpt AS doc_excerpt, \
                    d.published_at AS doc_published_at, d.language AS doc_language, \
                    d.domain AS doc_domain, d.lead_image_url AS doc_lead_image_url, \
                    d.thumbnail_url AS doc_thumbnail_url, d.word_count AS doc_word_count, \
                    d.reading_time_minutes AS doc_reading_time_minutes, \
                    d.created_at AS doc_created_at, d.updated_at AS doc_updated_at, \
                    (SELECT aa.failed_reason FROM archive_assets aa \
                     WHERE aa.document_id = d.id AND aa.asset_kind = 'readable_html' \
                       AND aa.status = 'failed') AS \"ingest_failure_reason?\" \
             FROM library_entries le \
             JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id \
             WHERE le.id = $1 AND le.user_id = $2 AND le.deleted_at IS NULL",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_library_error)?;

        row.map(LibraryWithDocRow::into_with_document).transpose()
    }

    pub(super) async fn find_active_by_canonical_url_impl(
        &self,
        user_id: UserId,
        canonical_url: &str,
    ) -> Result<Option<LibraryEntryWithDocument>, AppError> {
        let row = sqlx::query_as!(
            LibraryWithDocRow,
            "SELECT le.id, le.user_id, le.document_id, le.saved_at, le.triage_state, \
                    le.is_favorite, le.is_shortlisted, le.deleted_at, le.source, \
                    le.source_delivery_id, le.created_at, le.updated_at, \
                    d.document_type AS doc_document_type, d.canonical_url AS doc_canonical_url, \
                    d.original_url AS doc_original_url, d.content_hash AS doc_content_hash, \
                    d.title AS doc_title, d.author AS doc_author, d.excerpt AS doc_excerpt, \
                    d.published_at AS doc_published_at, d.language AS doc_language, \
                    d.domain AS doc_domain, d.lead_image_url AS doc_lead_image_url, \
                    d.thumbnail_url AS doc_thumbnail_url, d.word_count AS doc_word_count, \
                    d.reading_time_minutes AS doc_reading_time_minutes, \
                    d.created_at AS doc_created_at, d.updated_at AS doc_updated_at, \
                    (SELECT aa.failed_reason FROM archive_assets aa \
                     WHERE aa.document_id = d.id AND aa.asset_kind = 'readable_html' \
                       AND aa.status = 'failed') AS \"ingest_failure_reason?\" \
             FROM library_entries le \
             JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id \
             WHERE le.user_id = $1 AND d.canonical_url = $2 AND le.deleted_at IS NULL \
             LIMIT 1",
            user_id.into_uuid(),
            canonical_url,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_library_error)?;

        row.map(LibraryWithDocRow::into_with_document).transpose()
    }

    pub(super) async fn list_trashed_impl(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        query_library_entries_page(
            &self.pool,
            user_id,
            &LibraryListFilter {
                filter_expression: None,
                trashed_only: true,
            },
            cursor,
            limit,
        )
        .await
    }

    pub(super) async fn scope_counts_impl(
        &self,
        user_id: UserId,
        triage: Option<TriageState>,
    ) -> Result<LibraryScopeCounts, AppError> {
        let triage = triage.map(|t| t.as_str());

        // Grouping by document type and aggregating the read-state buckets in the same pass
        // keeps both breakdowns on one round trip; the scope totals are the column sums.
        let rows = sqlx::query!(
            "SELECT d.document_type AS \"item_type!\", \
                    COUNT(*) FILTER (WHERE uds.finished_at IS NOT NULL) AS \"done!\", \
                    COUNT(*) FILTER ( \
                        WHERE uds.finished_at IS NULL \
                          AND COALESCE(uds.max_progress_percent, 0) > 0 \
                    ) AS \"reading!\", \
                    COUNT(*) FILTER ( \
                        WHERE uds.finished_at IS NULL \
                          AND COALESCE(uds.max_progress_percent, 0) = 0 \
                    ) AS \"unread!\" \
             FROM library_entries le \
             JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id \
             LEFT JOIN user_document_state uds \
                    ON uds.user_id = le.user_id AND uds.document_id = le.document_id \
             WHERE le.user_id = $1 AND le.deleted_at IS NULL \
               AND ($2::text IS NULL OR le.triage_state = $2) \
             GROUP BY d.document_type \
             ORDER BY d.document_type",
            user_id.into_uuid(),
            triage,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_library_error)?;

        let mut counts = LibraryScopeCounts::default();
        for row in rows {
            counts.unread += row.unread;
            counts.reading += row.reading;
            counts.done += row.done;
            counts.by_item_type.push(LibraryItemTypeCount {
                item_type: parse_document_type(&row.item_type)?,
                count: row.unread + row.reading + row.done,
            });
        }

        Ok(counts)
    }

    pub(super) async fn list_by_user_impl(
        &self,
        user_id: UserId,
        triage: Option<TriageState>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;
        let triage = triage.map(|t| t.as_str());

        let rows = if let Some(ref cursor) = cursor {
            let (ts, id) = decode_cursor_ts(cursor)?;
            sqlx::query_as!(
                LibraryWithDocRow,
                "SELECT le.id, le.user_id, le.document_id, le.saved_at, le.triage_state, \
                        le.is_favorite, le.is_shortlisted, le.deleted_at, le.source, \
                        le.source_delivery_id, le.created_at, le.updated_at, \
                        d.document_type AS doc_document_type, d.canonical_url AS doc_canonical_url, \
                        d.original_url AS doc_original_url, d.content_hash AS doc_content_hash, \
                        d.title AS doc_title, d.author AS doc_author, d.excerpt AS doc_excerpt, \
                        d.published_at AS doc_published_at, d.language AS doc_language, \
                        d.domain AS doc_domain, d.lead_image_url AS doc_lead_image_url, \
                        d.thumbnail_url AS doc_thumbnail_url, d.word_count AS doc_word_count, \
                        d.reading_time_minutes AS doc_reading_time_minutes, \
                        d.created_at AS doc_created_at, d.updated_at AS doc_updated_at, \
                    (SELECT aa.failed_reason FROM archive_assets aa \
                     WHERE aa.document_id = d.id AND aa.asset_kind = 'readable_html' \
                       AND aa.status = 'failed') AS \"ingest_failure_reason?\" \
                 FROM library_entries le \
                 JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id \
                 WHERE le.user_id = $1 AND le.deleted_at IS NULL \
                   AND ($2::text IS NULL OR le.triage_state = $2) \
                   AND (le.saved_at, le.id) < ($3, $4) \
                 ORDER BY le.saved_at DESC, le.id DESC \
                 LIMIT $5",
                user_id.into_uuid(),
                triage,
                ts,
                id,
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_library_error)?
        } else {
            sqlx::query_as!(
                LibraryWithDocRow,
                "SELECT le.id, le.user_id, le.document_id, le.saved_at, le.triage_state, \
                        le.is_favorite, le.is_shortlisted, le.deleted_at, le.source, \
                        le.source_delivery_id, le.created_at, le.updated_at, \
                        d.document_type AS doc_document_type, d.canonical_url AS doc_canonical_url, \
                        d.original_url AS doc_original_url, d.content_hash AS doc_content_hash, \
                        d.title AS doc_title, d.author AS doc_author, d.excerpt AS doc_excerpt, \
                        d.published_at AS doc_published_at, d.language AS doc_language, \
                        d.domain AS doc_domain, d.lead_image_url AS doc_lead_image_url, \
                        d.thumbnail_url AS doc_thumbnail_url, d.word_count AS doc_word_count, \
                        d.reading_time_minutes AS doc_reading_time_minutes, \
                        d.created_at AS doc_created_at, d.updated_at AS doc_updated_at, \
                    (SELECT aa.failed_reason FROM archive_assets aa \
                     WHERE aa.document_id = d.id AND aa.asset_kind = 'readable_html' \
                       AND aa.status = 'failed') AS \"ingest_failure_reason?\" \
                 FROM library_entries le \
                 JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id \
                 WHERE le.user_id = $1 AND le.deleted_at IS NULL \
                   AND ($2::text IS NULL OR le.triage_state = $2) \
                 ORDER BY le.saved_at DESC, le.id DESC \
                 LIMIT $3",
                user_id.into_uuid(),
                triage,
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_library_error)?
        };

        let has_more = rows.len() as i64 > limit;
        let items: Vec<LibraryEntryWithDocument> = rows
            .into_iter()
            .take(limit as usize)
            .map(LibraryWithDocRow::into_with_document)
            .collect::<Result<_, _>>()?;
        let next_cursor = if has_more {
            items
                .last()
                .map(|e| encode_cursor_ts(e.entry.saved_at, e.entry.id.into_uuid()))
        } else {
            None
        };

        Ok(Page { items, next_cursor })
    }
}
