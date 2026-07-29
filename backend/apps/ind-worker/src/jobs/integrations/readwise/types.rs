use ind_application::AppError;
use ind_domain::ImportJobId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ReadwiseImportJob {
    pub import_job_id: ImportJobId,
}

#[derive(Debug, Deserialize)]
pub(super) struct ArtifactKeys {
    #[serde(default)]
    pub(super) csv_key: Option<String>,
    #[serde(default)]
    pub(super) zip_key: Option<String>,
    #[serde(default)]
    pub(super) opml_key: Option<String>,
}

#[derive(Debug)]
pub(super) struct ReadwiseCsvRow {
    pub(super) title: String,
    pub(super) url: Option<String>,
    pub(super) id: String,
    pub(super) document_tags: String,
    pub(super) saved_date: String,
    pub(super) reading_progress: f32,
    pub(super) location: String,
    pub(super) seen: bool,
}

#[derive(Debug)]
pub(super) struct ZipEntry {
    pub(super) path: String,
    pub(super) title: String,
    pub(super) extension: String,
    pub(super) bytes: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub(super) struct ReadwiseReport {
    pub(super) csv_rows: u32,
    pub(super) highlight_rows: u32,
    pub(super) reading_progress_rows: u32,
    pub(super) zip_files_total: u32,
    pub(super) zip_files_matched: u32,
    pub(super) zip_files_unmatched: u32,
    pub(super) unmatched_zip_assets: Vec<String>,
    pub(super) archive_assets_imported: u32,
    pub(super) search_reindex_jobs_enqueued: u32,
    pub(super) embedding_jobs_enqueued: u32,
    pub(super) opml_feeds_created: u32,
    pub(super) opml_feeds_skipped: u32,
    pub(super) opml_errors: Vec<String>,
}

pub(super) enum ProcessCsvOutcome {
    /// A fresh save (new or restored library entry) was created via `save_to_library`.
    Imported {
        search_reindex_jobs: u32,
        embedding_jobs: u32,
        diagnostics: Option<serde_json::Value>,
    },
    /// The content already had an active library entry (re-import or saved elsewhere).
    Duplicate {
        search_reindex_jobs: u32,
        embedding_jobs: u32,
        diagnostics: Option<serde_json::Value>,
    },
    /// A `private://` Readwise row (uploaded book / EPUB) that has no matching
    /// ZIP asset — without the asset we have nothing to import, so we skip.
    SkippedPrivate,
}

/// Per-row Readwise import provenance/diagnostics (TASK-241), recorded on `import_job_items`.
/// Serialized sparsely: absent/empty/false fields are omitted, and an all-empty record yields
/// `None` (SQL NULL) so clean URL rows carry no diagnostics.
#[derive(Debug, Default, Clone)]
pub(super) struct RowDiagnostics {
    pub(super) zip_path: Option<String>,
    pub(super) zip_only: bool,
    pub(super) tag_parse_errors: Vec<String>,
}

impl RowDiagnostics {
    pub(super) fn to_value(&self) -> Option<serde_json::Value> {
        let mut map = serde_json::Map::new();
        if let Some(zip_path) = &self.zip_path {
            map.insert(
                "zip_path".into(),
                serde_json::Value::String(zip_path.clone()),
            );
        }
        if self.zip_only {
            map.insert("zip_only".into(), serde_json::Value::Bool(true));
        }
        if !self.tag_parse_errors.is_empty() {
            map.insert(
                "tag_parse_errors".into(),
                serde_json::Value::Array(
                    self.tag_parse_errors
                        .iter()
                        .cloned()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
        }
        (!map.is_empty()).then_some(serde_json::Value::Object(map))
    }
}

pub(super) enum ProcessRowResult {
    Failed(AppError),
}

impl From<AppError> for ProcessRowResult {
    fn from(e: AppError) -> Self {
        Self::Failed(e)
    }
}
