use chrono::{DateTime, Utc};
use ind_domain::{ImportItemOutcome, ImportJob, ImportJobItem};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use ind_application::outputs::import::ImportStatusOutput;

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportUploadResponse {
    pub import_job_id: String,
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportJobCountsDto {
    pub imported: u32,
    pub updated: u32,
    pub duplicate: u32,
    pub skipped_private: u32,
    pub failed: u32,
}

impl ImportJobCountsDto {
    fn from_job(job: &ImportJob) -> Self {
        Self {
            imported: job.imported_count.max(0) as u32,
            updated: job.updated_count.max(0) as u32,
            duplicate: job.duplicate_count.max(0) as u32,
            skipped_private: job.skipped_private_count.max(0) as u32,
            failed: job.failed_count.max(0) as u32,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportJobItemOutcomeDto {
    pub external_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<ImportJobItem> for ImportJobItemOutcomeDto {
    fn from(item: ImportJobItem) -> Self {
        Self {
            external_id: item.external_id,
            title: item.title,
            outcome: outcome_to_str(item.outcome).to_string(),
            error: item.error,
        }
    }
}

fn outcome_to_str(outcome: ImportItemOutcome) -> &'static str {
    match outcome {
        ImportItemOutcome::Imported => "imported",
        ImportItemOutcome::Updated => "updated",
        ImportItemOutcome::Duplicate => "duplicate",
        ImportItemOutcome::SkippedPrivate => "skipped_private",
        ImportItemOutcome::Failed => "failed",
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReadwiseImportReportDto {
    #[serde(default)]
    pub csv_rows: u32,
    #[serde(default)]
    pub reading_progress_rows: u32,
    #[serde(default)]
    pub zip_files_total: u32,
    #[serde(default)]
    pub zip_files_matched: u32,
    #[serde(default)]
    pub zip_files_unmatched: u32,
    #[serde(default)]
    pub unmatched_zip_assets: Vec<String>,
    #[serde(default)]
    pub archive_assets_imported: u32,
    #[serde(default)]
    pub search_reindex_jobs_enqueued: u32,
    #[serde(default)]
    pub embedding_jobs_enqueued: u32,
    #[serde(default)]
    pub opml_feeds_created: u32,
    #[serde(default)]
    pub opml_feeds_skipped: u32,
    #[serde(default)]
    pub opml_errors: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportJobStatusResponse {
    pub id: String,
    pub import_source: String,
    pub import_method: String,
    pub status: String,
    pub counts: ImportJobCountsDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub finished_at: Option<DateTime<Utc>>,
    pub item_outcomes: Vec<ImportJobItemOutcomeDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readwise_report: Option<ReadwiseImportReportDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportJobListResponse {
    pub jobs: Vec<ImportJobStatusResponse>,
}

impl From<ImportStatusOutput> for ImportJobStatusResponse {
    fn from(output: ImportStatusOutput) -> Self {
        let ImportStatusOutput { job, items } = output;
        let counts = ImportJobCountsDto::from_job(&job);
        let readwise_report = job
            .provider_report
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok());
        Self {
            id: job.id.to_string(),
            import_source: job.import_source.as_str().to_string(),
            import_method: job.import_method.as_str().to_string(),
            status: job.status.as_str().to_string(),
            counts,
            error: job.error,
            created_at: job.created_at,
            started_at: job.started_at,
            finished_at: job.finished_at,
            item_outcomes: items
                .into_iter()
                .map(ImportJobItemOutcomeDto::from)
                .collect(),
            readwise_report,
        }
    }
}

pub fn project_import_status(output: ImportStatusOutput) -> ImportJobStatusResponse {
    ImportJobStatusResponse::from(output)
}
