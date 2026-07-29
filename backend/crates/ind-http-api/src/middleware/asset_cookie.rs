use axum::extract::FromRequestParts;
use http::HeaderMap;
use http::request::Parts;
use ind_auth::{ASSET_COOKIE_MAX_AGE_SECS, sign_asset_cookie, verify_asset_cookie};

use ind_application::asset_serving::AssetServingMode;
use ind_application::ports::UserLookup;
use ind_domain::{UserId, UserStatus};

use crate::error::ApiError;
use crate::state::{AppConfig, AppState, Environment};

const ASSET_COOKIE_NAME: &str = "ind_asset";

/// Set the `ind_asset` cookie on the response headers.
pub fn set_asset_cookie(headers: &mut HeaderMap, user_id: &UserId, config: &AppConfig) {
    let secret = match &config.asset_cookie_secret {
        Some(s) => s,
        None => return,
    };
    if !matches!(config.asset_serving_mode, AssetServingMode::Passthrough) {
        return;
    }

    let value = sign_asset_cookie(user_id, secret);
    let secure = config.environment != Environment::Development;
    let domain = cookie_domain_attribute(config);
    let cookie = format!(
        "{ASSET_COOKIE_NAME}={value}; Path=/api/v1/assets; HttpOnly; SameSite=Lax; Max-Age={ASSET_COOKIE_MAX_AGE_SECS}{domain}{secure}",
        secure = if secure { "; Secure" } else { "" },
    );
    #[expect(
        clippy::expect_used,
        reason = "cookie string is assembled from a signed token, an integer max-age, and config-derived attributes that contain only header-safe ASCII"
    )]
    let header_value = cookie.parse().expect("valid cookie header value");
    headers.append(http::header::SET_COOKIE, header_value);
}

/// Clear the `ind_asset` cookie.
pub fn clear_asset_cookie(headers: &mut HeaderMap, config: &AppConfig) {
    if !matches!(config.asset_serving_mode, AssetServingMode::Passthrough) {
        return;
    }
    let domain = cookie_domain_attribute(config);
    let cookie = format!(
        "{ASSET_COOKIE_NAME}=; Path=/api/v1/assets; HttpOnly; SameSite=Lax; Max-Age=0{domain}",
    );
    #[expect(
        clippy::expect_used,
        reason = "cookie string is a static template plus a config-derived domain attribute that contains only header-safe ASCII"
    )]
    let header_value = cookie.parse().expect("valid cookie header value");
    headers.append(http::header::SET_COOKIE, header_value);
}

fn cookie_domain_attribute(config: &AppConfig) -> String {
    config
        .cookie_domain
        .as_ref()
        .map(|domain| format!("; Domain={domain}"))
        .unwrap_or_default()
}

/// Extract the `ind_asset` cookie value from request headers.
fn extract_asset_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(http::header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|s| s.split(';'))
        .find_map(|cookie| {
            let cookie = cookie.trim();
            cookie.strip_prefix("ind_asset=").map(|v| v.to_string())
        })
}

async fn resolve_asset_cookie_user(
    headers: &HeaderMap,
    secret: Option<&[u8]>,
    user_lookup: &dyn UserLookup,
) -> Option<UserId> {
    let secret = secret?;
    let cookie_value = extract_asset_cookie(headers)?;
    let user_id = verify_asset_cookie(&cookie_value, secret)?;

    match user_lookup.get_user_by_id(user_id).await {
        Ok(Some(user)) if user.status == UserStatus::Active => Some(user_id),
        Ok(Some(user)) => {
            tracing::debug!(
                user_id = %user_id,
                status = ?user.status,
                "asset cookie user is not active; falling back to bearer auth"
            );
            None
        }
        Ok(None) => {
            tracing::debug!(
                user_id = %user_id,
                "asset cookie user not found; falling back to bearer auth"
            );
            None
        }
        Err(error) => {
            tracing::warn!(
                user_id = %user_id,
                error = %error,
                "asset cookie user lookup failed; falling back to bearer auth"
            );
            None
        }
    }
}

/// Asset access extractor. Validates via `ind_asset` cookie first,
/// falls back to Bearer token (JWT or API token).
pub struct AssetAccess {
    pub user_id: UserId,
}

impl FromRequestParts<AppState> for AssetAccess {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Try cookie-based auth first, but fall back to bearer auth if the
        // cookie is stale and no longer maps to an active user.
        if let Some(user_id) = resolve_asset_cookie_user(
            &parts.headers,
            state.config.asset_cookie_secret.as_deref(),
            state.user_lookup.as_ref(),
        )
        .await
        {
            return Ok(AssetAccess { user_id });
        }

        // Fall back to Bearer auth
        let auth_user = super::auth::AuthUser::from_request_parts(parts, state).await?;
        Ok(AssetAccess {
            user_id: auth_user.user_id,
        })
    }
}
