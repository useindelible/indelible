use super::dto::{NativeOAuthErrorResponse, OAuthCallbackPayload};
use super::helpers::*;
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/auth/providers",
    responses(
        (status = 200, description = "List of OAuth providers", body = OAuthProvidersResponse),
    ),
    tag = "Auth",
)]
pub async fn list_providers(State(state): State<AppState>) -> axum::Json<OAuthProvidersResponse> {
    let providers = state
        .oauth_config
        .as_ref()
        .map(|config| {
            config
                .configured_providers()
                .into_iter()
                .map(|provider| OAuthProviderInfo {
                    id: provider_id(&provider),
                    name: provider_name(&provider, Some(config)),
                    enabled: true,
                })
                .collect()
        })
        .unwrap_or_default();

    // Fail closed: a read error must not open signups; assume users already exist.
    let setup_required = match state.auth_service.has_any_users().await {
        Ok(any) => !any,
        Err(_) => false,
    };
    let signups_enabled = state.config.allow_signups || setup_required;

    axum::Json(OAuthProvidersResponse {
        providers,
        signups_enabled,
        setup_required,
    })
}

/// Start an OAuth authorization flow.
#[utoipa::path(
    get,
    path = "/api/v1/auth/oauth/{provider}/start",
    params(
        ("provider" = String, Path, description = "OAuth provider (google, apple, oidc)"),
    ),
    responses(
        (status = 302, description = "Redirect to provider authorization URL"),
        (status = 400, description = "Provider not configured"),
    ),
    tag = "Auth",
)]
pub async fn oauth_start(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<Response, ApiError> {
    let provider = parse_provider(&provider)?;

    let oauth_service = state
        .oauth_service
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest {
            message: "OAuth not configured".to_string(),
        })?;

    let auth_url = oauth_service
        .oauth_start(provider)
        .await
        .map_err(ApiError::from)?;

    persist_oauth_flow(
        &state,
        StoredOAuthFlow {
            provider: provider_id(&provider),
            csrf_state: auth_url.csrf_state.clone(),
            issuer: auth_url.issuer,
            oidc_flow: auth_url.oidc_flow,
            kind: StoredOAuthFlowKind::Web,
            expires_at: Utc::now().timestamp() + OAUTH_FLOW_MAX_AGE_SECS,
        },
    )
    .await?;

    let response = Redirect::temporary(&auth_url.url).into_response();

    Ok(response)
}

/// Start a native mobile OAuth authorization flow.
#[utoipa::path(
    get,
    path = "/api/v1/auth/oauth/{provider}/native/start",
    params(
        ("provider" = String, Path, description = "OAuth provider (google, apple, oidc)"),
        ("platform" = String, Query, description = "Native platform (ios or android)"),
        ("code_challenge" = String, Query, description = "Mobile PKCE S256 challenge"),
        ("code_challenge_method" = String, Query, description = "Must be S256"),
        ("app_state" = String, Query, description = "Opaque mobile-generated state"),
    ),
    responses(
        (status = 302, description = "Redirect to provider authorization URL"),
        (status = 400, description = "Provider or native OAuth request is invalid"),
    ),
    tag = "Auth",
)]
pub async fn native_oauth_start(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<NativeOAuthStartQuery>,
) -> Result<Response, ApiError> {
    let provider = parse_provider(&provider)?;
    let platform = parse_native_platform(&query.platform)?;
    validate_native_start_query(&query)?;

    let oauth_service = state
        .oauth_service
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest {
            message: "OAuth not configured".to_string(),
        })?;

    let auth_url = oauth_service
        .oauth_start(provider)
        .await
        .map_err(ApiError::from)?;

    persist_oauth_flow(
        &state,
        StoredOAuthFlow {
            provider: provider_id(&provider),
            csrf_state: auth_url.csrf_state.clone(),
            issuer: auth_url.issuer,
            oidc_flow: auth_url.oidc_flow,
            kind: StoredOAuthFlowKind::Native(NativeOAuthFlow {
                platform,
                redirect_uri: NATIVE_OAUTH_REDIRECT_URI.to_string(),
                code_challenge: query.code_challenge,
                app_state: query.app_state,
            }),
            expires_at: Utc::now().timestamp() + OAUTH_FLOW_MAX_AGE_SECS,
        },
    )
    .await?;

    Ok(Redirect::temporary(&auth_url.url).into_response())
}

/// Handle the OAuth provider callback (GET).
#[utoipa::path(
    get,
    path = "/api/v1/auth/oauth/{provider}/callback",
    params(
        ("provider" = String, Path, description = "OAuth provider (google, apple, oidc)"),
        ("code" = String, Query, description = "Authorization code"),
        ("state" = String, Query, description = "CSRF state"),
    ),
    responses(
        (status = 302, description = "Redirect to frontend with session"),
        (status = 400, description = "OAuth error"),
    ),
    security(),
    tag = "Auth",
)]
pub async fn oauth_callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
    client_ip: ClientIp,
    request_headers: HeaderMap,
) -> Result<Response, ApiError> {
    handle_oauth_callback(
        state,
        provider,
        OAuthCallbackPayload {
            code: query.code,
            state: query.state,
            iss: query.iss,
            error: query.error,
            error_description: query.error_description,
        },
        client_ip.audit_string(),
        request_headers,
    )
    .await
}

/// Handle the OAuth provider callback (POST — Apple's form_post).
#[utoipa::path(
    post,
    path = "/api/v1/auth/oauth/{provider}/callback",
    params(
        ("provider" = String, Path, description = "OAuth provider (apple)"),
    ),
    request_body(
        content = OAuthCallbackForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 302, description = "Redirect to frontend with session"),
        (status = 400, description = "OAuth error"),
    ),
    security(),
    tag = "Auth",
)]
pub async fn oauth_callback_post(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    client_ip: ClientIp,
    request_headers: HeaderMap,
    axum::extract::Form(form): axum::extract::Form<OAuthCallbackForm>,
) -> Result<Response, ApiError> {
    handle_oauth_callback(
        state,
        provider,
        OAuthCallbackPayload {
            code: form.code,
            state: form.state,
            iss: form.iss,
            error: form.error,
            error_description: form.error_description,
        },
        client_ip.audit_string(),
        request_headers,
    )
    .await
}

async fn handle_oauth_callback(
    state: AppState,
    provider: String,
    callback: OAuthCallbackPayload,
    client_ip: Option<String>,
    request_headers: HeaderMap,
) -> Result<Response, ApiError> {
    let provider_enum = parse_provider(&provider)?;

    let oauth_service = state
        .oauth_service
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest {
            message: "OAuth not configured".to_string(),
        })?;

    let flow = load_oauth_flow(&state, &callback.state)
        .await?
        .ok_or_else(|| ApiError::BadRequest {
            message: "invalid OAuth state".to_string(),
        })?;
    if flow.provider != provider {
        return Err(ApiError::BadRequest {
            message: "OAuth provider mismatch".to_string(),
        });
    }
    if flow.expires_at < Utc::now().timestamp() {
        return Err(ApiError::BadRequest {
            message: "OAuth flow expired".to_string(),
        });
    }
    validate_callback_issuer(flow.issuer.as_deref(), callback.iss.as_deref())?;

    if let Some(error) = callback.error {
        let description = callback.error_description.as_deref().unwrap_or(&error);
        if let StoredOAuthFlowKind::Native(native) = &flow.kind {
            return Ok(native_error_redirect(native, &error, description));
        }
        return Err(ApiError::BadRequest {
            message: format!("OAuth provider returned error: {description}"),
        });
    }

    let Some(code) = callback.code else {
        if let StoredOAuthFlowKind::Native(native) = &flow.kind {
            return Ok(native_error_redirect(
                native,
                "invalid_request",
                "OAuth provider did not return an authorization code",
            ));
        }
        return Err(ApiError::BadRequest {
            message: "OAuth provider did not return an authorization code".to_string(),
        });
    };

    let ip = client_ip;
    let user_agent = extract_user_agent(&request_headers);

    let result = match oauth_service
        .oauth_callback(
            provider_enum,
            &code,
            &callback.state,
            OAuthCallbackContext {
                expected_state: flow.csrf_state,
                oidc_flow: flow.oidc_flow,
            },
        )
        .await
    {
        Ok(result) => result,
        Err(err) => {
            if let StoredOAuthFlowKind::Native(native) = &flow.kind {
                return Ok(native_error_redirect(
                    native,
                    "provider_error",
                    &err.to_string(),
                ));
            }
            return Err(ApiError::from(err));
        }
    };

    if let StoredOAuthFlowKind::Native(native) = flow.kind {
        return handle_native_oauth_success(state, provider, result, native, ip, user_agent).await;
    }

    // Create JWT + refresh token for the OAuth user
    let token_result = state
        .auth_service
        .create_tokens_for_user(result.user.id, ClientType::Web, ip, user_agent)
        .await
        .map_err(ApiError::from)?;

    let mut cookie_headers = HeaderMap::new();
    set_refresh_cookie(
        &mut cookie_headers,
        &token_result.raw_refresh_token,
        &state.config,
    );
    set_asset_cookie(&mut cookie_headers, &result.user.id, &state.config);

    let redirect_url = if result.is_new_user {
        format!(
            "{}/auth/callback?auth=signup&provider={}",
            state.config.frontend_url, provider
        )
    } else {
        format!(
            "{}/auth/callback?auth=login&provider={}",
            state.config.frontend_url, provider
        )
    };

    let mut response = Redirect::temporary(&redirect_url).into_response();
    response.headers_mut().extend(cookie_headers.drain());

    Ok(response)
}

async fn handle_native_oauth_success(
    state: AppState,
    provider: String,
    result: OAuthCallbackResult,
    native: NativeOAuthFlow,
    _ip: Option<String>,
    _user_agent: Option<String>,
) -> Result<Response, ApiError> {
    let code_ops = state
        .extension_auth_ops
        .as_ref()
        .ok_or_else(|| ApiError::Internal {
            message: "native OAuth code exchange not configured".to_string(),
        })?;

    let raw_code = code_ops
        .create_authorization_code(
            result.user.id,
            native.platform,
            native.code_challenge,
            "S256".to_string(),
            native.redirect_uri.clone(),
        )
        .await
        .map_err(ApiError::from)?;

    let auth = if result.is_new_user {
        "signup"
    } else {
        "login"
    };
    let redirect_url = format!(
        "{}?code={}&state={}&provider={}&auth={}",
        native.redirect_uri,
        url_encode(&raw_code),
        url_encode(&native.app_state),
        url_encode(&provider),
        auth,
    );

    Ok(Redirect::temporary(&redirect_url).into_response())
}

fn native_error_redirect(
    native: &NativeOAuthFlow,
    error_code: &str,
    description: &str,
) -> Response {
    let redirect_url = format!(
        "{}?error={}&error_code={}&error_description={}&state={}",
        native.redirect_uri,
        error_code,
        error_code,
        url_encode(description),
        url_encode(&native.app_state),
    );
    Redirect::temporary(&redirect_url).into_response()
}

/// Exchange a native OAuth one-time code for Indelible API tokens.
#[utoipa::path(
    post,
    path = "/api/v1/auth/oauth/native/token",
    request_body(
        content = NativeOAuthTokenForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 200, description = "Native OAuth tokens issued", body = NativeOAuthTokenResponse),
        (status = 400, description = "Invalid native OAuth grant", body = NativeOAuthErrorResponse),
    ),
    tag = "Auth",
)]
pub async fn native_oauth_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    client_ip: ClientIp,
    axum::extract::Form(form): axum::extract::Form<NativeOAuthTokenForm>,
) -> Result<Response, ApiError> {
    let client_type = detect_client_type(&headers);
    if !matches!(client_type, ClientType::Ios | ClientType::Android) {
        return Ok(native_token_error());
    }
    if form.grant_type != "authorization_code" || form.redirect_uri != NATIVE_OAUTH_REDIRECT_URI {
        return Ok(native_token_error());
    }

    let code_ops = state
        .extension_auth_ops
        .as_ref()
        .ok_or_else(|| ApiError::Internal {
            message: "native OAuth code exchange not configured".to_string(),
        })?;

    let ip = client_ip.audit_string();
    let user_agent = extract_user_agent(&headers);
    let result = match code_ops
        .exchange_authorization_code(
            &form.code,
            &form.code_verifier,
            &form.redirect_uri,
            client_type,
            ip,
            user_agent,
        )
        .await
    {
        Ok(result) => result,
        Err(_) => return Ok(native_token_error()),
    };

    let response_body = NativeOAuthTokenResponse {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        token_type: "Bearer",
        expires_at: result.expires_at,
        refresh_token_expires_at: result.refresh_token_expires_at,
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
    Ok(response)
}

fn native_token_error() -> Response {
    #[expect(
        clippy::expect_used,
        reason = "serializing a struct of static string fields to JSON is infallible"
    )]
    let body = serde_json::to_vec(&NativeOAuthErrorResponse {
        error: "invalid_grant",
        error_description: "invalid authorization grant",
    })
    .expect("serializing static OAuth error cannot fail");
    let mut response = (StatusCode::BAD_REQUEST, body).into_response();
    #[expect(
        clippy::unwrap_used,
        reason = "parsing a static ASCII literal into a header value is infallible"
    )]
    let content_type = "application/json".parse().unwrap();
    response
        .headers_mut()
        .insert(http::header::CONTENT_TYPE, content_type);
    response
}
