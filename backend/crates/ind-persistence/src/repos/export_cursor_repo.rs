use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::export_cursor::ExportCursorRepository;
use ind_domain::{
    DocumentId, ExportCursor, HighlightId, IntegrationConnectionId, JobOutbox, JobOutboxId,
    LibraryEntryId, NotionExportDocumentJob, UserId, job_types,
};

pub struct PgExportCursorRepository {
    pool: PgPool,
}

impl PgExportCursorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CursorRow {
    connection_id: Uuid,
    library_entry_id: Uuid,
    last_synced_at: Option<DateTime<Utc>>,
    last_attempted_at: Option<DateTime<Utc>>,
    cursor_version: i32,
    last_error: Option<String>,
    remote_page_id: Option<String>,
    last_exported_highlight_created_at: Option<DateTime<Utc>>,
    last_exported_highlight_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<CursorRow> for ExportCursor {
    fn from(row: CursorRow) -> Self {
        ExportCursor {
            connection_id: IntegrationConnectionId::from_uuid(row.connection_id),
            library_entry_id: LibraryEntryId::from_uuid(row.library_entry_id),
            last_synced_at: row.last_synced_at,
            last_attempted_at: row.last_attempted_at,
            cursor_version: row.cursor_version,
            last_error: row.last_error,
            remote_page_id: row.remote_page_id,
            last_exported_highlight_created_at: row.last_exported_highlight_created_at,
            last_exported_highlight_id: row.last_exported_highlight_id.map(HighlightId::from_uuid),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("export_cursor", "export cursor already exists", err)
}

#[async_trait::async_trait]
impl ExportCursorRepository for PgExportCursorRepository {
    async fn upsert(
        &self,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
    ) -> Result<ExportCursor, AppError> {
        let now = Utc::now();
        let row = sqlx::query_as!(
            CursorRow,
            r#"INSERT INTO integration_export_cursor
                (connection_id, library_entry_id, created_at, updated_at)
               VALUES ($1, $2, $3, $3)
               ON CONFLICT (connection_id, library_entry_id) DO UPDATE
                 SET updated_at = EXCLUDED.updated_at
               RETURNING connection_id, library_entry_id, last_synced_at, last_attempted_at,
                         cursor_version, last_error, remote_page_id,
                         last_exported_highlight_created_at,
                         last_exported_highlight_id,
                         created_at, updated_at"#,
            connection_id.into_uuid(),
            library_entry_id.into_uuid(),
            now,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.into())
    }

    async fn mark_attempted(
        &self,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
        at: DateTime<Utc>,
        error: Option<String>,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"UPDATE integration_export_cursor
               SET last_attempted_at = $3, last_error = $4, updated_at = now()
               WHERE connection_id = $1 AND library_entry_id = $2"#,
            connection_id.into_uuid(),
            library_entry_id.into_uuid(),
            at,
            error.as_deref(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        if result.rows_affected() == 0 {
            tracing::warn!(
                %connection_id,
                %library_entry_id,
                "mark_attempted: no cursor row found; error not recorded"
            );
        }
        Ok(())
    }

    async fn mark_synced(
        &self,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
        at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"UPDATE integration_export_cursor
               SET last_synced_at = $3,
                   last_attempted_at = $3,
                   last_error = NULL,
                   cursor_version = cursor_version + 1,
                   updated_at = now()
               WHERE connection_id = $1 AND library_entry_id = $2"#,
            connection_id.into_uuid(),
            library_entry_id.into_uuid(),
            at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn list_stale(
        &self,
        connection_id: IntegrationConnectionId,
        older_than: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<ExportCursor>, AppError> {
        let rows = sqlx::query_as!(
            CursorRow,
            r#"SELECT connection_id, library_entry_id, last_synced_at, last_attempted_at,
                      cursor_version, last_error, remote_page_id,
                      last_exported_highlight_created_at, last_exported_highlight_id,
                      created_at, updated_at
               FROM integration_export_cursor
               WHERE connection_id = $1
                 AND (last_attempted_at IS NULL OR last_attempted_at < $2)
               ORDER BY last_attempted_at NULLS FIRST
               LIMIT $3"#,
            connection_id.into_uuid(),
            older_than,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(rows.into_iter().map(ExportCursor::from).collect())
    }

    async fn mark_remote_page_resolved(
        &self,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
        remote_page_id: &str,
        at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"UPDATE integration_export_cursor
               SET remote_page_id = $3, last_attempted_at = $4, updated_at = now()
               WHERE connection_id = $1 AND library_entry_id = $2"#,
            connection_id.into_uuid(),
            library_entry_id.into_uuid(),
            remote_page_id,
            at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn mark_highlight_chunk_synced(
        &self,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
        last_highlight_created_at: DateTime<Utc>,
        last_highlight_id: HighlightId,
        at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"UPDATE integration_export_cursor
               SET last_exported_highlight_created_at = $3,
                   last_exported_highlight_id = $4,
                   last_attempted_at = $5,
                   updated_at = now()
               WHERE connection_id = $1 AND library_entry_id = $2"#,
            connection_id.into_uuid(),
            library_entry_id.into_uuid(),
            last_highlight_created_at,
            last_highlight_id.into_uuid(),
            at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn reset_document_export_and_enqueue_notion(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
        document_id: DocumentId,
        replaced_page_id: Option<String>,
    ) -> Result<JobOutbox, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_err)?;
        sqlx::query!(
            r#"UPDATE integration_export_cursor
               SET remote_page_id = NULL,
                   last_exported_highlight_created_at = NULL,
                   last_exported_highlight_id = NULL,
                   last_synced_at = NULL,
                   last_attempted_at = NULL,
                   last_error = NULL,
                   cursor_version = cursor_version + 1,
                   updated_at = now()
               WHERE connection_id = $1 AND library_entry_id = $2"#,
            connection_id.into_uuid(),
            library_entry_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;

        let payload = serde_json::to_value(NotionExportDocumentJob {
            connection_id,
            user_id,
            library_entry_id,
            document_id,
            replaced_page_id,
        })
        .map_err(|error| AppError::ExternalService {
            service: "notion".into(),
            message: format!("failed to serialize export_document payload: {error}"),
        })?;
        let now = Utc::now();
        let id = JobOutboxId::new();
        let dedupe_key = format!(
            "export:{}:{}",
            connection_id.into_uuid(),
            library_entry_id.into_uuid()
        );
        let row = sqlx::query_as!(
            NotionOutboxRow,
            r#"INSERT INTO job_outbox
                    (id, job_type, payload, dedupe_key, available_at, created_at)
               VALUES ($1, $2, $3, $4, $5, $5)
               ON CONFLICT (dedupe_key) WHERE dedupe_key IS NOT NULL DO UPDATE
                 SET payload = EXCLUDED.payload,
                     available_at = EXCLUDED.available_at,
                     dispatched_at = NULL
               RETURNING id, job_type, payload, dedupe_key,
                         available_at, dispatched_at, created_at"#,
            id.as_uuid(),
            job_types::INTEGRATION_NOTION_EXPORT_DOCUMENT,
            payload,
            dedupe_key,
            now,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_err)?;
        tx.commit().await.map_err(map_err)?;
        Ok(row.into())
    }

    async fn record_generated_path(
        &self,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
        new_path: String,
        new_full_document_path: String,
    ) -> Result<bool, AppError> {
        let rows = sqlx::query!(
            r#"UPDATE integration_export_cursor
               SET generated_path = $3,
                   generated_full_document_path = CASE
                       WHEN generated_full_document_path IS NULL THEN NULL
                       ELSE $4
                   END,
                   updated_at = now()
               WHERE connection_id = $1 AND library_entry_id = $2"#,
            connection_id.into_uuid(),
            library_entry_id.into_uuid(),
            new_path,
            new_full_document_path,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(rows.rows_affected() > 0)
    }
}

struct NotionOutboxRow {
    id: uuid::Uuid,
    job_type: String,
    payload: serde_json::Value,
    dedupe_key: Option<String>,
    available_at: DateTime<Utc>,
    dispatched_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<NotionOutboxRow> for JobOutbox {
    fn from(row: NotionOutboxRow) -> Self {
        Self {
            id: JobOutboxId::from(row.id),
            job_type: row.job_type,
            payload: row.payload,
            dedupe_key: row.dedupe_key,
            available_at: row.available_at,
            dispatched_at: row.dispatched_at,
            created_at: row.created_at,
        }
    }
}
