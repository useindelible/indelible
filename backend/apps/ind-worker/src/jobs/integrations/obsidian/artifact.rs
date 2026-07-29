use std::io::{Cursor, Write};

use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_domain::{DomainError, HighlightId, LibraryEntryId};
use ind_integrations::obsidian::ObsidianArtifactEntry;
use ind_persistence::repos::ObsidianSyncArtifactItemInsert;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct ObsidianArtifactManifest {
    pub(super) version: u32,
    pub(super) run_id: uuid::Uuid,
    pub(super) generated_at: DateTime<Utc>,
    pub(super) entries: Vec<ObsidianArtifactEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) sync_notification: Option<ObsidianSyncNotificationArtifact>,
}

#[derive(Debug, Serialize)]
pub(super) struct ObsidianSyncNotificationArtifact {
    pub(super) file_path: String,
    pub(super) append_content: String,
}

pub(super) fn build_zip_artifact(manifest: &ObsidianArtifactManifest) -> Result<Vec<u8>, AppError> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let bytes =
        serde_json::to_vec_pretty(manifest).map_err(|e| AppError::Repository(Box::new(e)))?;
    zip.start_file("artifact.json", options)
        .map_err(|e| AppError::Repository(Box::new(e)))?;
    zip.write_all(&bytes)
        .map_err(|e| AppError::Repository(Box::new(e)))?;
    let cursor = zip
        .finish()
        .map_err(|e| AppError::Repository(Box::new(e)))?;
    Ok(cursor.into_inner())
}

pub(super) fn artifact_item_inserts_from_entries(
    entries: &[ObsidianArtifactEntry],
) -> Result<Vec<ObsidianSyncArtifactItemInsert>, AppError> {
    entries
        .iter()
        .map(|entry| {
            Ok(ObsidianSyncArtifactItemInsert {
                library_entry_id: parse_subject_uuid(&entry.subject_id)?,
                file_path: entry.file_path.clone(),
                full_document_path: entry.full_document_text_path.clone(),
                last_highlight_created_at: entry.last_highlight_created_at,
                last_highlight_id: entry
                    .last_highlight_id
                    .as_deref()
                    .map(parse_highlight_uuid)
                    .transpose()?,
                last_content_hash: entry.last_content_hash.clone(),
                last_full_document_hash: entry.last_full_document_hash.clone(),
            })
        })
        .collect()
}

fn parse_subject_uuid(raw: &str) -> Result<uuid::Uuid, AppError> {
    raw.parse::<LibraryEntryId>()
        .map(|id| id.into_uuid())
        .or_else(|_| uuid::Uuid::parse_str(raw))
        .map_err(|e| {
            AppError::Domain(DomainError::Validation {
                field: "subject_id".to_string(),
                message: e.to_string(),
            })
        })
}

fn parse_highlight_uuid(raw: &str) -> Result<uuid::Uuid, AppError> {
    raw.parse::<HighlightId>()
        .map(|id| id.into_uuid())
        .or_else(|_| uuid::Uuid::parse_str(raw))
        .map_err(|e| {
            AppError::Domain(DomainError::Validation {
                field: "highlight_id".to_string(),
                message: e.to_string(),
            })
        })
}
