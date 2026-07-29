mod dto;

use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::routing::{get, post};
use chrono::Utc;
use ind_application::asset_serving::AssetServingMode;
use ind_application::storage::ObjectStorage;
use ind_auth::{self as auth};
use ind_domain::Theme;

use axum::response::{IntoResponse, Response};
use http::StatusCode;

use crate::error::{ApiError, FieldError};
use crate::extract::ValidatedJson;
use crate::middleware::{AccountAccess, clear_asset_cookie, clear_refresh_cookie};
use crate::response::ApiResponse;
use crate::state::{AppConfig, AppState};
pub(crate) use dto::{
    AvatarUploadUrlRequest, AvatarUploadUrlResponse, ChangeEmailRequest, ChangePasswordRequest,
    DeleteAccountRequest, ProfileResponse, UpdateProfileRequest,
};

// -- Handlers --

#[utoipa::path(
    get,
    path = "/api/v1/me",
    responses(
        (status = 200, description = "Current user profile", body = ProfileResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Email verification required"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Account",
)]
pub async fn get_profile(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
) -> Result<ApiResponse<ProfileResponse>, ApiError> {
    let profile = state
        .account_ops
        .get_profile(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    let profile = build_profile_response(&state.config, state.storage.as_deref(), profile).await;
    Ok(ApiResponse::new(profile))
}

#[utoipa::path(
    patch,
    path = "/api/v1/me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Updated profile", body = ProfileResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Email verification required"),
        (status = 422, description = "Validation error"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Account",
)]
pub async fn update_profile(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<UpdateProfileRequest>,
) -> Result<ApiResponse<ProfileResponse>, ApiError> {
    let theme = body.theme.as_deref().map(parse_theme).transpose()?;

    let service_req = auth::UpdateProfileRequest {
        display_name: body.display_name,
        avatar_url: body.avatar_url,
        locale: body.locale,
        timezone: body.timezone,
        theme,
    };

    let profile = state
        .account_ops
        .update_profile(auth_user.user_id, service_req)
        .await
        .map_err(ApiError::from)?;

    let profile = build_profile_response(&state.config, state.storage.as_deref(), profile).await;
    Ok(ApiResponse::new(profile))
}

#[utoipa::path(
    delete,
    path = "/api/v1/me",
    request_body = DeleteAccountRequest,
    responses(
        (status = 204, description = "Account deleted"),
        (status = 400, description = "Invalid confirmation"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Email verification required"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Account",
)]
pub async fn delete_account(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<DeleteAccountRequest>,
) -> Result<Response, ApiError> {
    state
        .account_ops
        .delete_account(auth_user.user_id, body.confirmation)
        .await
        .map_err(ApiError::from)?;

    let mut headers = http::HeaderMap::new();
    clear_refresh_cookie(&mut headers, &state.config);
    clear_asset_cookie(&mut headers, &state.config);

    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().extend(headers.drain());

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/v1/me/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed"),
        (status = 401, description = "Invalid current password"),
        (status = 403, description = "Email verification required"),
        (status = 422, description = "Validation error"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Account",
)]
pub async fn change_password(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<ChangePasswordRequest>,
) -> Result<ApiResponse<serde_json::Value>, ApiError> {
    let service_req = auth::ChangePasswordRequest {
        current_password: body.current_password,
        new_password: body.new_password,
    };

    state
        .account_ops
        .change_password(auth_user.user_id, service_req)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(
        serde_json::json!({ "message": "password changed" }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/me/email",
    request_body = ChangeEmailRequest,
    responses(
        (status = 200, description = "Verification email sent to new address"),
        (status = 401, description = "Invalid password"),
        (status = 403, description = "Email verification required"),
        (status = 409, description = "Email already in use"),
        (status = 422, description = "Validation error"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Account",
)]
pub async fn change_email(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<ChangeEmailRequest>,
) -> Result<Response, ApiError> {
    state
        .account_ops
        .change_email(auth_user.user_id, body.new_email, body.password)
        .await
        .map_err(ApiError::from)?;

    let body_json = serde_json::json!({ "message": "verification email sent to new address" });
    let body_bytes = serde_json::to_vec(&body_json).map_err(ApiError::from)?;

    let mut headers = http::HeaderMap::new();
    clear_refresh_cookie(&mut headers, &state.config);
    clear_asset_cookie(&mut headers, &state.config);

    let mut response = (StatusCode::OK, body_bytes).into_response();
    #[expect(
        clippy::unwrap_used,
        reason = "parsing a static ASCII literal into a header value is infallible"
    )]
    let content_type = "application/json".parse().unwrap();
    response
        .headers_mut()
        .insert(http::header::CONTENT_TYPE, content_type);
    response.headers_mut().extend(headers.drain());

    Ok(response)
}

const AVATAR_UPLOAD_EXPIRY_SECS: u64 = 300;
const AVATAR_READ_EXPIRY_SECS: u64 = 3600;

#[utoipa::path(
    post,
    path = "/api/v1/me/avatar/upload-url",
    request_body = AvatarUploadUrlRequest,
    responses(
        (status = 200, description = "Presigned upload URL", body = AvatarUploadUrlResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Email verification required"),
        (status = 422, description = "Unsupported content type"),
        (status = 503, description = "Storage not configured"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Account",
)]
pub async fn avatar_upload_url(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    axum::Json(body): axum::Json<AvatarUploadUrlRequest>,
) -> Result<ApiResponse<AvatarUploadUrlResponse>, ApiError> {
    let storage = state
        .storage
        .as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable {
            message: "storage not configured".to_string(),
        })?;

    let ext = ind_domain::AvatarContentType::from_mime(&body.content_type)
        .map(|content_type| content_type.extension)
        .ok_or_else(|| ApiError::ValidationError {
            errors: vec![FieldError {
                field: "content_type".into(),
                message: "must be one of: image/jpeg, image/png, image/webp".into(),
            }],
        })?;

    let object_id = uuid::Uuid::now_v7();
    let key = format!("{}/avatars/{}.{}", auth_user.user_id, object_id, ext);

    let expiry = Duration::from_secs(AVATAR_UPLOAD_EXPIRY_SECS);
    let upload_url = storage
        .presigned_upload_url(&key, &body.content_type, expiry)
        .await
        .map_err(|_| ApiError::Internal {
            message: "failed to generate upload URL".to_string(),
        })?;

    let expires_at =
        (Utc::now() + chrono::Duration::seconds(AVATAR_UPLOAD_EXPIRY_SECS as i64)).to_rfc3339();

    Ok(ApiResponse::new(AvatarUploadUrlResponse {
        upload_url,
        object_url: key,
        expires_at,
    }))
}

// -- Helpers --

async fn build_profile_response(
    config: &AppConfig,
    storage: Option<&dyn ObjectStorage>,
    profile: auth::UserProfile,
) -> ProfileResponse {
    let avatar_url =
        resolve_avatar_url(config, storage, profile.id, profile.avatar_url.as_deref()).await;
    let ingest_email = config
        .email_feed_domain
        .as_deref()
        .map(|d| format!("{}@{d}", profile.email_token));
    let ingest_library_email = config
        .email_library_domain
        .as_deref()
        .map(|d| format!("{}@{d}", profile.email_token));
    ProfileResponse {
        id: profile.id.to_string(),
        object: "user",
        email: profile.email,
        display_name: profile.display_name,
        avatar_url,
        locale: profile.locale,
        timezone: profile.timezone,
        theme: theme_to_string(profile.theme),
        email_verified: profile.email_verified,
        onboarding_completed: profile.onboarding_completed,
        has_password: profile.has_password,
        ingest_email,
        ingest_library_email,
        created_at: profile.created_at,
        updated_at: profile.updated_at,
    }
}

async fn resolve_avatar_url(
    config: &AppConfig,
    storage: Option<&dyn ObjectStorage>,
    user_id: ind_domain::UserId,
    avatar_ref: Option<&str>,
) -> Option<String> {
    let avatar_ref = avatar_ref?;
    if avatar_ref.starts_with("https://") || avatar_ref.starts_with("http://") {
        return Some(avatar_ref.to_string());
    }
    if !crate::validation::avatar_key_belongs_to_user(&user_id, avatar_ref) {
        return None;
    }
    let storage = storage?;

    match config.asset_serving_mode {
        AssetServingMode::Presigned => storage
            .presigned_url(avatar_ref, Duration::from_secs(AVATAR_READ_EXPIRY_SECS))
            .await
            .ok(),
        AssetServingMode::Passthrough => Some(format!(
            "{}/api/v1/assets/{}",
            config.base_url.trim_end_matches('/'),
            avatar_ref,
        )),
    }
}

fn theme_to_string(theme: Theme) -> String {
    theme.as_str().to_string()
}

fn parse_theme(s: &str) -> Result<Theme, ApiError> {
    s.parse::<Theme>().map_err(|_| ApiError::ValidationError {
        errors: vec![FieldError {
            field: "theme".into(),
            message: format!("must be one of: {}", Theme::NAMES.join(", ")),
        }],
    })
}

pub fn account_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/me",
            get(get_profile)
                .patch(update_profile)
                .delete(delete_account),
        )
        .route("/api/v1/me/password", post(change_password))
        .route("/api/v1/me/email", post(change_email))
        .route("/api/v1/me/avatar/upload-url", post(avatar_upload_url))
}
