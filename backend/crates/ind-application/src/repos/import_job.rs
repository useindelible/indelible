use chrono::{DateTime, Utc};

use crate::error::AppError;
use ind_domain::{
    ImportItemOutcome, ImportJob, ImportJobCountsDelta, ImportJobId, ImportJobItem,
    ImportJobStatus, ImportMethod, ImportSource, UserId,
};

#[async_trait::async_trait]
pub trait ImportJobRepository: Send + Sync {
    async fn create(
        &self,
        user_id: UserId,
        source: ImportSource,
        method: ImportMethod,
        raw_artifact_key: Option<String>,
    ) -> Result<ImportJob, AppError>;

    async fn find_by_id(
        &self,
        user_id: UserId,
        id: ImportJobId,
    ) -> Result<Option<ImportJob>, AppError>;

    async fn list_by_user(
        &self,
        user_id: UserId,
        limit: i64,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<ImportJob>, AppError>;

    async fn set_raw_artifact_key(
        &self,
        user_id: UserId,
        id: ImportJobId,
        raw_artifact_key: String,
    ) -> Result<ImportJob, AppError>;

    async fn mark_started(&self, id: ImportJobId) -> Result<(), AppError>;

    async fn mark_finished(
        &self,
        id: ImportJobId,
        status: ImportJobStatus,
        error: Option<String>,
    ) -> Result<(), AppError>;

    async fn rollback_imported_library_entries(
        &self,
        user_id: UserId,
        id: ImportJobId,
    ) -> Result<(), AppError>;

    async fn increment_counts(
        &self,
        id: ImportJobId,
        delta: ImportJobCountsDelta,
    ) -> Result<(), AppError>;

    async fn append_item_outcome(
        &self,
        import_job_id: ImportJobId,
        external_id: &str,
        outcome: ImportItemOutcome,
        error: Option<String>,
        diagnostics: Option<serde_json::Value>,
    ) -> Result<(), AppError>;

    async fn list_item_outcomes(
        &self,
        import_job_id: ImportJobId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ImportJobItem>, AppError>;

    async fn set_provider_report(
        &self,
        id: ImportJobId,
        report: serde_json::Value,
    ) -> Result<(), AppError>;

    async fn find_by_id_unchecked(&self, id: ImportJobId) -> Result<Option<ImportJob>, AppError>;

    /// Look up a single per-row outcome by `(import_job_id, external_id)`.
    /// Used by the import handler to detect whether a row was already
    /// successfully processed on a previous attempt, avoiding double-counting
    /// on worker retry.
    async fn find_item_outcome_by_external_id(
        &self,
        import_job_id: ImportJobId,
        external_id: &str,
    ) -> Result<Option<ImportJobItem>, AppError>;
}
