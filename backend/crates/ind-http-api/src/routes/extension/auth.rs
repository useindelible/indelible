use super::*;
use axum::response::Redirect;
use url::Url;

/// Begin browser-managed extension authentication after validating its callback.
#[utoipa::path(
    get,
    path = "/api/v1/auth/extension/start",
    params(
        ("code_challenge" = String, Query, description = "S256 PKCE code challenge"),
        ("state" = String, Query, description = "Opaque CSRF state"),
        ("redirect_uri" = String, Query, description = "Exact browser Identity callback URL"),
    ),
    responses(
        (status = 307, description = "Continue to the web consent page"),
        (status = 400, description = "Missing, malformed, or unregistered callback"),
    ),
    tag = "Extension",
)]
pub async fn extension_auth_start(
    State(state): State<AppState>,
    Query(query): Query<ExtensionAuthStartQuery>,
) -> Result<Redirect, ApiError> {
    validate_extension_start_query(&state, &query)?;
    let consent_url = extension_consent_url(&state.config.frontend_url, &query)?;
    Ok(Redirect::temporary(consent_url.as_str()))
}

fn validate_extension_start_query(
    state: &AppState,
    query: &ExtensionAuthStartQuery,
) -> Result<(), ApiError> {
    if query.code_challenge.len() != 43
        || !query
            .code_challenge
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(ApiError::BadRequest {
            message: "code_challenge must be an S256 PKCE challenge".to_string(),
        });
    }
    if query.state.is_empty() {
        return Err(ApiError::BadRequest {
            message: "state is required".to_string(),
        });
    }
    if !state
        .config
        .extension_redirect_uris
        .contains(&query.redirect_uri)
    {
        return Err(ApiError::BadRequest {
            message: "redirect_uri is not allowed".to_string(),
        });
    }
    Ok(())
}

fn extension_consent_url(
    frontend_url: &str,
    query: &ExtensionAuthStartQuery,
) -> Result<Url, ApiError> {
    let mut url = Url::parse(frontend_url).map_err(|error| ApiError::Internal {
        message: format!("invalid configured frontend URL: {error}"),
    })?;
    url.set_path("/extension/auth");
    url.set_query(None);
    url.set_fragment(None);
    url.query_pairs_mut()
        .append_pair("code_challenge", &query.code_challenge)
        .append_pair("state", &query.state)
        .append_pair("redirect_uri", &query.redirect_uri);
    Ok(url)
}

/// Check extension connection status.
#[utoipa::path(
    get,
    path = "/api/v1/extension/status",
    responses(
        (status = 200, description = "Extension auth status", body = ExtensionStatusResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Extension access required"),
    ),
    security(("bearer" = [])),
    tag = "Extension",
)]
pub async fn extension_status(
    RequireExtensionAccess(auth_user): RequireExtensionAccess,
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
    security(),
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
