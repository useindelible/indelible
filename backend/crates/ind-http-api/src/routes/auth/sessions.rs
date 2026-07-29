use super::helpers::*;
use super::*;

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Account created", body = AuthResponse),
        (status = 403, description = "Signups disabled"),
        (status = 409, description = "Email already exists"),
        (status = 422, description = "Validation error"),
    ),
    tag = "Auth",
)]
pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    client_ip: ClientIp,
    ValidatedJson(body): ValidatedJson<RegisterRequest>,
) -> Result<Response, ApiError> {
    let client_type = detect_client_type(&headers);
    let ip = client_ip.audit_string();
    let user_agent = extract_user_agent(&headers);

    let result = state
        .auth_service
        .register(
            ind_auth::RegisterRequest {
                email: body.email,
                password: body.password,
                display_name: body.display_name,
            },
            client_type,
            ip,
            user_agent,
        )
        .await
        .map_err(ApiError::from)?;

    let refresh_for_body = if is_non_browser_client(client_type) {
        Some(result.raw_refresh_token.clone())
    } else {
        None
    };

    let mut cookie_headers = HeaderMap::new();
    set_auth_cookies_if_browser(
        &mut cookie_headers,
        client_type,
        &result.raw_refresh_token,
        &result.user.id,
        &state.config,
    );

    let response_body = AuthResponse::from_login(
        &result.user,
        result.access_token,
        result.expires_at,
        refresh_for_body,
    );
    let body_bytes = serde_json::to_vec(&response_body).map_err(ApiError::from)?;

    let mut response = (StatusCode::CREATED, body_bytes).into_response();
    #[expect(
        clippy::unwrap_used,
        reason = "parsing a static ASCII literal into a header value is infallible"
    )]
    let content_type = "application/json".parse().unwrap();
    response
        .headers_mut()
        .insert(http::header::CONTENT_TYPE, content_type);
    response.headers_mut().extend(cookie_headers.drain());

    Ok(response)
}

/// Log in with email and password.
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 403, description = "Account disabled"),
        (status = 422, description = "Validation error"),
    ),
    tag = "Auth",
)]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    client_ip: ClientIp,
    ValidatedJson(body): ValidatedJson<LoginRequest>,
) -> Result<Response, ApiError> {
    let client_type = detect_client_type(&headers);
    let ip = client_ip.audit_string();
    let user_agent = extract_user_agent(&headers);

    let result = state
        .auth_service
        .login(
            ind_auth::LoginRequest {
                email: body.email,
                password: body.password,
            },
            client_type,
            ip,
            user_agent,
        )
        .await
        .map_err(ApiError::from)?;

    let refresh_for_body = if is_non_browser_client(client_type) {
        Some(result.raw_refresh_token.clone())
    } else {
        None
    };

    let mut cookie_headers = HeaderMap::new();
    set_auth_cookies_if_browser(
        &mut cookie_headers,
        client_type,
        &result.raw_refresh_token,
        &result.user.id,
        &state.config,
    );

    let response_body = AuthResponse::from_login(
        &result.user,
        result.access_token,
        result.expires_at,
        refresh_for_body,
    );
    let body_bytes = serde_json::to_vec(&response_body).map_err(ApiError::from)?;

    let mut response = (StatusCode::OK, body_bytes).into_response();
    #[expect(
        clippy::unwrap_used,
        reason = "parsing a static ASCII literal into a header value is infallible"
    )]
    let content_type = "application/json".parse().unwrap();
    response
        .headers_mut()
        .insert(http::header::CONTENT_TYPE, content_type);
    response.headers_mut().extend(cookie_headers.drain());

    Ok(response)
}

/// Refresh access token using refresh cookie or body.
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = Option<RefreshTokenRequest>,
    responses(
        (status = 200, description = "Token refreshed", body = RefreshResponse),
        (status = 401, description = "Invalid or expired refresh token"),
    ),
    tag = "Auth",
)]
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    client_ip: ClientIp,
    body: String,
) -> Result<Response, ApiError> {
    let client_type = detect_client_type(&headers);
    let ip = client_ip.audit_string();
    let user_agent = extract_user_agent(&headers);
    let body_token = extract_body_refresh_token(&body);

    let raw_refresh = extract_refresh_cookie(&headers)
        .or(body_token)
        .ok_or_else(|| ApiError::Unauthorized {
            message: "refresh token required".to_string(),
        })?;

    let result = state
        .auth_service
        .refresh(&raw_refresh, ip, user_agent)
        .await
        .map_err(ApiError::from)?;

    let mut cookie_headers = HeaderMap::new();
    if !is_non_browser_client(client_type)
        && !result.raw_refresh_token.is_empty()
        && let Ok(claims) = ind_auth::validate_access_token(&result.access_token, &state.jwt_secret)
        && let Ok(user_id) = claims.user_id()
    {
        set_auth_cookies_if_browser(
            &mut cookie_headers,
            client_type,
            &result.raw_refresh_token,
            &user_id,
            &state.config,
        );
    }

    let response_body = RefreshResponse {
        access_token: result.access_token,
        expires_at: result.expires_at,
        refresh_token: refresh_token_for_body(client_type, &result.raw_refresh_token),
    };
    let body_bytes = serde_json::to_vec(&response_body).map_err(ApiError::from)?;

    let mut response = (StatusCode::OK, body_bytes).into_response();
    #[expect(
        clippy::unwrap_used,
        reason = "parsing a static ASCII literal into a header value is infallible"
    )]
    let content_type = "application/json".parse().unwrap();
    response
        .headers_mut()
        .insert(http::header::CONTENT_TYPE, content_type);
    response.headers_mut().extend(cookie_headers.drain());

    Ok(response)
}

/// Log out the current session (revoke the refresh token family).
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    request_body = Option<RefreshTokenRequest>,
    responses(
        (status = 204, description = "Logged out"),
        (status = 401, description = "Authentication required"),
    ),
    tag = "Auth",
)]
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<Response, ApiError> {
    if let Some(raw_refresh) =
        extract_refresh_cookie(&headers).or(extract_body_refresh_token(&body))
    {
        let _ = state
            .auth_service
            .logout_by_refresh_token(&raw_refresh)
            .await;
    }

    let mut cookie_headers = HeaderMap::new();
    clear_refresh_cookie(&mut cookie_headers, &state.config);
    clear_asset_cookie(&mut cookie_headers, &state.config);

    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().extend(cookie_headers.drain());

    Ok(response)
}

/// Revoke all refresh tokens (logout everywhere).
#[utoipa::path(
    delete,
    path = "/api/v1/auth/refresh-tokens",
    responses(
        (status = 204, description = "All refresh tokens revoked"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Account session required"),
    ),
    security(("bearer" = [])),
    tag = "Auth",
)]
pub async fn revoke_all_refresh_tokens(
    State(state): State<AppState>,
    RequireAccountSession(auth_user): RequireAccountSession,
) -> Result<Response, ApiError> {
    state
        .auth_service
        .logout_all(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;

    let mut cookie_headers = HeaderMap::new();
    clear_refresh_cookie(&mut cookie_headers, &state.config);
    clear_asset_cookie(&mut cookie_headers, &state.config);

    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().extend(cookie_headers.drain());

    Ok(response)
}

/// List active refresh token families.
#[utoipa::path(
    get,
    path = "/api/v1/auth/refresh-tokens",
    responses(
        (status = 200, description = "Active refresh tokens", body = RefreshTokenListResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Account session required"),
    ),
    security(("bearer" = [])),
    tag = "Auth",
)]
pub async fn list_refresh_tokens(
    State(state): State<AppState>,
    RequireAccountSession(auth_user): RequireAccountSession,
) -> Result<axum::Json<RefreshTokenListResponse>, ApiError> {
    let families = state
        .auth_service
        .list_active_refresh_families(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;

    let tokens = families
        .iter()
        .map(RefreshTokenDetail::from_refresh_token)
        .collect();

    Ok(axum::Json(RefreshTokenListResponse { tokens }))
}

/// Request a password reset email.
#[utoipa::path(
    post,
    path = "/api/v1/auth/password/forgot",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "If the email exists, a reset link has been sent", body = MessageResponse),
        (status = 422, description = "Validation error"),
    ),
    tag = "Auth",
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<ForgotPasswordRequest>,
) -> Result<axum::Json<MessageResponse>, ApiError> {
    match state.auth_service.forgot_password(&body.email).await {
        Ok(_) => {}
        Err(e) => {
            tracing::warn!(error = %e, "forgot_password service error (suppressed to prevent enumeration)");
        }
    }

    Ok(axum::Json(MessageResponse {
        message: "If an account with that email exists, a password reset link has been sent."
            .to_string(),
    }))
}

/// Complete a password reset using a token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/password/reset",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful", body = AuthResponse),
        (status = 400, description = "Invalid or expired token"),
        (status = 422, description = "Validation error"),
    ),
    tag = "Auth",
)]
pub async fn reset_password(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<ResetPasswordRequest>,
) -> Result<Response, ApiError> {
    let user = state
        .auth_service
        .reset_password(&body.token, &body.new_password)
        .await
        .map_err(ApiError::from)?;

    let response_body = AuthResponse::from_user(&user);
    let body_bytes = serde_json::to_vec(&response_body).map_err(ApiError::from)?;

    let mut cookie_headers = HeaderMap::new();
    clear_refresh_cookie(&mut cookie_headers, &state.config);
    clear_asset_cookie(&mut cookie_headers, &state.config);

    let mut response = (StatusCode::OK, body_bytes).into_response();
    #[expect(
        clippy::unwrap_used,
        reason = "parsing a static ASCII literal into a header value is infallible"
    )]
    let content_type = "application/json".parse().unwrap();
    response
        .headers_mut()
        .insert(http::header::CONTENT_TYPE, content_type);
    response.headers_mut().extend(cookie_headers.drain());

    Ok(response)
}

/// Resend the email verification link.
#[utoipa::path(
    post,
    path = "/api/v1/auth/email/resend",
    responses(
        (status = 200, description = "Verification email sent (or already verified)", body = MessageResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Account session required"),
        (status = 429, description = "Rate limited"),
    ),
    security(("bearer" = [])),
    tag = "Auth",
)]
pub async fn resend_verification(
    State(state): State<AppState>,
    RequireAccountSession(auth_user): RequireAccountSession,
) -> Result<axum::Json<MessageResponse>, ApiError> {
    let result = state
        .auth_service
        .resend_verification(&auth_user.user_id)
        .await
        .map_err(ApiError::from)?;

    let message = if result.is_some() {
        "Verification email sent."
    } else {
        "Email is already verified."
    };

    Ok(axum::Json(MessageResponse {
        message: message.to_string(),
    }))
}

/// Verify email address using a token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/email/verify",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified", body = AuthResponse),
        (status = 400, description = "Invalid or expired token"),
        (status = 422, description = "Validation error"),
    ),
    tag = "Auth",
)]
pub async fn verify_email(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<VerifyEmailRequest>,
) -> Result<axum::Json<AuthResponse>, ApiError> {
    let user = state
        .auth_service
        .verify_email(&body.token)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(AuthResponse::from_user(&user)))
}
