use chrono::{DateTime, Duration, Utc};
use ind_domain::ApiToken;
use serde::{Deserialize, Serialize};
use serde_with::{DurationSeconds, serde_as};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiTokenResponse {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    pub prefix: String,
    pub scopes: Vec<String>,
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
            scopes: token.scopes,
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

#[serde_as]
#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateApiTokenRequest {
    #[validate(custom(function = "crate::validation::trimmed_non_blank"))]
    #[validate(custom(function = "crate::validation::trimmed_max_display_name_length"))]
    pub name: String,
    #[validate(length(min = 1, message = "must include at least one scope"))]
    #[validate(custom(function = "crate::validation::allowed_scopes"))]
    pub scopes: Vec<String>,
    #[serde_as(as = "Option<DurationSeconds<i64>>")]
    #[schema(value_type = Option<i64>)]
    #[validate(custom(function = "crate::validation::positive_duration"))]
    pub expires_in: Option<Duration>,
}
