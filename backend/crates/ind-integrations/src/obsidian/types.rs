use chrono::{DateTime, Utc};
use ind_domain::ItemType;
use serde::{Deserialize, Serialize};

pub struct ObsidianRenderDocument {
    pub subject_id: String,
    pub subject_kind: String,
    pub title: String,
    pub full_title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub item_type: ItemType,
    pub image_url: Option<String>,
    pub summary: Option<String>,
    pub full_document_text: Option<String>,
    pub document_tags: Vec<String>,
    pub highlights: Vec<ObsidianRenderHighlight>,
}

#[derive(Debug, Clone)]
pub struct ObsidianRenderHighlight {
    pub id: String,
    pub text: String,
    pub note: Option<String>,
    pub color: String,
    pub tags: Vec<String>,
    pub location: Option<String>,
    pub location_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct ObsidianRenderCursor {
    pub has_delivered: bool,
    pub last_highlight_created_at: Option<DateTime<Utc>>,
    pub last_highlight_id: Option<String>,
    pub force_full: bool,
    pub last_content_hash: Option<String>,
    pub last_full_document_hash: Option<String>,
    pub generated_path: Option<String>,
    pub generated_full_document_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianArtifactEntry {
    pub subject_id: String,
    pub subject_kind: String,
    pub book_id: String,
    pub file_path: String,
    pub full_content: Option<String>,
    pub append_only_content: Option<String>,
    pub last_content_hash: Option<String>,
    pub last_highlight_created_at: Option<DateTime<Utc>>,
    pub last_highlight_id: Option<String>,
    pub full_document_text_path: Option<String>,
    pub full_document_text: Option<String>,
    pub last_full_document_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RenderedObsidianDocument {
    pub entry: ObsidianArtifactEntry,
}

#[derive(Debug, thiserror::Error)]
pub enum ObsidianRenderError {
    #[error("failed to render template `{name}`: {source}")]
    Template {
        name: &'static str,
        source: minijinja::Error,
    },
}
