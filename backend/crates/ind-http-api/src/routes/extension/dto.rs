use super::*;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, ToSchema)]
pub struct ExtensionStatusResponse {
    pub authenticated: bool,
    pub user: Option<ExtensionUserInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExtensionUserInfo {
    pub id: String,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AuthorizeExtensionRequest {
    pub code_challenge: String,
    #[serde(default = "default_s256")]
    pub code_challenge_method: String,
    pub state: String,
    pub redirect_uri: String,
}

fn default_s256() -> String {
    "S256".to_string()
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthorizeExtensionResponse {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ExtensionTokenRequest {
    pub code: String,
    pub code_verifier: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExtensionTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    #[schema(value_type = String)]
    pub token_type: &'static str,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ExtensionRefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ExtensionRevokeRequest {
    pub refresh_token: String,
}

// -- Save DTOs --

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct QuickSaveRequest {
    #[validate(length(min = 1, message = "url is required"))]
    pub url: String,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct ReaderSaveRequest {
    #[validate(length(min = 1, message = "url is required"))]
    pub url: String,
    pub canonical_url: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub excerpt: Option<String>,
    #[validate(length(min = 1, message = "reader_html is required"))]
    pub reader_html: String,
    pub language: Option<String>,
    pub lead_image_url: Option<String>,
    pub item_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct FullArchiveRequest {
    #[validate(length(min = 1, message = "url is required"))]
    pub url: String,
    pub canonical_url: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub reader_html: Option<String>,
    #[validate(length(min = 1, message = "html_base64 is required"))]
    pub html_base64: String,
    pub lead_image_url: Option<String>,
    pub excerpt: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    // Readability may return date-only strings like "2024-03-26" which fail
    // chrono's strict RFC 3339 parser. Accept as a raw string and parse leniently.
    pub published_at: Option<String>,
    pub item_type: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExtensionSaveResponse {
    /// Saved Library membership id (canonical `lib_` `LibraryEntryId`). The reader link is in
    /// `reader_url`; document-keyed capabilities resolve through the document the entry points at.
    pub library_entry_id: String,
    pub status: String,
    pub reader_url: String,
}

// -- Handlers --

/// Check extension connection status.

#[derive(Debug, Serialize, ToSchema)]
pub struct TagResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ExtensionUpsertNoteBody {
    pub body: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExtensionNoteResponse {
    pub id: String,
    pub body: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ExtensionReplaceTagsBody {
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExtensionReplaceTagsResponse {
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExtensionUrlCheckResponse {
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_entry_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reader_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub saved_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triage_state: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ExtensionCheckUrlParams {
    pub url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExtensionSavedEntryResponse {
    pub library_entry_id: String,
    pub document_id: String,
    pub reader_url: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub triage_state: String,
    pub is_favorite: bool,
    #[schema(value_type = String, format = DateTime)]
    pub saved_at: DateTime<Utc>,
    pub tags: Vec<TagResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<ExtensionNoteResponse>,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct PatchExtensionEntryBody {
    pub triage_state: Option<String>,
    pub is_favorite: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ExtensionCreateHighlightBody {
    pub color: String,
    pub text_content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<crate::routes::highlights::LocatorSchemaFlat>)]
    pub locator: Option<LocatorSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<crate::routes::highlights::SourceLocatorSchemaFlat>)]
    pub source_locator: Option<SourceLocatorSchema>,
}
