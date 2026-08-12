use chrono::{DateTime, Utc};
use ind_domain::ObsidianExportSettings;
use ind_domain::{IntegrationConnection, IntegrationProvider};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, ToSchema)]
pub struct IntegrationListResponse {
    pub connections: Vec<IntegrationConnectionDto>,
    /// Lowercase ids of OAuth providers this instance holds credentials for.
    /// A provider absent here cannot be connected until an administrator
    /// configures it.
    #[schema(example = json!(["notion"]))]
    pub available_oauth_providers: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IntegrationConnectionDto {
    pub id: String,
    #[schema(value_type = String, example = "notion")]
    pub provider: IntegrationProvider,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_sync_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub config: IntegrationConnectionConfigDto,
    /// Count of queued integration jobs for this connection (sync + export).
    /// Frontend uses this to render a "pending" pill alongside connection status.
    pub pending_jobs: u32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl IntegrationConnectionDto {
    pub fn from_with_pending(c: IntegrationConnection, pending_jobs: u32) -> Self {
        let config = IntegrationConnectionConfigDto::from_domain(&c.provider, &c.config);
        Self {
            id: c.id.to_string(),
            provider: c.provider,
            status: c.status,
            last_sync_at: c.last_sync_at,
            last_error: c.last_error,
            config,
            pending_jobs,
            created_at: c.created_at,
        }
    }
}

impl From<IntegrationConnection> for IntegrationConnectionDto {
    fn from(c: IntegrationConnection) -> Self {
        Self::from_with_pending(c, 0)
    }
}

/// Provider-shaped configuration payload. Uses serde's tagged enum so the
/// generated OpenAPI schema produces a concrete discriminated union instead
/// of a bare object — mobile/web codegen can model it as a sealed class/union.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum IntegrationConnectionConfigDto {
    Obsidian {
        group_files_in_category_folders: bool,
        export_all_reader_documents: bool,
        sync_notifications: bool,
    },
    Notion {
        #[serde(skip_serializing_if = "Option::is_none")]
        workspace_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        workspace_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        workspace_icon: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        database_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data_source_id: Option<String>,
        export_automatically: bool,
        include_highlight_locations: bool,
        compact_layout: bool,
        selection_enabled: bool,
    },
    EmailIngest {
        address: String,
    },
    /// Catch-all for providers that don't have first-class structured config
    /// on this surface yet (currently Logseq, BrowserExtension). Kept as one
    /// variant so generated mobile/web codegen doesn't ship empty
    /// `LogseqConfig` / `BrowserExtensionConfig` types that can never be
    /// instantiated. When a provider gains structured config, add a dedicated
    /// variant and route to it from `from_domain`.
    Other {
        provider_name: String,
    },
}

impl IntegrationConnectionConfigDto {
    pub fn from_domain(
        provider: &IntegrationProvider,
        raw: &serde_json::Value,
    ) -> IntegrationConnectionConfigDto {
        match provider {
            IntegrationProvider::Obsidian => {
                let settings = ind_integrations::obsidian::settings_from_config(raw);
                IntegrationConnectionConfigDto::Obsidian {
                    group_files_in_category_folders: settings.group_files_in_category_folders,
                    export_all_reader_documents: settings.export_all_reader_documents,
                    sync_notifications: settings.sync_notifications,
                }
            }
            IntegrationProvider::Notion => IntegrationConnectionConfigDto::Notion {
                workspace_id: string_field(raw, "workspace_id"),
                workspace_name: string_field(raw, "workspace_name"),
                workspace_icon: string_field(raw, "workspace_icon"),
                database_id: string_field(raw, "database_id"),
                data_source_id: string_field(raw, "data_source_id"),
                export_automatically: bool_field(raw, "export_automatically", true),
                include_highlight_locations: bool_field(raw, "include_highlight_locations", true),
                compact_layout: bool_field(raw, "compact_layout", true),
                selection_enabled: bool_field(raw, "selection_enabled", false),
            },
            IntegrationProvider::EmailIngest => IntegrationConnectionConfigDto::EmailIngest {
                address: string_field(raw, "address").unwrap_or_default(),
            },
            IntegrationProvider::Logseq => IntegrationConnectionConfigDto::Other {
                provider_name: "logseq".to_string(),
            },
            IntegrationProvider::BrowserExtension => IntegrationConnectionConfigDto::Other {
                provider_name: "browser_extension".to_string(),
            },
        }
    }
}

fn string_field(raw: &serde_json::Value, key: &str) -> Option<String> {
    raw.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn bool_field(raw: &serde_json::Value, key: &str, default: bool) -> bool {
    raw.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NotionSettingsDto {
    pub export_automatically: bool,
    pub include_highlight_locations: bool,
    pub compact_layout: bool,
    pub selection_enabled: bool,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateNotionSettingsRequest {
    #[serde(default)]
    pub export_automatically: Option<bool>,
    #[serde(default)]
    pub include_highlight_locations: Option<bool>,
    #[serde(default)]
    pub compact_layout: Option<bool>,
    #[serde(default)]
    pub selection_enabled: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NotionExportItemDto {
    pub library_entry_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub item_type: String,
    pub selected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exported_page_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_synced_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

impl From<ind_domain::NotionExportItem> for NotionExportItemDto {
    fn from(value: ind_domain::NotionExportItem) -> Self {
        Self {
            library_entry_id: value.library_entry_id.to_string(),
            title: value.title,
            url: value.url,
            item_type: value.document_type.as_str().to_string(),
            selected: value.selected,
            exported_page_id: value.exported_page_id,
            last_synced_at: value.last_synced_at,
            last_error: value.last_error,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NotionExportItemsResponse {
    pub items: Vec<NotionExportItemDto>,
    pub total_count: i64,
    pub filtered_count: i64,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams, validator::Validate)]
pub struct ListNotionExportItemsQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default = "default_export_item_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_export_item_limit() -> i64 {
    50
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateNotionExportItemsRequest {
    #[schema(max_items = 200)]
    pub selections: Vec<NotionExportItemSelectionDto>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NotionExportItemSelectionDto {
    pub library_entry_id: String,
    pub selected: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NotionRefreshItemResponse {
    pub library_entry_id: String,
    pub job_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_page_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ObsidianSettingsDto {
    pub group_files_in_category_folders: bool,
    pub export_all_reader_documents: bool,
    pub sync_notifications: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties_template: Option<String>,
    pub page_title_template: String,
    pub metadata_template: String,
    pub highlight_header_template: String,
    pub highlight_template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name_template: Option<String>,
    pub category_folder_templates: std::collections::HashMap<String, String>,
    pub sync_notification_template: String,
}

impl From<ObsidianExportSettings> for ObsidianSettingsDto {
    fn from(value: ObsidianExportSettings) -> Self {
        Self {
            group_files_in_category_folders: value.group_files_in_category_folders,
            export_all_reader_documents: value.export_all_reader_documents,
            sync_notifications: value.sync_notifications,
            properties_template: value.properties_template,
            page_title_template: value.page_title_template,
            metadata_template: value.metadata_template,
            highlight_header_template: value.highlight_header_template,
            highlight_template: value.highlight_template,
            file_name_template: value.file_name_template,
            category_folder_templates: value.category_folder_templates,
            sync_notification_template: value.sync_notification_template,
        }
    }
}

impl From<ObsidianSettingsDto> for ObsidianExportSettings {
    fn from(value: ObsidianSettingsDto) -> Self {
        Self {
            group_files_in_category_folders: value.group_files_in_category_folders,
            export_all_reader_documents: value.export_all_reader_documents,
            sync_notifications: value.sync_notifications,
            properties_template: value.properties_template,
            page_title_template: value.page_title_template,
            metadata_template: value.metadata_template,
            highlight_header_template: value.highlight_header_template,
            highlight_template: value.highlight_template,
            file_name_template: value.file_name_template,
            category_folder_templates: value.category_folder_templates,
            sync_notification_template: value.sync_notification_template,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateObsidianSettingsRequest {
    pub group_files_in_category_folders: bool,
    pub export_all_reader_documents: bool,
    pub sync_notifications: bool,
    #[serde(default)]
    pub properties_template: Option<String>,
    pub page_title_template: String,
    pub metadata_template: String,
    pub highlight_header_template: String,
    pub highlight_template: String,
    #[serde(default)]
    pub file_name_template: Option<String>,
    #[serde(default)]
    pub category_folder_templates: std::collections::HashMap<String, String>,
    pub sync_notification_template: String,
}

impl From<UpdateObsidianSettingsRequest> for ObsidianExportSettings {
    fn from(value: UpdateObsidianSettingsRequest) -> Self {
        Self {
            group_files_in_category_folders: value.group_files_in_category_folders,
            export_all_reader_documents: value.export_all_reader_documents,
            sync_notifications: value.sync_notifications,
            properties_template: value.properties_template,
            page_title_template: value.page_title_template,
            metadata_template: value.metadata_template,
            highlight_header_template: value.highlight_header_template,
            highlight_template: value.highlight_template,
            file_name_template: value.file_name_template,
            category_folder_templates: value.category_folder_templates,
            sync_notification_template: value.sync_notification_template,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct ObsidianPreviewRequest {
    #[serde(default)]
    #[validate(length(min = 1, max = 64))]
    pub library_entry_id: Option<String>,
    #[serde(default)]
    pub settings: Option<ObsidianSettingsDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ObsidianPreviewResponse {
    pub file_path: String,
    pub full_content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub append_only_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_document_text_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_document_text: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct AuthorizeIntegrationRequest {
    #[serde(default)]
    pub redirect_after: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthorizeIntegrationResponse {
    pub authorize_url: String,
}

/// OAuth callback query string. Code/state are absent when the provider
/// reports an error or when the user navigates to the URL manually; in those
/// cases the handler emits a redirect to the hub with `integration_error=…`.
#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SyncIntegrationResponse {
    pub job_id: String,
}
