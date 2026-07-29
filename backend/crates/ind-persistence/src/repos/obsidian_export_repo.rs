use std::collections::{BTreeSet, HashMap, HashSet};

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};

use ind_application::AppError;
use ind_application::repos::obsidian_export::{
    AckObsidianRunInput, CreateObsidianRunInput, ObsidianArtifactDownloadRecord,
    ObsidianExportRepository, ObsidianRunStatusRecord,
};
use ind_domain::{DomainError, IntegrationConnectionId, LibraryEntryId, UserId};

pub struct PgObsidianExportRepository {
    pub(super) pool: PgPool,
}

impl PgObsidianExportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct ObsidianRunStatusRow {
    id: uuid::Uuid,
    status: String,
    total_documents: i32,
    documents_exported: i32,
    error: Option<String>,
    artifact_ids: Vec<uuid::Uuid>,
}

struct ArtifactRow {
    id: uuid::Uuid,
    content_type: String,
    bytes: Vec<u8>,
}

struct ObsidianArtifactItemRow {
    artifact_id: uuid::Uuid,
    library_entry_id: uuid::Uuid,
    file_path: String,
    full_document_path: Option<String>,
    last_highlight_created_at: Option<DateTime<Utc>>,
    last_highlight_id: Option<uuid::Uuid>,
}

struct RefreshAttemptRow {
    library_entry_id: uuid::Uuid,
    delivery_attempts: i32,
}

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("obsidian_export", "obsidian export row already exists", err)
}

fn run_status_from_row(row: ObsidianRunStatusRow) -> ObsidianRunStatusRecord {
    ObsidianRunStatusRecord {
        run_id: row.id,
        status: row.status,
        total_documents: row.total_documents,
        documents_exported: row.documents_exported,
        artifact_ids: row.artifact_ids,
        error: row.error,
    }
}

fn unique_library_entry_ids(
    library_entry_ids: &[LibraryEntryId],
    field: &'static str,
) -> Result<Vec<uuid::Uuid>, AppError> {
    let ids: Vec<uuid::Uuid> = library_entry_ids.iter().map(|id| id.into_uuid()).collect();
    let unique: HashSet<uuid::Uuid> = ids.iter().copied().collect();
    if unique.len() != ids.len() {
        return Err(AppError::Domain(DomainError::Validation {
            field: field.to_string(),
            message: "library entry ids must be unique".to_string(),
        }));
    }
    Ok(ids)
}

async fn validate_owned_live_library_entry_ids_tx(
    tx: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    library_entry_ids: &[LibraryEntryId],
    field: &'static str,
) -> Result<Vec<uuid::Uuid>, AppError> {
    let ids = unique_library_entry_ids(library_entry_ids, field)?;
    if ids.is_empty() {
        return Ok(ids);
    }

    let owned = sqlx::query_scalar!(
        "SELECT id FROM library_entries
         WHERE user_id = $1 AND deleted_at IS NULL AND id = ANY($2)
         FOR UPDATE",
        user_id.into_uuid(),
        &ids,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(map_err)?;

    if owned.len() != ids.len() {
        return Err(AppError::Domain(DomainError::Validation {
            field: field.to_string(),
            message: "all library entry ids must belong to the authenticated user".to_string(),
        }));
    }

    Ok(ids)
}

async fn fetch_run_status(
    pool: &PgPool,
    user_id: UserId,
    run_id: uuid::Uuid,
) -> Result<Option<ObsidianRunStatusRecord>, AppError> {
    let row = sqlx::query_as!(
        ObsidianRunStatusRow,
        r#"SELECT r.id, r.status, r.total_documents, r.documents_exported, r.error,
                  COALESCE(array_agg(a.id ORDER BY a.created_at) FILTER (WHERE a.id IS NOT NULL), '{}') AS "artifact_ids!: Vec<uuid::Uuid>"
           FROM obsidian_export_runs r
           LEFT JOIN obsidian_export_artifacts a ON a.run_id = r.id
           WHERE r.id = $1 AND r.user_id = $2
           GROUP BY r.id, r.status, r.total_documents, r.documents_exported, r.error"#,
        run_id,
        user_id.into_uuid(),
    )
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;

    Ok(row.map(run_status_from_row))
}

#[async_trait::async_trait]
impl ObsidianExportRepository for PgObsidianExportRepository {
    async fn create_run(&self, input: CreateObsidianRunInput) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_err)?;
        let force_subject_ids = validate_owned_live_library_entry_ids_tx(
            &mut tx,
            input.user_id,
            &input.force_library_entry_ids,
            "force_library_entry_ids",
        )
        .await?;

        sqlx::query!(
            r#"INSERT INTO obsidian_export_runs
               (id, connection_id, user_id, status, requested_by_user, auto,
                parent_folder_deleted, force_item_ids, created_at, updated_at)
               VALUES ($1, $2, $3, 'pending', $4, $5, $6, $7, now(), now())"#,
            input.run_id,
            input.connection_id.into_uuid(),
            input.user_id.into_uuid(),
            input.requested_by_user,
            input.auto,
            input.parent_folder_deleted,
            &force_subject_ids,
        )
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;

        tx.commit().await.map_err(map_err)?;
        Ok(())
    }

    async fn run_status(
        &self,
        user_id: UserId,
        run_id: uuid::Uuid,
    ) -> Result<Option<ObsidianRunStatusRecord>, AppError> {
        fetch_run_status(&self.pool, user_id, run_id).await
    }

    async fn artifact_download(
        &self,
        user_id: UserId,
        artifact_id: uuid::Uuid,
    ) -> Result<Option<ObsidianArtifactDownloadRecord>, AppError> {
        let row = sqlx::query_as!(
            ArtifactRow,
            r#"SELECT id, content_type, bytes
               FROM obsidian_export_artifacts
               WHERE id = $1 AND user_id = $2"#,
            artifact_id,
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(|row| ObsidianArtifactDownloadRecord {
            artifact_id: row.id,
            content_type: row.content_type,
            bytes: row.bytes,
        }))
    }

    async fn ack_run(
        &self,
        user_id: UserId,
        run_id: uuid::Uuid,
        input: AckObsidianRunInput,
    ) -> Result<ObsidianRunStatusRecord, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_err)?;

        let connection_id = sqlx::query_scalar!(
            "SELECT connection_id FROM obsidian_export_runs WHERE id = $1 AND user_id = $2",
            run_id,
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_err)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "obsidian_export_run",
                id: run_id.to_string(),
            })
        })?;

        let artifact_ids = if input.artifact_ids.is_empty() {
            sqlx::query_scalar!(
                "SELECT id FROM obsidian_export_artifacts
                 WHERE run_id = $1 AND user_id = $2 AND connection_id = $3",
                run_id,
                user_id.into_uuid(),
                connection_id,
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(map_err)?
        } else {
            let requested: BTreeSet<uuid::Uuid> = input.artifact_ids.iter().copied().collect();
            let scoped = sqlx::query_scalar!(
                "SELECT id FROM obsidian_export_artifacts
                 WHERE id = ANY($1) AND run_id = $2 AND user_id = $3 AND connection_id = $4",
                &input.artifact_ids,
                run_id,
                user_id.into_uuid(),
                connection_id,
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(map_err)?;
            let scoped_set: BTreeSet<uuid::Uuid> = scoped.iter().copied().collect();
            if scoped_set != requested {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "artifact_ids".into(),
                    message: "all artifact_ids must belong to the acknowledged run".into(),
                }));
            }
            scoped
        };

        let rows = sqlx::query_as!(
            ObsidianArtifactItemRow,
            r#"SELECT artifact_id, library_entry_id, file_path, full_document_path,
                      last_highlight_created_at, last_highlight_id
               FROM obsidian_export_artifact_items
               WHERE artifact_id = ANY($1)"#,
            &artifact_ids,
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(map_err)?;

        let total_artifact_subjects = i32::try_from(rows.len()).map_err(|_| {
            AppError::Domain(DomainError::Validation {
                field: "artifact_ids".into(),
                message: "too many artifact subjects to acknowledge".into(),
            })
        })?;
        let ack_by_subject: HashMap<uuid::Uuid, _> = input
            .subjects
            .into_iter()
            .map(|subject| (subject.library_entry_id.into_uuid(), subject))
            .collect();

        let mut success_artifact_ids = Vec::new();
        let mut success_subject_ids = Vec::new();
        let mut success_last_highlight_created_ats = Vec::new();
        let mut success_last_highlight_ids = Vec::new();
        let mut success_content_hashes = Vec::new();
        let mut success_full_document_hashes = Vec::new();
        let mut success_file_paths = Vec::new();
        let mut success_full_document_paths = Vec::new();
        let mut failed_artifact_ids = Vec::new();
        let mut failed_subject_ids = Vec::new();
        let mut failed_errors = Vec::new();

        for row in rows {
            let ack = ack_by_subject.get(&row.library_entry_id);
            // Missing subject acknowledgements are failures so omitted plugin responses cannot advance cursors silently.
            let status = ack
                .map(|subject| subject.status.as_str())
                .unwrap_or("failed");
            if status == "success" {
                success_artifact_ids.push(row.artifact_id);
                success_subject_ids.push(row.library_entry_id);
                success_last_highlight_created_ats.push(row.last_highlight_created_at);
                success_last_highlight_ids.push(row.last_highlight_id);
                success_content_hashes
                    .push(ack.and_then(|subject| subject.last_content_hash.clone()));
                success_full_document_hashes
                    .push(ack.and_then(|subject| subject.last_full_document_hash.clone()));
                success_file_paths.push(row.file_path);
                success_full_document_paths.push(row.full_document_path);
            } else {
                let error = ack
                    .and_then(|subject| subject.error.clone())
                    .unwrap_or_else(|| {
                        if ack.is_none() {
                            "plugin did not report status for this subject".to_string()
                        } else {
                            "plugin write failed".to_string()
                        }
                    });
                failed_artifact_ids.push(row.artifact_id);
                failed_subject_ids.push(row.library_entry_id);
                failed_errors.push(error);
            }
        }

        if !success_subject_ids.is_empty() {
            sqlx::query!(
                r#"UPDATE obsidian_export_artifact_items AS target
                   SET delivered_at = now(), last_error = NULL
                   FROM UNNEST($1::uuid[], $2::uuid[]) AS delivered(artifact_id, library_entry_id)
                   WHERE target.artifact_id = delivered.artifact_id
                     AND target.library_entry_id = delivered.library_entry_id"#,
                &success_artifact_ids,
                &success_subject_ids,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;

            sqlx::query!(
                r#"INSERT INTO integration_export_cursor
                    (connection_id, library_entry_id, last_synced_at, last_attempted_at,
                     last_delivered_at, last_exported_highlight_created_at,
                     last_exported_highlight_id, last_exported_file_hash,
                     last_exported_full_document_hash, generated_path,
                     generated_full_document_path, created_at, updated_at)
                   SELECT $1, delivered.library_entry_id, now(), now(), now(),
                          delivered.last_highlight_created_at,
                          delivered.last_highlight_id,
                          delivered.last_exported_file_hash,
                          delivered.last_exported_full_document_hash,
                          delivered.file_path,
                          delivered.full_document_path,
                          now(), now()
                   FROM UNNEST(
                        $2::uuid[],
                        $3::timestamptz[],
                        $4::uuid[],
                        $5::text[],
                        $6::text[],
                        $7::text[],
                        $8::text[]
                   ) AS delivered(
                        library_entry_id,
                        last_highlight_created_at,
                        last_highlight_id,
                        last_exported_file_hash,
                        last_exported_full_document_hash,
                        file_path,
                        full_document_path
                   )
                   ON CONFLICT (connection_id, library_entry_id) DO UPDATE SET
                     last_synced_at = now(),
                     last_attempted_at = now(),
                     last_delivered_at = now(),
                     last_exported_highlight_created_at = COALESCE(EXCLUDED.last_exported_highlight_created_at, integration_export_cursor.last_exported_highlight_created_at),
                     last_exported_highlight_id = COALESCE(EXCLUDED.last_exported_highlight_id, integration_export_cursor.last_exported_highlight_id),
                     last_exported_file_hash = COALESCE(EXCLUDED.last_exported_file_hash, integration_export_cursor.last_exported_file_hash),
                     last_exported_full_document_hash = COALESCE(EXCLUDED.last_exported_full_document_hash, integration_export_cursor.last_exported_full_document_hash),
                     generated_path = EXCLUDED.generated_path,
                     generated_full_document_path = EXCLUDED.generated_full_document_path,
                     explicit_reimport_requested_at = NULL,
                     last_error = NULL,
                     updated_at = now()"#,
                connection_id,
                &success_subject_ids,
                &success_last_highlight_created_ats as &[Option<DateTime<Utc>>],
                &success_last_highlight_ids as &[Option<uuid::Uuid>],
                &success_content_hashes as &[Option<String>],
                &success_full_document_hashes as &[Option<String>],
                &success_file_paths,
                &success_full_document_paths as &[Option<String>],
            )
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;

            sqlx::query!(
                "DELETE FROM obsidian_export_refresh_queue WHERE connection_id = $1 AND library_entry_id = ANY($2)",
                connection_id,
                &success_subject_ids,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
        }

        if !failed_subject_ids.is_empty() {
            sqlx::query!(
                r#"UPDATE obsidian_export_artifact_items AS target
                   SET last_error = failed.last_error
                   FROM UNNEST($1::uuid[], $2::uuid[], $3::text[]) AS failed(artifact_id, library_entry_id, last_error)
                   WHERE target.artifact_id = failed.artifact_id
                     AND target.library_entry_id = failed.library_entry_id"#,
                &failed_artifact_ids,
                &failed_subject_ids,
                &failed_errors,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;

            sqlx::query!(
                r#"UPDATE integration_export_cursor AS cursor
                   SET last_error = failed.last_error, updated_at = now()
                   FROM UNNEST($2::uuid[], $3::text[]) AS failed(library_entry_id, last_error)
                   WHERE cursor.connection_id = $1
                     AND cursor.library_entry_id = failed.library_entry_id"#,
                connection_id,
                &failed_subject_ids,
                &failed_errors,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;

            let attempts = sqlx::query_as!(
                RefreshAttemptRow,
                r#"WITH failed(library_entry_id) AS (
                     SELECT library_entry_id FROM UNNEST($2::uuid[]) AS failed(library_entry_id)
                   ),
                   bumped AS (
                     UPDATE obsidian_export_refresh_queue AS queue
                     SET delivery_attempts = queue.delivery_attempts + 1,
                         next_attempt_at = now() + (interval '1 minute' * (1 << LEAST(queue.delivery_attempts, 8)))
                     FROM failed
                     WHERE queue.connection_id = $1
                       AND queue.library_entry_id = failed.library_entry_id
                     RETURNING queue.library_entry_id, queue.delivery_attempts
                   )
                   SELECT library_entry_id, delivery_attempts FROM bumped"#,
                connection_id,
                &failed_subject_ids,
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(map_err)?;

            // Permanently failing refreshes eventually require manual intervention instead of requeueing forever.
            const MAX_REFRESH_ATTEMPTS: i32 = 5;
            let failed_errors_by_subject: HashMap<uuid::Uuid, String> = failed_subject_ids
                .iter()
                .copied()
                .zip(failed_errors.iter().cloned())
                .collect();
            let mut gave_up_subject_ids = Vec::new();
            let mut gave_up_errors = Vec::new();
            for attempt in attempts {
                if attempt.delivery_attempts >= MAX_REFRESH_ATTEMPTS {
                    gave_up_subject_ids.push(attempt.library_entry_id);
                    let error = failed_errors_by_subject
                        .get(&attempt.library_entry_id)
                        .cloned()
                        .unwrap_or_else(|| "plugin write failed".to_string());
                    gave_up_errors.push(format!(
                        "{error} (gave up after {MAX_REFRESH_ATTEMPTS} delivery attempts; manual refresh required)"
                    ));
                }
            }

            if !gave_up_subject_ids.is_empty() {
                sqlx::query!(
                    "DELETE FROM obsidian_export_refresh_queue WHERE connection_id = $1 AND library_entry_id = ANY($2)",
                    connection_id,
                    &gave_up_subject_ids,
                )
                .execute(&mut *tx)
                .await
                .map_err(map_err)?;

                sqlx::query!(
                    r#"UPDATE integration_export_cursor AS cursor
                       SET last_error = gave_up.last_error, updated_at = now()
                       FROM UNNEST($2::uuid[], $3::text[]) AS gave_up(library_entry_id, last_error)
                    WHERE cursor.connection_id = $1
                         AND cursor.library_entry_id = gave_up.library_entry_id"#,
                    connection_id,
                    &gave_up_subject_ids,
                    &gave_up_errors,
                )
                .execute(&mut *tx)
                .await
                .map_err(map_err)?;
            }
        }

        let delivered_count = i32::try_from(success_subject_ids.len()).map_err(|_| {
            AppError::Domain(DomainError::Validation {
                field: "artifact_ids".into(),
                message: "too many artifact subjects to acknowledge".into(),
            })
        })?;
        let run_status =
            if total_artifact_subjects == 0 || delivered_count == total_artifact_subjects {
                "success"
            } else if delivered_count == 0 {
                "failed"
            } else {
                "partial_success"
            };
        let run_error = match run_status {
            "partial_success" => Some("Some Obsidian documents failed to write"),
            "failed" => Some("No Obsidian documents were delivered by the plugin"),
            _ => None,
        };

        sqlx::query!(
            r#"UPDATE obsidian_export_runs
               SET status = $2,
                   documents_exported = $3,
                   error = $4,
                   updated_at = now(),
                   finished_at = now()
               WHERE id = $1"#,
            run_id,
            run_status,
            delivered_count,
            run_error,
        )
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;

        if delivered_count > 0 {
            sqlx::query!(
                r#"UPDATE integration_connections
                   SET last_sync_at = now(),
                       status = CASE WHEN status = 'pending' THEN 'active' ELSE status END,
                       last_error = NULL,
                       updated_at = now()
                   WHERE id = $1"#,
                connection_id,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
        } else {
            sqlx::query!(
                r#"UPDATE integration_connections
                   SET status = CASE WHEN status = 'pending' THEN 'active' ELSE status END,
                       last_error = $2,
                       updated_at = now()
                   WHERE id = $1"#,
                connection_id,
                run_error,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
        }

        let row = sqlx::query_as!(
            ObsidianRunStatusRow,
            r#"SELECT r.id, r.status, r.total_documents, r.documents_exported, r.error,
                      COALESCE(array_agg(a.id ORDER BY a.created_at) FILTER (WHERE a.id IS NOT NULL), '{}') AS "artifact_ids!: Vec<uuid::Uuid>"
               FROM obsidian_export_runs r
               LEFT JOIN obsidian_export_artifacts a ON a.run_id = r.id
               WHERE r.id = $1 AND r.user_id = $2
               GROUP BY r.id, r.status, r.total_documents, r.documents_exported, r.error"#,
            run_id,
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_err)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "obsidian_export_run",
                id: run_id.to_string(),
            })
        })?;

        tx.commit().await.map_err(map_err)?;
        Ok(run_status_from_row(row))
    }

    async fn queue_refresh_subjects(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        library_entry_ids: &[LibraryEntryId],
        reason: &str,
    ) -> Result<u32, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_err)?;
        let library_entry_ids = validate_owned_live_library_entry_ids_tx(
            &mut tx,
            user_id,
            library_entry_ids,
            "library_entry_ids",
        )
        .await?;
        if library_entry_ids.is_empty() {
            return Ok(0);
        }

        let queued = sqlx::query!(
            r#"INSERT INTO obsidian_export_refresh_queue
                (connection_id, library_entry_id, reason, requested_at)
               SELECT $1, library_entry_id, $3, now()
               FROM UNNEST($2::uuid[]) AS input(library_entry_id)
               ON CONFLICT (connection_id, library_entry_id) DO UPDATE
                 SET reason = EXCLUDED.reason, requested_at = EXCLUDED.requested_at
               RETURNING library_entry_id"#,
            connection_id.into_uuid(),
            &library_entry_ids,
            reason,
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(map_err)?;

        tx.commit().await.map_err(map_err)?;

        u32::try_from(queued.len()).map_err(|_| {
            AppError::Domain(DomainError::Validation {
                field: "library_entry_ids".into(),
                message: "too many items queued for refresh".into(),
            })
        })
    }
}
