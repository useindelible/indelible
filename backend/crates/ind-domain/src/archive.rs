use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{ArchiveAssetId, DocumentId, ItemNoteId, UserId};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveAssetStatus {
    Pending,
    #[default]
    Completed,
    Degraded,
    Failed,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveAssetKind {
    ReadableHtml,
    Monolith,
    Pdf,
    Screenshot,
    Thumbnail,
    Warc,
    Epub,
    OriginalUpload,
    ExtractedText,
    /// Raw provider HTML for emails, stored alongside readable_html so users can
    /// toggle between Reader view and Original view.
    OriginalHtml,
    /// Derived article table of contents (JSON payload at a content-addressed key).
    /// Row existence means "computed" — a stored `status: none` payload is a terminal
    /// result, so headingless documents are never recomputed.
    ArticleToc,
}

impl fmt::Display for ArchiveAssetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ReadableHtml => "readable_html",
            Self::Monolith => "monolith",
            Self::Pdf => "pdf",
            Self::Screenshot => "screenshot",
            Self::Thumbnail => "thumbnail",
            Self::Warc => "warc",
            Self::Epub => "epub",
            Self::OriginalUpload => "original_upload",
            Self::ExtractedText => "extracted_text",
            Self::OriginalHtml => "original_html",
            Self::ArticleToc => "article_toc",
        };
        f.write_str(s)
    }
}

impl FromStr for ArchiveAssetKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "readable_html" => Ok(Self::ReadableHtml),
            "monolith" => Ok(Self::Monolith),
            "pdf" => Ok(Self::Pdf),
            "screenshot" => Ok(Self::Screenshot),
            "thumbnail" => Ok(Self::Thumbnail),
            "warc" => Ok(Self::Warc),
            "epub" => Ok(Self::Epub),
            "original_upload" => Ok(Self::OriginalUpload),
            "extracted_text" => Ok(Self::ExtractedText),
            "original_html" => Ok(Self::OriginalHtml),
            "article_toc" => Ok(Self::ArticleToc),
            other => Err(format!("unknown asset kind: {other}")),
        }
    }
}

/// A rendered asset owned by a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentAsset {
    pub id: ArchiveAssetId,
    pub document_id: DocumentId,
    pub asset_kind: ArchiveAssetKind,
    pub s3_key: String,
    pub s3_bucket: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub status: ArchiveAssetStatus,
    pub failed_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Write payload for a document-keyed archive asset (id and created_at are assigned
/// by the repository).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDocumentAsset {
    pub document_id: DocumentId,
    pub asset_kind: ArchiveAssetKind,
    pub s3_key: String,
    pub s3_bucket: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub status: ArchiveAssetStatus,
    pub failed_reason: Option<String>,
}

/// The single per-document note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentNote {
    pub id: ItemNoteId,
    pub document_id: DocumentId,
    pub user_id: UserId,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::ArchiveAssetKind;

    #[test]
    fn asset_kind_display_from_str_round_trips() {
        let kinds = [
            ArchiveAssetKind::ReadableHtml,
            ArchiveAssetKind::Monolith,
            ArchiveAssetKind::Pdf,
            ArchiveAssetKind::Screenshot,
            ArchiveAssetKind::Thumbnail,
            ArchiveAssetKind::Warc,
            ArchiveAssetKind::Epub,
            ArchiveAssetKind::OriginalUpload,
            ArchiveAssetKind::ExtractedText,
            ArchiveAssetKind::OriginalHtml,
            ArchiveAssetKind::ArticleToc,
        ];
        for kind in kinds {
            assert_eq!(ArchiveAssetKind::from_str(&kind.to_string()), Ok(kind));
        }
        assert_eq!(ArchiveAssetKind::ArticleToc.to_string(), "article_toc");
    }
}
