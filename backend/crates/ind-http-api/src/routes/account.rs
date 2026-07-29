mod dto;

use axum::Router;
use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::routing::{get, post};
use ind_application::storage::ObjectStorage;
use ind_auth::{self as auth};
use ind_domain::Theme;
use ind_integrations::email::format_ingest_address;

use axum::response::{IntoResponse, Response};
use http::StatusCode;

use crate::error::{ApiError, FieldError};
use crate::extract::{ValidatedJson, read_multipart_field_bytes};
use crate::middleware::{RequireVerifiedUserAccessJwt, clear_asset_cookie, clear_refresh_cookie};
use crate::response::ApiResponse;
use crate::state::{AppConfig, AppState};
pub(crate) use dto::{
    AvatarUploadSchema, ChangeEmailRequest, ChangePasswordRequest, DeleteAccountRequest,
    ProfileResponse, UpdateProfileRequest,
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
    security(("bearer" = [])),
    tag = "Account",
)]
pub async fn get_profile(
    RequireVerifiedUserAccessJwt(auth_user): RequireVerifiedUserAccessJwt,
    State(state): State<AppState>,
) -> Result<ApiResponse<ProfileResponse>, ApiError> {
    let profile = state
        .account_ops
        .get_profile(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    let profile = build_profile_response(&state.config, state.storage.as_deref(), profile);
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
    security(("bearer" = [])),
    tag = "Account",
)]
pub async fn update_profile(
    RequireVerifiedUserAccessJwt(auth_user): RequireVerifiedUserAccessJwt,
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

    let profile = build_profile_response(&state.config, state.storage.as_deref(), profile);
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
    security(("bearer" = [])),
    tag = "Account",
)]
pub async fn delete_account(
    RequireVerifiedUserAccessJwt(auth_user): RequireVerifiedUserAccessJwt,
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
    security(("bearer" = [])),
    tag = "Account",
)]
pub async fn change_password(
    RequireVerifiedUserAccessJwt(auth_user): RequireVerifiedUserAccessJwt,
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
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Verified user access JWT from a supported client and verified email required"),
        (status = 422, description = "Validation error"),
        (status = 503, description = "Changing an email address needs an outbound mail transport, which is not configured"),
    ),
    security(("bearer" = [])),
    tag = "Account",
)]
pub async fn change_email(
    RequireVerifiedUserAccessJwt(_auth_user): RequireVerifiedUserAccessJwt,
    State(_state): State<AppState>,
    ValidatedJson(_body): ValidatedJson<ChangeEmailRequest>,
) -> Result<Response, ApiError> {
    // Changing an address clears verification and revokes every session, then
    // relies on a verification link to restore access. This release configures no
    // outbound mail transport, so that link can never arrive: the account would
    // be left unverified, signed out everywhere, and unrecoverable short of
    // editing the database.
    //
    // The refusal happens here rather than inside the service so the change-email
    // implementation stays intact for the release that adds a transport, and so
    // that no password is checked first — the endpoint cannot be used to probe
    // credentials. Restore the body below, and revisit the unconditional
    // `email_verified: true` in `ind-auth`'s registration path, together.
    Err(auth::AuthError::MailTransportUnavailable.into())
}

const AVATAR_MAX_UPLOAD_BYTES: usize = 2 * 1024 * 1024;

#[utoipa::path(
    post,
    path = "/api/v1/me/avatar",
    request_body(content_type = "multipart/form-data", content = inline(AvatarUploadSchema)),
    responses(
        (status = 200, description = "Profile with the new avatar", body = ProfileResponse),
        (status = 400, description = "Missing file field or invalid upload"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Email verification required"),
        (status = 413, description = "File too large"),
        (status = 422, description = "Unsupported content type"),
        (status = 503, description = "Storage not configured"),
    ),
    security(("bearer" = [])),
    tag = "Account",
)]
pub async fn upload_avatar(
    RequireVerifiedUserAccessJwt(auth_user): RequireVerifiedUserAccessJwt,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<ApiResponse<ProfileResponse>, ApiError> {
    let storage = state
        .storage
        .as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable {
            message: "storage not configured".to_string(),
        })?;

    let mut file: Option<(String, bytes::Bytes)> = None;
    let mut total = 0usize;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::BadRequest {
            message: format!("multipart parse error: {err}"),
        })?
    {
        if field.name() != Some("file") {
            continue;
        }
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let data = read_multipart_field_bytes(field, &mut total, AVATAR_MAX_UPLOAD_BYTES).await?;
        file = Some((content_type, data));
    }
    let (content_type, data) = file.ok_or_else(|| ApiError::BadRequest {
        message: "missing 'file' field in multipart upload".into(),
    })?;

    let ext = ind_domain::AvatarContentType::from_mime(&content_type)
        .map(|content_type| content_type.extension)
        .ok_or_else(|| ApiError::ValidationError {
            errors: vec![FieldError {
                field: "file".into(),
                message: "must be one of: image/jpeg, image/png, image/webp".into(),
            }],
        })?;

    let key = format!(
        "{}/avatars/{}.{}",
        auth_user.user_id,
        uuid::Uuid::now_v7(),
        ext
    );
    storage
        .upload(&key, &content_type, data)
        .await
        .map_err(|_| ApiError::Internal {
            message: "failed to store avatar".to_string(),
        })?;

    let previous_avatar = state
        .account_ops
        .get_profile(auth_user.user_id)
        .await
        .map_err(ApiError::from)?
        .avatar_url;

    let profile = state
        .account_ops
        .update_profile(
            auth_user.user_id,
            auth::UpdateProfileRequest {
                display_name: None,
                avatar_url: Some(Some(key.clone())),
                locale: None,
                timezone: None,
                theme: None,
            },
        )
        .await
        .map_err(ApiError::from)?;

    // A replaced avatar is unreachable once the profile points elsewhere;
    // delete best-effort so self-hosted buckets do not accumulate orphans.
    if let Some(old_key) = previous_avatar
        && old_key != key
        && crate::validation::avatar_key_belongs_to_user(&auth_user.user_id, &old_key)
        && let Err(error) = storage.delete(&old_key).await
    {
        tracing::warn!(%error, old_key, "failed to delete replaced avatar object");
    }

    let profile = build_profile_response(&state.config, state.storage.as_deref(), profile);
    Ok(ApiResponse::new(profile))
}

// -- Helpers --

fn build_profile_response(
    config: &AppConfig,
    storage: Option<&dyn ObjectStorage>,
    profile: auth::UserProfile,
) -> ProfileResponse {
    let avatar_url = resolve_avatar_url(config, storage, profile.id, profile.avatar_url.as_deref());
    let ingest_email = format_ingest_address(
        &profile.email_token,
        ind_domain::EmailDestination::Feed,
        config.email_feed_domain.as_deref(),
        config.email_library_domain.as_deref(),
    );
    let ingest_library_email = format_ingest_address(
        &profile.email_token,
        ind_domain::EmailDestination::Library,
        config.email_feed_domain.as_deref(),
        config.email_library_domain.as_deref(),
    );
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

/// Project a stored avatar reference into a client-facing URL. External
/// (OAuth-provided) http(s) references pass through; internal keys become
/// API asset-proxy URLs. The proxy itself decides stream-vs-redirect based
/// on `asset_serving_mode`, so no mode branch belongs here.
fn resolve_avatar_url(
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
    // Without storage the proxy cannot serve the bytes; omit the URL.
    storage?;
    Some(crate::routes::asset_urls::avatar_url(
        &config.base_url,
        avatar_ref,
    ))
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
        .route("/api/v1/me/avatar", post(upload_avatar))
        // Leave headroom over the avatar cap for multipart framing; the
        // handler enforces the real per-file limit.
        .layer(DefaultBodyLimit::max(AVATAR_MAX_UPLOAD_BYTES + 64 * 1024))
}
