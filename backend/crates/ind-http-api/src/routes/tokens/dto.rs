use chrono::{DateTime, Duration, Utc};
use ind_domain::{ApiPermission, ApiToken};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const DEFAULT_EXPIRY_SECONDS: i64 = 90 * 24 * 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub enum ApiPermissionDto {
    #[serde(rename = "library:read")]
    LibraryRead,
    #[serde(rename = "library:write")]
    LibraryWrite,
    #[serde(rename = "feeds:read")]
    FeedsRead,
    #[serde(rename = "feeds:write")]
    FeedsWrite,
    #[serde(rename = "integrations:read")]
    IntegrationsRead,
    #[serde(rename = "integrations:write")]
    IntegrationsWrite,
    #[serde(rename = "webhooks:read")]
    WebhooksRead,
    #[serde(rename = "webhooks:write")]
    WebhooksWrite,
    #[serde(rename = "ai:read")]
    AiRead,
    #[serde(rename = "ai:write")]
    AiWrite,
    #[serde(rename = "ai:use")]
    AiUse,
    #[serde(rename = "obsidian:sync")]
    ObsidianSync,
}

impl From<ApiPermissionDto> for ApiPermission {
    fn from(value: ApiPermissionDto) -> Self {
        match value {
            ApiPermissionDto::LibraryRead => Self::LibraryRead,
            ApiPermissionDto::LibraryWrite => Self::LibraryWrite,
            ApiPermissionDto::FeedsRead => Self::FeedsRead,
            ApiPermissionDto::FeedsWrite => Self::FeedsWrite,
            ApiPermissionDto::IntegrationsRead => Self::IntegrationsRead,
            ApiPermissionDto::IntegrationsWrite => Self::IntegrationsWrite,
            ApiPermissionDto::WebhooksRead => Self::WebhooksRead,
            ApiPermissionDto::WebhooksWrite => Self::WebhooksWrite,
            ApiPermissionDto::AiRead => Self::AiRead,
            ApiPermissionDto::AiWrite => Self::AiWrite,
            ApiPermissionDto::AiUse => Self::AiUse,
            ApiPermissionDto::ObsidianSync => Self::ObsidianSync,
        }
    }
}

impl From<ApiPermission> for ApiPermissionDto {
    fn from(value: ApiPermission) -> Self {
        match value {
            ApiPermission::LibraryRead => Self::LibraryRead,
            ApiPermission::LibraryWrite => Self::LibraryWrite,
            ApiPermission::FeedsRead => Self::FeedsRead,
            ApiPermission::FeedsWrite => Self::FeedsWrite,
            ApiPermission::IntegrationsRead => Self::IntegrationsRead,
            ApiPermission::IntegrationsWrite => Self::IntegrationsWrite,
            ApiPermission::WebhooksRead => Self::WebhooksRead,
            ApiPermission::WebhooksWrite => Self::WebhooksWrite,
            ApiPermission::AiRead => Self::AiRead,
            ApiPermission::AiWrite => Self::AiWrite,
            ApiPermission::AiUse => Self::AiUse,
            ApiPermission::ObsidianSync => Self::ObsidianSync,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(transparent)]
#[schema(value_type = Option<i64>, default = 7776000)]
pub struct RequiredNullableDuration(Option<i64>);

impl RequiredNullableDuration {
    pub fn into_duration(self) -> Option<Duration> {
        self.0.map(Duration::seconds)
    }

    fn seconds(self) -> Option<i64> {
        self.0
    }
}

impl Default for RequiredNullableDuration {
    fn default() -> Self {
        Self(Some(DEFAULT_EXPIRY_SECONDS))
    }
}

fn valid_api_token_expiry(
    value: &RequiredNullableDuration,
) -> Result<(), validator::ValidationError> {
    let Some(seconds) = value.seconds() else {
        return Ok(());
    };
    if (1..=ind_auth::MAX_API_TOKEN_EXPIRY_SECONDS).contains(&seconds) {
        return Ok(());
    }

    let mut error = validator::ValidationError::new("range");
    error.message = Some(
        format!(
            "must be between 1 and {} seconds",
            ind_auth::MAX_API_TOKEN_EXPIRY_SECONDS
        )
        .into(),
    );
    Err(error)
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiTokenResponse {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    pub prefix: String,
    pub permissions: Vec<ApiPermissionDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_used_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub expires_at: Option<DateTime<Utc>>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl From<ApiToken> for ApiTokenResponse {
    fn from(token: ApiToken) -> Self {
        Self {
            id: token.id.to_string(),
            object: "api_token",
            name: token.name,
            prefix: token.prefix,
            permissions: token.permissions.into_iter().map(Into::into).collect(),
            last_used_at: token.last_used_at,
            expires_at: token.expires_at,
            created_at: token.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateApiTokenResponse {
    #[serde(flatten)]
    pub token: ApiTokenResponse,
    pub raw_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenListResponse {
    pub data: Vec<ApiTokenResponse>,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateApiTokenRequest {
    #[validate(custom(function = "crate::validation::trimmed_non_blank"))]
    #[validate(custom(function = "crate::validation::trimmed_max_display_name_length"))]
    pub name: String,
    #[validate(length(min = 1, message = "must include at least one permission"))]
    pub permissions: Vec<ApiPermissionDto>,
    #[serde(default)]
    #[validate(custom(function = "valid_api_token_expiry"))]
    pub expires_in: RequiredNullableDuration,
}
