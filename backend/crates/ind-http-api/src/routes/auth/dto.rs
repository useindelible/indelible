use chrono::{DateTime, Utc};
use ind_domain::{RefreshToken, User};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct RegisterRequest {
    #[validate(custom(function = "crate::validation::trimmed_email"))]
    pub email: String,
    #[validate(length(min = 1, message = "must not be empty"))]
    pub password: String,
    #[validate(custom(function = "crate::validation::trimmed_non_blank"))]
    #[validate(custom(function = "crate::validation::trimmed_max_display_name_length"))]
    pub display_name: String,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct LoginRequest {
    #[validate(custom(function = "crate::validation::trimmed_email"))]
    pub email: String,
    #[validate(length(min = 1, message = "must not be empty"))]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct ForgotPasswordRequest {
    #[validate(custom(function = "crate::validation::trimmed_email"))]
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct ResetPasswordRequest {
    #[validate(custom(function = "crate::validation::trimmed_non_blank"))]
    pub token: String,
    #[validate(length(min = 1, message = "must not be empty"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct VerifyEmailRequest {
    #[validate(custom(function = "crate::validation::trimmed_non_blank"))]
    pub token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub id: String,
    pub object: &'static str,
    pub email: String,
    pub display_name: String,
    pub email_verified: bool,
    pub onboarding_completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

impl AuthResponse {
    pub fn from_login(
        user: &User,
        access_token: String,
        expires_at: i64,
        refresh_token_for_body: Option<String>,
    ) -> Self {
        Self {
            id: user.id.to_string(),
            object: "user",
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            email_verified: user.email_verified,
            onboarding_completed: user.onboarding_completed,
            access_token: Some(access_token),
            expires_at: Some(expires_at),
            refresh_token: refresh_token_for_body,
        }
    }

    pub fn from_user(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            object: "user",
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            email_verified: user.email_verified,
            onboarding_completed: user.onboarding_completed,
            access_token: None,
            expires_at: None,
            refresh_token: None,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshResponse {
    pub access_token: String,
    pub expires_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshTokenDetail {
    pub family_id: String,
    pub client_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub last_used_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub expires_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl RefreshTokenDetail {
    pub fn from_refresh_token(token: &RefreshToken) -> Self {
        Self {
            family_id: token.family_id.to_string(),
            client_type: serde_json::to_value(token.client_type)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "web".to_string()),
            ip_address: token.ip_address.clone(),
            last_used_at: token.last_used_at,
            expires_at: token.expires_at,
            created_at: token.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshTokenListResponse {
    pub tokens: Vec<RefreshTokenDetail>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthProvidersResponse {
    pub providers: Vec<OAuthProviderInfo>,
    /// Whether new account registration is currently permitted (config flag, or
    /// always true during first-run setup when no users exist yet).
    pub signups_enabled: bool,
    /// True when the instance has no users yet and is awaiting first-run setup.
    pub setup_required: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthProviderInfo {
    pub id: String,
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: String,
    pub iss: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackForm {
    pub code: Option<String>,
    pub state: String,
    pub iss: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NativeOAuthStartQuery {
    pub(super) platform: String,
    pub(super) code_challenge: String,
    pub(super) code_challenge_method: String,
    pub(super) app_state: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NativeOAuthTokenForm {
    pub(super) grant_type: String,
    pub(super) code: String,
    pub(super) code_verifier: String,
    pub(super) redirect_uri: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NativeOAuthTokenResponse {
    pub(super) access_token: String,
    pub(super) refresh_token: String,
    pub(super) token_type: &'static str,
    pub(super) expires_at: i64,
    pub(super) refresh_token_expires_at: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub(super) struct NativeOAuthErrorResponse {
    pub(super) error: &'static str,
    pub(super) error_description: &'static str,
}

pub(super) struct OAuthCallbackPayload {
    pub(super) code: Option<String>,
    pub(super) state: String,
    pub(super) iss: Option<String>,
    pub(super) error: Option<String>,
    pub(super) error_description: Option<String>,
}
