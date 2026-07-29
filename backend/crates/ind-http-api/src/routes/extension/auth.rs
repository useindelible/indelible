use super::*;

/// Check extension connection status.
#[utoipa::path(
    get,
    path = "/api/v1/extension/status",
    responses(
        (status = 200, description = "Extension auth status", body = ExtensionStatusResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_status(
    auth_user: AuthUser,
) -> Result<ApiResponse<ExtensionStatusResponse>, ApiError> {
    let user = &auth_user.user;

    Ok(ApiResponse::new(ExtensionStatusResponse {
        authenticated: true,
        user: Some(ExtensionUserInfo {
            id: user.id.to_string(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
        }),
    }))
}

/// Create an authorization code for the extension PKCE flow.
/// Requires a web JWT — extension or API tokens cannot mint authorization codes.
#[utoipa::path(
    post,
    path = "/api/v1/auth/extension/authorize",
    request_body = AuthorizeExtensionRequest,
    responses(
        (status = 200, description = "Authorization code issued", body = AuthorizeExtensionResponse),
        (status = 400, description = "Extension auth not configured"),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = [])),
    tag = "Extension",
)]
pub async fn extension_authorize(
    State(state): State<AppState>,
    RequireWebAccess(auth_user): RequireWebAccess,
    axum::Json(body): axum::Json<AuthorizeExtensionRequest>,
) -> Result<axum::Json<AuthorizeExtensionResponse>, ApiError> {
    let ext_auth = state
        .extension_auth_ops
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest {
            message: "extension auth not configured".to_string(),
        })?;

    let raw_code = ext_auth
        .create_authorization_code(
            auth_user.user_id,
            ClientType::Extension,
            body.code_challenge,
            body.code_challenge_method,
            body.redirect_uri,
        )
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(AuthorizeExtensionResponse {
        code: raw_code,
        state: body.state,
    }))
}

/// Exchange an authorization code + PKCE verifier for extension tokens.
#[utoipa::path(
    post,
    path = "/api/v1/auth/extension/token",
    request_body = ExtensionTokenRequest,
    responses(
        (status = 200, description = "Extension tokens issued", body = ExtensionTokenResponse),
        (status = 400, description = "Invalid code or verifier"),
    ),
    tag = "Extension",
)]
pub async fn extension_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    client_ip: ClientIp,
    axum::Json(body): axum::Json<ExtensionTokenRequest>,
) -> Result<axum::Json<ExtensionTokenResponse>, ApiError> {
    let ext_auth = state
        .extension_auth_ops
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest {
            message: "extension auth not configured".to_string(),
        })?;

    let ip = client_ip.audit_string();

    let user_agent = headers
        .get(http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let result = ext_auth
        .exchange_authorization_code(
            &body.code,
            &body.code_verifier,
            &body.redirect_uri,
            ClientType::Extension,
            ip,
            user_agent,
        )
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(ExtensionTokenResponse {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        expires_at: result.expires_at,
        token_type: "Bearer",
    }))
}

/// Refresh an extension access token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/extension/refresh",
    request_body = ExtensionRefreshRequest,
    responses(
        (status = 200, description = "New extension tokens issued", body = ExtensionTokenResponse),
        (status = 401, description = "Refresh token invalid or expired"),
    ),
    tag = "Extension",
)]
pub async fn extension_refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    client_ip: ClientIp,
    axum::Json(body): axum::Json<ExtensionRefreshRequest>,
) -> Result<axum::Json<ExtensionTokenResponse>, ApiError> {
    let ip = client_ip.audit_string();

    let user_agent = headers
        .get(http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let result = state
        .auth_service
        .refresh(&body.refresh_token, ip, user_agent)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(ExtensionTokenResponse {
        access_token: result.access_token,
        refresh_token: result.raw_refresh_token,
        expires_at: result.expires_at,
        token_type: "Bearer",
    }))
}

/// Revoke an extension refresh token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/extension/revoke",
    request_body = ExtensionRevokeRequest,
    responses(
        (status = 204, description = "Token revoked"),
        (status = 401, description = "Authentication required"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_revoke(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<ExtensionRevokeRequest>,
) -> Result<http::StatusCode, ApiError> {
    let _ = state
        .auth_service
        .logout_by_refresh_token(&body.refresh_token)
        .await;

    Ok(http::StatusCode::NO_CONTENT)
}
