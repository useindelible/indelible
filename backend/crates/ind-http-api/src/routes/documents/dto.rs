use chrono::{DateTime, Utc};
use ind_application::ports::DocumentReaderView;
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, DocumentId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::extract::Validate;

pub(crate) fn parse_document_id(raw: &str) -> Result<DocumentId, ApiError> {
    raw.parse().map_err(|_| ApiError::NotFound {
        entity: "Document",
        id: raw.to_string(),
    })
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DocumentUpsertNoteBody {
    pub body: String,
}

impl Validate for DocumentUpsertNoteBody {
    fn validate(&self) -> Result<(), Vec<crate::error::FieldError>> {
        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentNoteResponse {
    pub id: String,
    pub body: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, validator::Validate, ToSchema)]
pub struct UpdateDocumentProgressBody {
    #[validate(range(min = 0.0, max = 100.0))]
    pub progress_percent: f32,
    pub chapter_locator: Option<String>,
    pub chapter_offset: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentReprocessResponse {
    pub queued: bool,
    pub job_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<u64>,
}

/// Reader read-model. `library_entry_id`/`saved` distinguish a prepared-but-unsaved document
/// from a saved Library entry; `readable_ready` tells the client whether the readable asset has
/// landed (the on-tap render is async, so a freshly opened document polls until ready).
#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentReaderResponse {
    pub document_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub document_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_entry_id: Option<String>,
    pub saved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_progress_percent: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_locator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_read_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub finished_at: Option<DateTime<Utc>>,
    pub available_assets: Vec<String>,
    pub assets: Vec<DocumentReaderAssetResponse>,
    pub readable_ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    /// Resolved short summary. Returns the stored AI summary when present,
    /// falling back to `excerpt`. `null` when neither is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading_time_minutes: Option<i32>,
}

impl DocumentReaderResponse {
    pub fn from_view(
        view: DocumentReaderView,
        asset_base_url: &str,
        summary: Option<String>,
    ) -> Self {
        let readable_ready = view.assets.iter().any(|asset| {
            asset.asset_kind == ArchiveAssetKind::ReadableHtml
                && asset.status == ArchiveAssetStatus::Completed
        });
        let available_assets = view
            .assets
            .iter()
            .filter(|asset| {
                asset.status == ArchiveAssetStatus::Completed && !asset.s3_key.trim().is_empty()
            })
            .map(|asset| asset.asset_kind.to_string())
            .collect();
        let assets = view
            .assets
            .iter()
            .map(DocumentReaderAssetResponse::from)
            .collect();
        let thumbnail_url = view.document.thumbnail_url.or_else(|| {
            view.assets
                .iter()
                .any(|asset| {
                    asset.asset_kind == ArchiveAssetKind::Thumbnail
                        && asset.status == ArchiveAssetStatus::Completed
                })
                .then(|| {
                    crate::routes::asset_urls::document_asset_url(
                        asset_base_url,
                        view.document.id,
                        ArchiveAssetKind::Thumbnail,
                    )
                })
        });
        let state = view.state;
        Self {
            document_id: view.document.id.to_string(),
            title: view.document.title,
            url: view.document.canonical_url.or(view.document.original_url),
            document_type: view.document.document_type.as_str().to_string(),
            library_entry_id: view.library_entry_id.map(|id| id.to_string()),
            saved: view.library_entry_id.is_some(),
            progress_percent: state.as_ref().and_then(|s| s.progress_percent),
            max_progress_percent: state.as_ref().and_then(|s| s.max_progress_percent),
            chapter_locator: state.as_ref().and_then(|s| s.chapter_locator.clone()),
            chapter_offset: state.as_ref().and_then(|s| s.chapter_offset),
            last_read_at: state.as_ref().and_then(|s| s.last_read_at),
            finished_at: state.as_ref().and_then(|s| s.finished_at),
            available_assets,
            assets,
            readable_ready,
            domain: view.document.domain,
            author: view.document.author,
            excerpt: view.document.excerpt,
            summary,
            published_at: view.document.published_at,
            language: view.document.language,
            lead_image_url: view.document.lead_image_url,
            thumbnail_url,
            word_count: view.document.word_count,
            reading_time_minutes: view.document.reading_time_minutes,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentReaderAssetResponse {
    pub id: String,
    pub asset_kind: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_reason: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl From<&ind_domain::DocumentAsset> for DocumentReaderAssetResponse {
    fn from(asset: &ind_domain::DocumentAsset) -> Self {
        Self {
            id: asset.id.to_string(),
            asset_kind: asset.asset_kind.to_string(),
            content_type: asset.content_type.clone(),
            size_bytes: asset.size_bytes,
            status: status_str(asset.status).to_string(),
            failed_reason: asset.failed_reason.clone(),
            created_at: asset.created_at,
        }
    }
}

/// Document asset metadata plus an API-origin download URL for its bytes (the
/// reader fetches the readable HTML from this URL; the asset proxy behind it
/// streams or redirects depending on `asset_serving_mode`).
#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentAssetResponse {
    pub id: String,
    pub object: &'static str,
    pub document_id: String,
    pub asset_kind: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub status: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub download_url: String,
}

impl DocumentAssetResponse {
    pub fn from_asset(asset: ind_domain::DocumentAsset, base_url: &str) -> Self {
        let download_url = crate::routes::asset_urls::document_asset_url(
            base_url,
            asset.document_id,
            asset.asset_kind,
        );
        Self {
            id: asset.id.to_string(),
            object: "document_asset",
            document_id: asset.document_id.to_string(),
            asset_kind: asset.asset_kind.to_string(),
            content_type: asset.content_type,
            size_bytes: asset.size_bytes,
            status: status_str(asset.status).to_string(),
            created_at: asset.created_at,
            download_url,
        }
    }
}

fn status_str(status: ArchiveAssetStatus) -> &'static str {
    match status {
        ArchiveAssetStatus::Pending => "pending",
        ArchiveAssetStatus::Completed => "completed",
        ArchiveAssetStatus::Degraded => "degraded",
        ArchiveAssetStatus::Failed => "failed",
        ArchiveAssetStatus::Unsupported => "unsupported",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArticleTocResponseStatus {
    Ready,
    None,
    Pending,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ArticleTocEntryResponse {
    /// Document-order heading ordinal (pre-dedupe): the positional fallback
    /// when a cached article body predates anchor ids.
    pub source_heading_index: u32,
    /// Anchor id present on the heading element in the stored readable HTML.
    pub id: String,
    pub title: String,
    /// Relative outline depth (0 = top level), normalized from tag levels.
    pub depth: u8,
    /// Words in this entry's own section; clients derive minutes at 238 WPM.
    pub word_count: u32,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ArticleTocResponse {
    pub status: ArticleTocResponseStatus,
    /// True when the outline was capped at the entry limit.
    pub truncated: bool,
    /// Empty unless `status` is `ready`.
    pub entries: Vec<ArticleTocEntryResponse>,
}
