use chrono::{DateTime, Utc};
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::integration_connection::{
    NotionExportCandidate, NotionExportCursor, NotionExportItemsPage,
};
use ind_domain::{
    ContentSource, DocumentId, DocumentType, DomainError, IntegrationConnectionId, LibraryEntryId,
    NotionExportItem, UserId,
};

use super::{PgIntegrationConnectionRepository, escape_like_pattern, map_err};

// Flat row struct used with `sqlx::query_as!` for `list_notion_export_items`.
// Field ORDER must match the SELECT ORDER in the query; the macro binds positionally.
struct NotionExportItemFlat {
    library_entry_id: Uuid,
    document_id: Uuid,
    title: String,
    url: Option<String>,
    document_type: String,
    source: String,
    selected: bool,
    remote_page_id: Option<String>,
    export_last_synced_at: Option<DateTime<Utc>>,
    export_last_error: Option<String>,
}

fn parse_enum_err(field: &'static str, message: String) -> AppError {
    AppError::Domain(DomainError::Validation {
        field: field.to_string(),
        message,
    })
}

impl TryFrom<NotionExportItemFlat> for NotionExportItem {
    type Error = AppError;

    fn try_from(row: NotionExportItemFlat) -> Result<Self, Self::Error> {
        let document_type = row
            .document_type
            .parse::<DocumentType>()
            .map_err(|e| parse_enum_err("document_type", e))?;
        let source = row
            .source
            .parse::<ContentSource>()
            .map_err(|e| parse_enum_err("source", e))?;
        Ok(NotionExportItem {
            library_entry_id: LibraryEntryId::from_uuid(row.library_entry_id),
            document_id: DocumentId::from_uuid(row.document_id),
            title: row.title,
            url: row.url,
            document_type,
            source,
            selected: row.selected,
            exported_page_id: row.remote_page_id,
            last_synced_at: row.export_last_synced_at,
            last_error: row.export_last_error,
        })
    }
}

impl PgIntegrationConnectionRepository {
    pub(super) async fn list_notion_export_items_impl(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        query: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<NotionExportItemsPage, AppError> {
        let search = query
            .as_deref()
            .map(str::trim)
            .filter(|q| !q.is_empty())
            .map(|q| format!("%{}%", escape_like_pattern(q)));
        let limit = limit.clamp(1, 200);
        let offset = offset.max(0);
        let rows = sqlx::query_as!(
            NotionExportItemFlat,
            // TASK-236: only saved Library content is exportable, so enumeration is
            // `library_entries JOIN documents` (AC#4). Selection/cursor join on library_entry_id.
            r#"SELECT le.id AS "library_entry_id!",
                      d.id AS "document_id!",
                      d.title AS "title!",
                      COALESCE(d.original_url, d.canonical_url) AS url,
                      d.document_type AS "document_type!",
                      le.source AS "source!",
                      COALESCE(s.selected, true) AS "selected!: bool",
                      c.remote_page_id,
                      c.last_synced_at AS export_last_synced_at,
                      c.last_error AS export_last_error
               FROM integration_connections ic
               JOIN library_entries le ON le.user_id = ic.user_id AND le.deleted_at IS NULL
               JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id
               LEFT JOIN notion_export_item_selection s
                 ON s.connection_id = ic.id AND s.library_entry_id = le.id
               LEFT JOIN integration_export_cursor c
                 ON c.connection_id = ic.id AND c.library_entry_id = le.id
               WHERE ic.id = $1
                 AND ic.user_id = $2
                 AND ic.provider = 'notion'
                 AND ($3::text IS NULL
                      OR d.title ILIKE $3 ESCAPE '\'
                      OR COALESCE(d.original_url, d.canonical_url) ILIKE $3 ESCAPE '\')
               ORDER BY le.saved_at DESC, le.id DESC
               LIMIT $4 OFFSET $5"#,
            connection_id.into_uuid(),
            user_id.into_uuid(),
            search.as_deref(),
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        let total_count = sqlx::query_scalar!(
            r#"SELECT count(*) AS "count!"
               FROM integration_connections ic
               JOIN library_entries le ON le.user_id = ic.user_id AND le.deleted_at IS NULL
               JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id
               WHERE ic.id = $1
                 AND ic.user_id = $2
                 AND ic.provider = 'notion'"#,
            connection_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        let filtered_count = sqlx::query_scalar!(
            r#"SELECT count(*) AS "count!"
               FROM integration_connections ic
               JOIN library_entries le ON le.user_id = ic.user_id AND le.deleted_at IS NULL
               JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id
               WHERE ic.id = $1
                 AND ic.user_id = $2
                 AND ic.provider = 'notion'
                 AND ($3::text IS NULL
                      OR d.title ILIKE $3 ESCAPE '\'
                      OR COALESCE(d.original_url, d.canonical_url) ILIKE $3 ESCAPE '\')"#,
            connection_id.into_uuid(),
            user_id.into_uuid(),
            search.as_deref(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        let items = rows
            .into_iter()
            .map(NotionExportItem::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(NotionExportItemsPage {
            items,
            total_count,
            filtered_count,
        })
    }

    pub(super) async fn list_notion_export_candidates_impl(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        selected_only: bool,
        after: Option<NotionExportCursor>,
        limit: i64,
    ) -> Result<Vec<NotionExportCandidate>, AppError> {
        let limit = limit.clamp(1, 1000);
        let (after_saved_at, after_id) = match after {
            Some(c) => (Some(c.saved_at), Some(c.library_entry_id.into_uuid())),
            None => (None, None),
        };
        let rows = sqlx::query!(
            r#"SELECT le.id AS "library_entry_id!",
                      le.document_id AS "document_id!",
                      le.saved_at AS "saved_at!"
               FROM library_entries le
               JOIN documents d ON d.id = le.document_id
               LEFT JOIN notion_export_item_selection s
                 ON s.connection_id = $2 AND s.library_entry_id = le.id
               LEFT JOIN integration_export_cursor c
                 ON c.connection_id = $2 AND c.library_entry_id = le.id
               WHERE le.user_id = $1
                 AND le.deleted_at IS NULL
                 AND (NOT $3 OR COALESCE(s.selected, true) = true)
                 AND (c.last_synced_at IS NULL
                      OR c.last_synced_at < GREATEST(
                          COALESCE(d.updated_at, '-infinity'::timestamptz),
                          COALESCE(le.updated_at, '-infinity'::timestamptz)
                      )
                      OR EXISTS (
                          SELECT 1
                          FROM highlights h
                          WHERE h.document_id = d.id
                            AND h.user_id = d.user_id
                            AND (
                                c.last_exported_highlight_created_at IS NULL
                                OR h.created_at > c.last_exported_highlight_created_at
                                OR (
                                    h.created_at = c.last_exported_highlight_created_at
                                    AND h.id > c.last_exported_highlight_id
                                )
                            )
                      ))
                 AND ($4::timestamptz IS NULL
                      OR le.saved_at > $4
                      OR (le.saved_at = $4 AND le.id > $5))
               ORDER BY le.saved_at ASC, le.id ASC
               LIMIT $6"#,
            user_id.into_uuid(),
            connection_id.into_uuid(),
            selected_only,
            after_saved_at,
            after_id,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|row| NotionExportCandidate {
                library_entry_id: LibraryEntryId::from_uuid(row.library_entry_id),
                document_id: DocumentId::from_uuid(row.document_id),
                saved_at: row.saved_at,
            })
            .collect())
    }

    pub(super) async fn set_notion_export_item_selections_batch_impl(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        selections: &[(LibraryEntryId, bool)],
    ) -> Result<(), AppError> {
        if selections.is_empty() {
            return Ok(());
        }

        let library_entry_ids: Vec<Uuid> =
            selections.iter().map(|(id, _)| id.into_uuid()).collect();
        let selected_flags: Vec<bool> = selections.iter().map(|(_, s)| *s).collect();

        let mut tx = self.pool.begin().await.map_err(map_err)?;

        // Verify the connection belongs to the user up-front so we fail with a
        // domain NotFound rather than silently writing zero rows below if the
        // connection is gone or owned by someone else.
        let owner = sqlx::query_scalar!(
            r#"SELECT id FROM integration_connections
               WHERE id = $1 AND user_id = $2 AND provider = 'notion'"#,
            connection_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_err)?;

        if owner.is_none() {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "integration_connection",
                id: connection_id.to_string(),
            }));
        }

        let inserted = sqlx::query!(
            r#"WITH inputs AS (
                   SELECT library_entry_id, selected
                   FROM UNNEST($2::uuid[], $3::bool[]) AS t(library_entry_id, selected)
               ),
               valid AS (
                   SELECT i.library_entry_id, i.selected
                   FROM inputs i
                   JOIN library_entries le
                     ON le.id = i.library_entry_id AND le.user_id = $4 AND le.deleted_at IS NULL
               )
               INSERT INTO notion_export_item_selection
                   (connection_id, library_entry_id, selected, created_at, updated_at)
               SELECT $1, library_entry_id, selected, now(), now()
               FROM valid
               ON CONFLICT (connection_id, library_entry_id) DO UPDATE
                   SET selected = EXCLUDED.selected,
                       updated_at = now()
               RETURNING library_entry_id"#,
            connection_id.into_uuid(),
            &library_entry_ids,
            &selected_flags,
            user_id.into_uuid(),
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(map_err)?;

        if inserted.len() != selections.len() {
            // At least one item didn't belong to this user; roll the whole
            // batch back so a partial PATCH doesn't leave a confusing mix of
            // applied + skipped selections behind.
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "notion_export_item",
                id: format!(
                    "{} of {} items missing or not owned by user",
                    selections.len() - inserted.len(),
                    selections.len()
                ),
            }));
        }

        tx.commit().await.map_err(map_err)?;
        Ok(())
    }
}
