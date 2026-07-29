use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    ContentSource, DocumentId, DocumentType, HighlightId, ImportJobId, IntegrationConnectionId,
    LibraryEntryId, UserId,
};

// Notion API version header value used by both ind-integrations
// (worker HTTP client) and ind-auth (OAuth callback). Bump here once.
pub const NOTION_API_VERSION: &str = "2026-03-11";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationProvider {
    Obsidian,
    Notion,
    Logseq,
    BrowserExtension,
    EmailIngest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationOAuthProvider {
    Notion,
}

impl IntegrationOAuthProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Notion => "notion",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportMethod {
    Oauth,
    Csv,
    Zip,
}

impl ImportMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Oauth => "oauth",
            Self::Csv => "csv",
            Self::Zip => "zip",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportJobStatus {
    AwaitingProvider,
    Pending,
    Running,
    Completed,
    Failed,
    Partial,
    RolledBack,
}

impl ImportJobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AwaitingProvider => "awaiting_provider",
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Partial => "partial",
            Self::RolledBack => "rolled_back",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportItemOutcome {
    Imported,
    Updated,
    Duplicate,
    SkippedPrivate,
    Failed,
}

impl ImportItemOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Imported => "imported",
            Self::Updated => "updated",
            Self::Duplicate => "duplicate",
            Self::SkippedPrivate => "skipped_private",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConnection {
    pub id: IntegrationConnectionId,
    pub user_id: UserId,
    pub provider: IntegrationProvider,
    pub config: serde_json::Value,
    pub status: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Optimistic-locking version. Incremented every time `config` is
    /// mutated through `update_config_with_version`. Settings PATCH
    /// handlers read this alongside the rest of the connection and pass
    /// it back so concurrent writes race-lose with `Conflict` instead of
    /// silently overwriting each other.
    #[serde(default)]
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionExportSettings {
    pub export_automatically: bool,
    pub include_highlight_locations: bool,
    pub compact_layout: bool,
    pub selection_enabled: bool,
}

impl Default for NotionExportSettings {
    fn default() -> Self {
        Self {
            export_automatically: true,
            include_highlight_locations: true,
            compact_layout: true,
            selection_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianExportSettings {
    #[serde(default = "default_true")]
    pub group_files_in_category_folders: bool,
    #[serde(default)]
    pub export_all_reader_documents: bool,
    #[serde(default)]
    pub sync_notifications: bool,
    #[serde(default)]
    pub properties_template: Option<String>,
    #[serde(default = "default_page_title_template")]
    pub page_title_template: String,
    #[serde(default = "default_metadata_template")]
    pub metadata_template: String,
    #[serde(default = "default_highlight_header_template")]
    pub highlight_header_template: String,
    #[serde(default = "default_highlight_template")]
    pub highlight_template: String,
    #[serde(default)]
    pub file_name_template: Option<String>,
    #[serde(default)]
    pub category_folder_templates: HashMap<String, String>,
    #[serde(default = "default_sync_notification_template")]
    pub sync_notification_template: String,
}

impl Default for ObsidianExportSettings {
    fn default() -> Self {
        Self {
            group_files_in_category_folders: true,
            export_all_reader_documents: false,
            sync_notifications: false,
            properties_template: None,
            page_title_template: default_page_title_template(),
            metadata_template: default_metadata_template(),
            highlight_header_template: default_highlight_header_template(),
            highlight_template: default_highlight_template(),
            file_name_template: None,
            category_folder_templates: HashMap::new(),
            sync_notification_template: default_sync_notification_template(),
        }
    }
}

fn default_true() -> bool {
    true
}

pub fn default_page_title_template() -> String {
    String::new()
}

pub fn default_metadata_template() -> String {
    r#"{% if image_url -%}
![cover]({{image_url}})
{% endif -%}
## Metadata
{% if author -%}
- Author: [[{{author}}]]
{% endif -%}
- Full Title: {{full_title}}
- Category: #{{category}}
{% if document_tags -%}
- Document Tags: {% for tag in document_tags %}[[{{tag}}]] {% endfor %}
{% endif -%}
{% if url -%}
- URL: {{url}}
{% endif -%}
{% if summary -%}
- Summary: {{summary}}
{% endif -%}"#
        .to_string()
}

pub fn default_highlight_header_template() -> String {
    r#"{% if is_new_page %}
## Highlights
{% elif has_new_highlights -%}
## New highlights added {{date}} at {{time}}
{% endif -%}"#
        .to_string()
}

pub fn default_highlight_template() -> String {
    r#"- {{ highlight_text }}{% if highlight_location and highlight_location_url %} ([{{highlight_location}}]({{highlight_location_url}})){% elif highlight_location %} ({{highlight_location}}){% endif %}{% if highlight_tags %}
    - Tags: {% for tag in highlight_tags %}[[{{tag}}]] {% endfor %}{% endif %}{% if highlight_note %}
    - Note: {{ highlight_note }}{% endif %}"#
        .to_string()
}

pub fn default_sync_notification_template() -> String {
    "- {{date}} {{time}}: Synced {{document_count}} documents".to_string()
}

/// A saved Library entry offered in the Notion export picker (TASK-236). Keyed on the
/// `library_entry_id` (selectable + cursor key) with its backing `document_id`; only saved Library
/// content is enumerable, so the legacy "explicitly-saved feed item" filter is unnecessary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionExportItem {
    pub library_entry_id: LibraryEntryId,
    pub document_id: DocumentId,
    pub title: String,
    pub url: Option<String>,
    pub document_type: DocumentType,
    pub source: ContentSource,
    pub selected: bool,
    pub exported_page_id: Option<String>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationOAuthToken {
    pub id: uuid::Uuid,
    pub user_id: UserId,
    pub provider: IntegrationOAuthProvider,
    pub access_token_enc: Vec<u8>,
    pub refresh_token_enc: Option<Vec<u8>>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub extra: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJob {
    pub id: ImportJobId,
    pub user_id: UserId,
    pub import_source: crate::ImportSource,
    pub import_method: ImportMethod,
    pub status: ImportJobStatus,
    pub imported_count: i32,
    pub updated_count: i32,
    pub duplicate_count: i32,
    pub skipped_private_count: i32,
    pub failed_count: i32,
    pub raw_artifact_key: Option<String>,
    pub provider_report: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJobItem {
    pub id: uuid::Uuid,
    pub import_job_id: ImportJobId,
    pub external_id: String,
    pub title: Option<String>,
    pub outcome: ImportItemOutcome,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ImportJobCountsDelta {
    pub imported: i32,
    pub updated: i32,
    pub duplicate: i32,
    pub skipped_private: i32,
    pub failed: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportCursor {
    pub connection_id: IntegrationConnectionId,
    pub library_entry_id: LibraryEntryId,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_attempted_at: Option<DateTime<Utc>>,
    pub cursor_version: i32,
    pub last_error: Option<String>,
    pub remote_page_id: Option<String>,
    pub last_exported_highlight_created_at: Option<DateTime<Utc>>,
    pub last_exported_highlight_id: Option<HighlightId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
