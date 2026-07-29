use chrono::{DateTime, Utc};

use ind_application::AppError;
use ind_domain::{DomainError, IntegrationConnectionId, UserId};

use super::obsidian_export_repo::PgObsidianExportRepository;

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("obsidian_export", "obsidian export row already exists", err)
}

impl PgObsidianExportRepository {
    pub async fn ensure_sync_run(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        requested_by_user: bool,
        run_id: Option<uuid::Uuid>,
    ) -> Result<uuid::Uuid, AppError> {
        if let Some(run_id) = run_id {
            let exists = sqlx::query_scalar!(
                r#"SELECT id
                   FROM obsidian_export_runs
                   WHERE id = $1 AND connection_id = $2 AND user_id = $3"#,
                run_id,
                connection_id.into_uuid(),
                user_id.into_uuid(),
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(map_err)?;
            if exists.is_none() {
                return Err(AppError::Domain(DomainError::NotFound {
                    entity: "obsidian_export_run",
                    id: run_id.to_string(),
                }));
            }
            return Ok(run_id);
        }

        let run_id = uuid::Uuid::now_v7();
        sqlx::query!(
            r#"INSERT INTO obsidian_export_runs
               (id, connection_id, user_id, status, requested_by_user, auto,
                parent_folder_deleted, force_item_ids, created_at, updated_at)
               VALUES ($1, $2, $3, 'pending', $4, false, false, '{}', now(), now())"#,
            run_id,
            connection_id.into_uuid(),
            user_id.into_uuid(),
            requested_by_user,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(run_id)
    }

    pub async fn load_sync_connection(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
    ) -> Result<ObsidianSyncConnectionRecord, AppError> {
        sqlx::query_as!(
            ObsidianSyncConnectionRecord,
            r#"SELECT provider, status, config
               FROM integration_connections
               WHERE id = $1 AND user_id = $2"#,
            connection_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "integration_connection",
                id: connection_id.to_string(),
            })
        })
    }

    pub async fn load_sync_run(
        &self,
        run_id: uuid::Uuid,
    ) -> Result<ObsidianSyncRunRecord, AppError> {
        sqlx::query_as!(
            ObsidianSyncRunRecord,
            r#"SELECT parent_folder_deleted, force_item_ids
               FROM obsidian_export_runs
               WHERE id = $1"#,
            run_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "obsidian_export_run",
                id: run_id.to_string(),
            })
        })
    }

    pub async fn claim_sync_run(&self, run_id: uuid::Uuid) -> Result<bool, AppError> {
        let claimed = sqlx::query_scalar!(
            r#"UPDATE obsidian_export_runs
               SET status = 'running', error = NULL, updated_at = now()
               WHERE id = $1 AND status IN ('pending', 'running')
               RETURNING id"#,
            run_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(claimed.is_some())
    }

    pub async fn list_sync_candidates(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        run: &ObsidianSyncRunRecord,
        export_all_reader_documents: bool,
    ) -> Result<Vec<ObsidianSyncCandidateRecord>, AppError> {
        // TASK-236: only saved Library content is enumerable (AC#4). `library_entries JOIN
        // documents` is the saved set, so the legacy explicitly-saved-feed-item filter is gone.
        // Capabilities/assets hydrate by `document_id`; cursor/refresh-queue join on
        // `library_entry_id`. Unsaved-authored documents (AC#5) are served by the plugin pull API
        // under `ExportScope::IncludeUnsavedAuthored`, not by this server-built ZIP sync.
        let rows = sqlx::query_as!(
            ObsidianSyncCandidateRecord,
            r#"SELECT le.id AS "library_entry_id!",
                      d.id AS "document_id!",
                      iec.last_delivered_at,
                      iec.last_exported_highlight_created_at,
                      iec.last_exported_highlight_id,
                      iec.last_exported_file_hash,
                      iec.last_exported_full_document_hash,
                      iec.generated_path,
                      iec.generated_full_document_path,
                      ($4 OR le.id = ANY($3) OR q.library_entry_id IS NOT NULL OR iec.last_delivered_at IS NULL) AS "force_full!: bool"
               FROM library_entries le
               JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id
               LEFT JOIN integration_export_cursor iec
                 ON iec.connection_id = $2 AND iec.library_entry_id = le.id
               LEFT JOIN obsidian_export_refresh_queue q
                 ON q.connection_id = $2 AND q.library_entry_id = le.id
                    AND (q.next_attempt_at IS NULL OR q.next_attempt_at <= now())
               WHERE le.user_id = $1
                 AND le.deleted_at IS NULL
                 AND (
                   le.id = ANY($3)
                   OR q.library_entry_id IS NOT NULL
                   OR $4
                   OR iec.last_delivered_at IS NULL
                   OR EXISTS (
                     SELECT 1
                     FROM highlights h
                     WHERE h.user_id = $1
                       AND h.document_id = d.id
                       AND (
                         iec.last_exported_highlight_created_at IS NULL
                         OR h.created_at > iec.last_exported_highlight_created_at
                         OR (
                           h.created_at = iec.last_exported_highlight_created_at
                           AND h.id > COALESCE(iec.last_exported_highlight_id, '00000000-0000-0000-0000-000000000000'::uuid)
                         )
                       )
                   )
                   OR (
                     $5
                     AND EXISTS (
                       SELECT 1
                       FROM archive_assets aa
                       WHERE aa.document_id = d.id
                         AND aa.asset_kind IN ('readable_html', 'epub', 'pdf')
                         AND aa.status = 'completed'
                         AND btrim(aa.s3_key) <> ''
                         AND (
                           iec.last_delivered_at IS NULL
                           OR iec.last_exported_full_document_hash IS NULL
                           OR aa.created_at > iec.last_delivered_at
                         )
                     )
                   )
                 )
                 AND (
                   le.id = ANY($3)
                   OR q.library_entry_id IS NOT NULL
                   OR EXISTS (
                     SELECT 1
                     FROM highlights h
                     WHERE h.user_id = $1 AND h.document_id = d.id
                   )
                   OR (
                     $5
                     AND EXISTS (
                       SELECT 1
                       FROM archive_assets aa
                       WHERE aa.document_id = d.id
                         AND aa.asset_kind IN ('readable_html', 'epub', 'pdf')
                         AND aa.status = 'completed'
                         AND btrim(aa.s3_key) <> ''
                     )
                   )
                 )
               ORDER BY le.saved_at ASC, le.id ASC"#,
            user_id.into_uuid(),
            connection_id.into_uuid(),
            &run.force_item_ids,
            run.parent_folder_deleted,
            export_all_reader_documents,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(rows)
    }

    pub async fn store_sync_artifact(
        &self,
        run_id: uuid::Uuid,
        connection_id: IntegrationConnectionId,
        user_id: UserId,
        zip_bytes: &[u8],
        items: &[ObsidianSyncArtifactItemInsert],
    ) -> Result<uuid::Uuid, AppError> {
        let artifact_id = uuid::Uuid::now_v7();
        let zip_byte_size = i32::try_from(zip_bytes.len()).map_err(|_| {
            AppError::Domain(DomainError::Validation {
                field: "zip".to_string(),
                message: "artifact exceeds i32 byte size".to_string(),
            })
        })?;

        let mut tx = self.pool.begin().await.map_err(map_err)?;
        sqlx::query!(
            r#"INSERT INTO obsidian_export_artifacts
               (id, run_id, connection_id, user_id, content_type, byte_size, bytes, created_at)
               VALUES ($1, $2, $3, $4, 'application/zip', $5, $6, now())"#,
            artifact_id,
            run_id,
            connection_id.into_uuid(),
            user_id.into_uuid(),
            zip_byte_size,
            zip_bytes,
        )
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;

        for item in items {
            sqlx::query!(
                r#"INSERT INTO obsidian_export_artifact_items
                   (artifact_id, library_entry_id, file_path, full_document_path,
                    last_highlight_created_at, last_highlight_id, content_hash,
                    full_document_hash, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())"#,
                artifact_id,
                item.library_entry_id,
                item.file_path,
                item.full_document_path,
                item.last_highlight_created_at,
                item.last_highlight_id,
                item.last_content_hash,
                item.last_full_document_hash,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
        }

        tx.commit().await.map_err(map_err)?;
        Ok(artifact_id)
    }

    pub async fn mark_sync_run_succeeded(
        &self,
        run_id: uuid::Uuid,
        total_documents: i32,
        documents_exported: i32,
    ) -> Result<(), AppError> {
        let connection_id = sqlx::query_scalar!(
            r#"UPDATE obsidian_export_runs
               SET status = 'success',
                   total_documents = $2,
                   documents_exported = $3,
                   error = NULL,
                   updated_at = now(),
                   finished_at = now()
               WHERE id = $1 AND status = 'running'
               RETURNING connection_id"#,
            run_id,
            total_documents,
            documents_exported,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        if let Some(connection_id) = connection_id {
            if documents_exported > 0 {
                sqlx::query!(
                    r#"UPDATE integration_connections
                       SET last_sync_at = now(),
                           status = CASE WHEN status = 'pending' THEN 'active' ELSE status END,
                           last_error = NULL,
                           updated_at = now()
                       WHERE id = $1"#,
                    connection_id,
                )
                .execute(&self.pool)
                .await
                .map_err(map_err)?;
            } else {
                sqlx::query!(
                    r#"UPDATE integration_connections
                       SET status = CASE WHEN status = 'pending' THEN 'active' ELSE status END,
                           last_error = NULL,
                           updated_at = now()
                       WHERE id = $1"#,
                    connection_id,
                )
                .execute(&self.pool)
                .await
                .map_err(map_err)?;
            }
        } else {
            tracing::warn!(
                run_id = %run_id,
                "mark_run_succeeded skipped: run is no longer in 'running' state"
            );
        }
        Ok(())
    }

    pub async fn mark_sync_run_artifact_ready(
        &self,
        run_id: uuid::Uuid,
        total_documents: i32,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"UPDATE obsidian_export_runs
               SET status = 'artifact_ready',
                   total_documents = $2,
                   documents_exported = 0,
                   error = NULL,
                   updated_at = now(),
                   finished_at = now()
               WHERE id = $1 AND status = 'running'"#,
            run_id,
            total_documents,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        if result.rows_affected() == 0 {
            tracing::warn!(
                run_id = %run_id,
                "mark_run_artifact_ready skipped: run is no longer in 'running' state"
            );
        }
        Ok(())
    }

    pub async fn mark_sync_run_failed(
        &self,
        run_id: uuid::Uuid,
        error: &str,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"UPDATE obsidian_export_runs
               SET status = 'failed',
                   error = $2,
                   updated_at = now(),
                   finished_at = now()
               WHERE id = $1 AND status = 'running'"#,
            run_id,
            error,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        if result.rows_affected() == 0 {
            tracing::warn!(
                run_id = %run_id,
                error = %error,
                "mark_run_failed skipped: run is no longer in 'running' state"
            );
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ObsidianSyncConnectionRecord {
    pub provider: String,
    pub status: String,
    pub config: serde_json::Value,
}

#[derive(Debug)]
pub struct ObsidianSyncRunRecord {
    pub parent_folder_deleted: bool,
    pub force_item_ids: Vec<uuid::Uuid>,
}

#[derive(Debug)]
pub struct ObsidianSyncCandidateRecord {
    pub library_entry_id: uuid::Uuid,
    pub document_id: uuid::Uuid,
    pub last_delivered_at: Option<DateTime<Utc>>,
    pub last_exported_highlight_created_at: Option<DateTime<Utc>>,
    pub last_exported_highlight_id: Option<uuid::Uuid>,
    pub last_exported_file_hash: Option<String>,
    pub last_exported_full_document_hash: Option<String>,
    pub generated_path: Option<String>,
    pub generated_full_document_path: Option<String>,
    pub force_full: bool,
}

#[derive(Debug)]
pub struct ObsidianSyncArtifactItemInsert {
    pub library_entry_id: uuid::Uuid,
    pub file_path: String,
    pub full_document_path: Option<String>,
    pub last_highlight_created_at: Option<DateTime<Utc>>,
    pub last_highlight_id: Option<uuid::Uuid>,
    pub last_content_hash: Option<String>,
    pub last_full_document_hash: Option<String>,
}
