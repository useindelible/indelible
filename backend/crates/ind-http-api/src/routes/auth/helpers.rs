use super::*;

pub(super) fn detect_client_type(headers: &HeaderMap) -> ClientType {
    if let Some(ct) = headers.get("x-client-type").and_then(|v| v.to_str().ok()) {
        return match ct {
            "ios" => ClientType::Ios,
            "android" => ClientType::Android,
            "desktop" => ClientType::Desktop,
            "extension" => ClientType::Extension,
            "cli" => ClientType::Cli,
            _ => ClientType::Web,
        };
    }
    ClientType::Web
}

pub(super) fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Returns true for client types that should receive the raw refresh token
/// in the response body (non-browser clients).
pub(super) fn is_non_browser_client(client_type: ClientType) -> bool {
    matches!(
        client_type,
        ClientType::Cli | ClientType::Ios | ClientType::Android | ClientType::Desktop
    )
}

pub(super) fn extract_body_refresh_token(body: &str) -> Option<String> {
    if body.trim().is_empty() {
        return None;
    }

    serde_json::from_str::<RefreshTokenRequest>(body)
        .ok()
        .map(|request| request.refresh_token)
}

pub(super) fn refresh_token_for_body(
    client_type: ClientType,
    raw_refresh_token: &str,
) -> Option<String> {
    if is_non_browser_client(client_type) && !raw_refresh_token.is_empty() {
        Some(raw_refresh_token.to_string())
    } else {
        None
    }
}

pub(super) fn set_auth_cookies_if_browser(
    headers: &mut HeaderMap,
    client_type: ClientType,
    raw_refresh_token: &str,
    user_id: &UserId,
    config: &AppConfig,
) {
    if is_non_browser_client(client_type) {
        return;
    }
    set_refresh_cookie(headers, raw_refresh_token, config);
    set_asset_cookie(headers, user_id, config);
}

pub(super) fn url_encode(value: &str) -> String {
    form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

// ---------------------------------------------------------------------------
// Provider helpers
// ---------------------------------------------------------------------------

pub(super) fn parse_provider(s: &str) -> Result<OAuthProvider, ApiError> {
    match s {
        "google" => Ok(OAuthProvider::Google),
        "apple" => Ok(OAuthProvider::Apple),
        "oidc" => Ok(OAuthProvider::Oidc),
        _ => Err(ApiError::BadRequest {
            message: format!("unknown OAuth provider: {s}"),
        }),
    }
}

pub(super) fn parse_native_platform(s: &str) -> Result<ClientType, ApiError> {
    match s {
        "ios" => Ok(ClientType::Ios),
        "android" => Ok(ClientType::Android),
        _ => Err(ApiError::BadRequest {
            message: "platform must be ios or android".to_string(),
        }),
    }
}

pub(super) fn validate_native_start_query(query: &NativeOAuthStartQuery) -> Result<(), ApiError> {
    if query.code_challenge_method != "S256" {
        return Err(ApiError::BadRequest {
            message: "code_challenge_method must be S256".to_string(),
        });
    }
    if !is_base64url_token(&query.code_challenge)
        || query.code_challenge.len() < NATIVE_PKCE_CHALLENGE_MIN_LEN
        || query.code_challenge.len() > NATIVE_PKCE_CHALLENGE_MAX_LEN
    {
        return Err(ApiError::BadRequest {
            message: "invalid code_challenge".to_string(),
        });
    }
    if !is_base64url_token(&query.app_state) || query.app_state.len() < NATIVE_APP_STATE_MIN_LEN {
        return Err(ApiError::BadRequest {
            message: "invalid app_state".to_string(),
        });
    }
    Ok(())
}

pub(super) fn is_base64url_token(value: &str) -> bool {
    value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub(super) fn validate_callback_issuer(
    expected_issuer: Option<&str>,
    callback_issuer: Option<&str>,
) -> Result<(), ApiError> {
    match (expected_issuer, callback_issuer) {
        (Some(expected), Some(actual)) if expected != actual => Err(ApiError::BadRequest {
            message: "OAuth issuer mismatch".to_string(),
        }),
        _ => Ok(()),
    }
}

pub(super) fn provider_id(p: &OAuthProvider) -> String {
    match p {
        OAuthProvider::Google => "google".to_string(),
        OAuthProvider::Apple => "apple".to_string(),
        OAuthProvider::Oidc => "oidc".to_string(),
    }
}

pub(super) fn provider_name(
    p: &OAuthProvider,
    config: Option<&ind_auth::oauth::OAuthConfig>,
) -> String {
    match p {
        OAuthProvider::Google => "Google".to_string(),
        OAuthProvider::Apple => "Apple".to_string(),
        OAuthProvider::Oidc => config
            .and_then(|config| config.oidc())
            .map(|oidc| oidc.provider_name.clone())
            .unwrap_or_else(|| "OpenID Connect".to_string()),
    }
}

pub(super) async fn persist_oauth_flow(
    state: &AppState,
    flow: StoredOAuthFlow,
) -> Result<(), ApiError> {
    let repo = state
        .oauth_flow_repo
        .as_ref()
        .ok_or_else(|| ApiError::Internal {
            message: "OAuth flow repository not configured".to_string(),
        })?;

    store_stored_oauth_flow(repo.as_ref(), &flow, &state.config.csrf_secret)
        .await
        .map_err(oauth_flow_storage_error)
}

pub(super) async fn load_oauth_flow(
    state: &AppState,
    raw_state: &str,
) -> Result<Option<StoredOAuthFlow>, ApiError> {
    let repo = state
        .oauth_flow_repo
        .as_ref()
        .ok_or_else(|| ApiError::Internal {
            message: "OAuth flow repository not configured".to_string(),
        })?;

    consume_stored_oauth_flow(repo.as_ref(), raw_state, &state.config.csrf_secret)
        .await
        .map_err(oauth_flow_storage_error)
}

pub(super) fn oauth_flow_storage_error(error: OAuthFlowStorageError) -> ApiError {
    match error {
        OAuthFlowStorageError::Seal(error) => ApiError::Internal {
            message: format!("failed to seal OAuth flow: {error}"),
        },
        OAuthFlowStorageError::Invalid(_) => ApiError::BadRequest {
            message: "invalid OAuth flow".to_string(),
        },
        OAuthFlowStorageError::InvalidExpiration => ApiError::Internal {
            message: "invalid OAuth flow expiration".to_string(),
        },
        OAuthFlowStorageError::Repository(error) => ApiError::from(error),
    }
}
