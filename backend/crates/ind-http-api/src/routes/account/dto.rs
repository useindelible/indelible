use chrono::{DateTime, Utc};
use ind_auth::UserProfile;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct AvatarUploadUrlRequest {
    /// MIME type of the image to upload. Must be image/jpeg, image/png, or image/webp.
    pub content_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AvatarUploadUrlResponse {
    pub upload_url: String,
    /// Stable avatar reference to persist via PATCH /api/v1/me.
    pub object_url: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileResponse {
    pub id: String,
    pub object: &'static str,
    pub email: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub locale: String,
    pub timezone: String,
    pub theme: String,
    pub email_verified: bool,
    pub onboarding_completed: bool,
    pub has_password: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingest_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingest_library_email: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateProfileRequest {
    #[validate(custom(function = "crate::validation::optional_trimmed_non_blank"))]
    #[validate(custom(function = "crate::validation::optional_trimmed_max_display_name_length"))]
    pub display_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[schema(value_type = Option<String>, nullable)]
    #[validate(custom(function = "crate::validation::optional_avatar_reference"))]
    pub avatar_url: Option<Option<String>>,
    #[validate(custom(function = "crate::validation::optional_locale"))]
    pub locale: Option<String>,
    #[validate(custom(function = "crate::validation::optional_timezone"))]
    pub timezone: Option<String>,
    #[validate(custom(function = "crate::validation::optional_theme"))]
    pub theme: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(custom(function = "crate::validation::password_length"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct ChangeEmailRequest {
    #[validate(custom(function = "crate::validation::trimmed_email"))]
    pub new_email: String,
    #[validate(length(min = 1, message = "must not be empty"))]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct DeleteAccountRequest {
    #[validate(length(min = 1, message = "must not be empty"))]
    pub confirmation: String,
}

pub fn deserialize_optional_nullable<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val: Option<Option<String>> = Option::deserialize(deserializer)?;
    Ok(val)
}

impl ProfileResponse {
    pub fn from_user_profile(
        profile: UserProfile,
        theme: String,
        email_feed_domain: Option<&str>,
        email_library_domain: Option<&str>,
    ) -> Self {
        let ingest_email = email_feed_domain.map(|d| format!("{}@{d}", profile.email_token));
        let ingest_library_email =
            email_library_domain.map(|d| format!("{}@{d}", profile.email_token));

        Self {
            id: profile.id.to_string(),
            object: "user",
            email: profile.email,
            display_name: profile.display_name,
            avatar_url: profile.avatar_url,
            locale: profile.locale,
            timezone: profile.timezone,
            theme,
            email_verified: profile.email_verified,
            onboarding_completed: profile.onboarding_completed,
            has_password: profile.has_password,
            ingest_email,
            ingest_library_email,
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        }
    }
}
