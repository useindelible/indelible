use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::import_job::ImportJobRepository;
use ind_domain::{
    DocumentOriginType, DomainError, ImportItemOutcome, ImportJob, ImportJobCountsDelta,
    ImportJobId, ImportJobItem, ImportJobStatus, ImportMethod, ImportSource, UserId,
    deterministic_origin_id,
};

pub struct PgImportJobRepository {
    pool: PgPool,
}

impl PgImportJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct JobRow {
    id: Uuid,
    user_id: Uuid,
    import_source: String,
    import_method: String,
    status: String,
    imported_count: i32,
    updated_count: i32,
    duplicate_count: i32,
    skipped_private_count: i32,
    failed_count: i32,
    raw_artifact_key: Option<String>,
    provider_report: Option<serde_json::Value>,
    error: Option<String>,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
}

impl TryFrom<JobRow> for ImportJob {
    type Error = AppError;

    fn try_from(row: JobRow) -> Result<Self, Self::Error> {
        Ok(ImportJob {
            id: ImportJobId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            import_source: parse_import_source(&row.import_source)?,
            import_method: parse_import_method(&row.import_method)?,
            status: parse_status(&row.status)?,
            imported_count: row.imported_count,
            updated_count: row.updated_count,
            duplicate_count: row.duplicate_count,
            skipped_private_count: row.skipped_private_count,
            failed_count: row.failed_count,
            raw_artifact_key: row.raw_artifact_key,
            provider_report: row.provider_report,
            error: row.error,
            created_at: row.created_at,
            started_at: row.started_at,
            finished_at: row.finished_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ItemOutcomeRow {
    id: Uuid,
    import_job_id: Uuid,
    external_id: String,
    title: Option<String>,
    outcome: String,
    error: Option<String>,
    created_at: DateTime<Utc>,
}

impl TryFrom<ItemOutcomeRow> for ImportJobItem {
    type Error = AppError;

    fn try_from(row: ItemOutcomeRow) -> Result<Self, Self::Error> {
        Ok(ImportJobItem {
            id: row.id,
            import_job_id: ImportJobId::from_uuid(row.import_job_id),
            external_id: row.external_id,
            title: row
                .title
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty()),
            outcome: parse_outcome(&row.outcome)?,
            error: row.error,
            created_at: row.created_at,
        })
    }
}

fn parse_import_source(s: &str) -> Result<ImportSource, AppError> {
    match s {
        "readwise_import" => Ok(ImportSource::ReadwiseImport),
        "notion_import" => Ok(ImportSource::NotionImport),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid import source: {other}"),
        })),
    }
}

fn parse_import_method(s: &str) -> Result<ImportMethod, AppError> {
    match s {
        "oauth" => Ok(ImportMethod::Oauth),
        "csv" => Ok(ImportMethod::Csv),
        "zip" => Ok(ImportMethod::Zip),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid import method: {other}"),
        })),
    }
}

fn parse_status(s: &str) -> Result<ImportJobStatus, AppError> {
    match s {
        "awaiting_provider" => Ok(ImportJobStatus::AwaitingProvider),
        "pending" => Ok(ImportJobStatus::Pending),
        "running" => Ok(ImportJobStatus::Running),
        "completed" => Ok(ImportJobStatus::Completed),
        "failed" => Ok(ImportJobStatus::Failed),
        "partial" => Ok(ImportJobStatus::Partial),
        "rolled_back" => Ok(ImportJobStatus::RolledBack),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid import job status: {other}"),
        })),
    }
}

fn parse_outcome(s: &str) -> Result<ImportItemOutcome, AppError> {
    match s {
        "imported" => Ok(ImportItemOutcome::Imported),
        "updated" => Ok(ImportItemOutcome::Updated),
        "duplicate" => Ok(ImportItemOutcome::Duplicate),
        "skipped_private" => Ok(ImportItemOutcome::SkippedPrivate),
        "failed" => Ok(ImportItemOutcome::Failed),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid import item outcome: {other}"),
        })),
    }
}

fn import_source_to_str(source: ImportSource) -> &'static str {
    source.as_str()
}

fn import_method_to_str(method: ImportMethod) -> &'static str {
    match method {
        ImportMethod::Oauth => "oauth",
        ImportMethod::Csv => "csv",
        ImportMethod::Zip => "zip",
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("import_job", "import job already exists", err)
}

#[async_trait::async_trait]
impl ImportJobRepository for PgImportJobRepository {
    async fn create(
        &self,
        user_id: UserId,
        source: ImportSource,
        method: ImportMethod,
        raw_artifact_key: Option<String>,
    ) -> Result<ImportJob, AppError> {
        let now = Utc::now();
        let id = Uuid::now_v7();
        let row = sqlx::query_as!(
            JobRow,
            r#"INSERT INTO import_jobs
                (id, user_id, import_source, import_method, status, raw_artifact_key, created_at)
               VALUES ($1, $2, $3, $4, 'awaiting_provider', $5, $6)
               RETURNING id, user_id, import_source, import_method, status,
                         imported_count, updated_count, duplicate_count,
                         skipped_private_count, failed_count, raw_artifact_key,
                         provider_report, error, created_at, started_at, finished_at"#,
            id,
            user_id.into_uuid(),
            import_source_to_str(source),
            import_method_to_str(method),
            raw_artifact_key,
            now,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        ImportJob::try_from(row)
    }

    async fn find_by_id(
        &self,
        user_id: UserId,
        id: ImportJobId,
    ) -> Result<Option<ImportJob>, AppError> {
        let row = sqlx::query_as!(
            JobRow,
            r#"SELECT id, user_id, import_source, import_method, status,
                      imported_count, updated_count, duplicate_count,
                      skipped_private_count, failed_count, raw_artifact_key,
                      provider_report, error, created_at, started_at, finished_at
               FROM import_jobs
               WHERE id = $1 AND user_id = $2"#,
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(ImportJob::try_from).transpose()
    }

    async fn find_by_id_unchecked(&self, id: ImportJobId) -> Result<Option<ImportJob>, AppError> {
        let row = sqlx::query_as!(
            JobRow,
            r#"SELECT id, user_id, import_source, import_method, status,
                      imported_count, updated_count, duplicate_count,
                      skipped_private_count, failed_count, raw_artifact_key,
                      provider_report, error, created_at, started_at, finished_at
               FROM import_jobs
               WHERE id = $1"#,
            id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(ImportJob::try_from).transpose()
    }

    async fn list_by_user(
        &self,
        user_id: UserId,
        limit: i64,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<ImportJob>, AppError> {
        let rows = sqlx::query_as!(
            JobRow,
            r#"SELECT id, user_id, import_source, import_method, status,
                      imported_count, updated_count, duplicate_count,
                      skipped_private_count, failed_count, raw_artifact_key,
                      provider_report, error, created_at, started_at, finished_at
               FROM import_jobs
               WHERE user_id = $1 AND ($2::timestamptz IS NULL OR created_at < $2)
               ORDER BY created_at DESC
               LIMIT $3"#,
            user_id.into_uuid(),
            before,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        rows.into_iter().map(ImportJob::try_from).collect()
    }

    async fn set_raw_artifact_key(
        &self,
        user_id: UserId,
        id: ImportJobId,
        raw_artifact_key: String,
    ) -> Result<ImportJob, AppError> {
        let row = sqlx::query_as!(
            JobRow,
            r#"UPDATE import_jobs
               SET raw_artifact_key = $3
               WHERE id = $1 AND user_id = $2
               RETURNING id, user_id, import_source, import_method, status,
                         imported_count, updated_count, duplicate_count,
                         skipped_private_count, failed_count, raw_artifact_key,
                         provider_report, error, created_at, started_at, finished_at"#,
            id.into_uuid(),
            user_id.into_uuid(),
            raw_artifact_key,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "import_job",
                id: id.to_string(),
            })
        })?;

        ImportJob::try_from(row)
    }

    async fn mark_started(&self, id: ImportJobId) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"UPDATE import_jobs
               SET status = 'running', started_at = now()
               WHERE id = $1"#,
            id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "import_job",
                id: id.to_string(),
            }));
        }
        Ok(())
    }

    async fn mark_finished(
        &self,
        id: ImportJobId,
        status: ImportJobStatus,
        error: Option<String>,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"UPDATE import_jobs
               SET status = $2, finished_at = now(), error = $3
               WHERE id = $1"#,
            id.into_uuid(),
            status.as_str(),
            error.as_deref(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "import_job",
                id: id.to_string(),
            }));
        }
        Ok(())
    }

    async fn rollback_imported_library_entries(
        &self,
        user_id: UserId,
        id: ImportJobId,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_err)?;

        let external_ids = sqlx::query!(
            r#"SELECT external_id
               FROM import_job_items
               WHERE import_job_id = $1
                 AND outcome = 'imported'"#,
            id.into_uuid(),
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(map_err)?
        .into_iter()
        .map(|row| row.external_id)
        .collect::<Vec<_>>();

        if !external_ids.is_empty() {
            let origin_ids = external_ids
                .iter()
                .map(|external_id| {
                    deterministic_origin_id(
                        DocumentOriginType::ReadwiseImportItem,
                        user_id,
                        &format!("readwise:{external_id}"),
                    )
                })
                .collect::<Vec<_>>();

            sqlx::query!(
                r#"UPDATE library_entries le
                   SET deleted_at = now(),
                       updated_at = now()
                   WHERE le.user_id = $1
                     AND le.deleted_at IS NULL
                     AND EXISTS (
                         SELECT 1
                         FROM document_origins origin
                         WHERE origin.user_id = le.user_id
                           AND origin.document_id = le.document_id
                           AND origin.origin_type = 'readwise_import_item'
                           AND origin.origin_id = ANY($2)
                     )"#,
                user_id.into_uuid(),
                &origin_ids,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
        }

        sqlx::query!(
            r#"UPDATE import_jobs
               SET status = 'rolled_back', finished_at = now()
               WHERE id = $1 AND user_id = $2"#,
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;

        tx.commit().await.map_err(map_err)?;
        Ok(())
    }

    async fn increment_counts(
        &self,
        id: ImportJobId,
        delta: ImportJobCountsDelta,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"UPDATE import_jobs
               SET imported_count = imported_count + $2,
                   updated_count = updated_count + $3,
                   duplicate_count = duplicate_count + $4,
                   skipped_private_count = skipped_private_count + $5,
                   failed_count = failed_count + $6
               WHERE id = $1"#,
            id.into_uuid(),
            delta.imported,
            delta.updated,
            delta.duplicate,
            delta.skipped_private,
            delta.failed,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn append_item_outcome(
        &self,
        import_job_id: ImportJobId,
        external_id: &str,
        outcome: ImportItemOutcome,
        error: Option<String>,
        diagnostics: Option<serde_json::Value>,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        let id = Uuid::now_v7();
        sqlx::query!(
            r#"INSERT INTO import_job_items
                (id, import_job_id, external_id, outcome, error, diagnostics, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            id,
            import_job_id.into_uuid(),
            external_id,
            outcome.as_str(),
            error.as_deref(),
            diagnostics,
            now,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn list_item_outcomes(
        &self,
        import_job_id: ImportJobId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ImportJobItem>, AppError> {
        let rows = sqlx::query_as!(
            ItemOutcomeRow,
            r#"SELECT
                   iji.id,
                   iji.import_job_id,
                   iji.external_id,
                   NULL::text AS "title?",
                   iji.outcome,
                   iji.error,
                   iji.created_at
               FROM import_job_items iji
               WHERE iji.import_job_id = $1
               ORDER BY iji.created_at ASC
               LIMIT $2 OFFSET $3"#,
            import_job_id.into_uuid(),
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        rows.into_iter().map(ImportJobItem::try_from).collect()
    }

    async fn set_provider_report(
        &self,
        id: ImportJobId,
        report: serde_json::Value,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"UPDATE import_jobs
               SET provider_report = $2
               WHERE id = $1"#,
            id.into_uuid(),
            report,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn find_item_outcome_by_external_id(
        &self,
        import_job_id: ImportJobId,
        external_id: &str,
    ) -> Result<Option<ImportJobItem>, AppError> {
        let mut matches = Vec::new();
        let mut offset = 0;
        const PAGE_SIZE: i64 = 1_000;

        loop {
            let page = self
                .list_item_outcomes(import_job_id, PAGE_SIZE, offset)
                .await?;
            if page.is_empty() {
                break;
            }

            matches.extend(
                page.into_iter()
                    .filter(|outcome| outcome.external_id == external_id),
            );
            offset += PAGE_SIZE;
        }

        matches.sort_by_key(|outcome| (outcome.created_at, outcome.id));
        Ok(matches.pop())
    }
}
