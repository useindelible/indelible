use axum::extract::FromRequestParts;
use http::HeaderMap;
use http::request::Parts;
use ind_auth::{ASSET_COOKIE_MAX_AGE_SECS, sign_asset_cookie, verify_asset_cookie};
use ind_domain::UserId;

use ind_application::ports::UserLookup;
use ind_domain::UserStatus;

use crate::error::ApiError;
use crate::middleware::jwt_access::RequireUserAccessJwt;
use crate::middleware::permission_access::{
    AccessPolicy, AiReadAndLibraryReadPolicy, DocumentAssetPolicy, PermissionAccess,
};
use crate::state::{AppConfig, AppState, Environment};

const ASSET_COOKIE_NAME: &str = "ind_asset";

/// Set the `ind_asset` cookie on the response headers. Asset URLs point at
/// the API in every serving mode, so the cookie is issued unconditionally;
/// in `presigned` mode it authenticates the redirect request.
pub fn set_asset_cookie(headers: &mut HeaderMap, user_id: &UserId, config: &AppConfig) {
    let secret = match &config.asset_cookie_secret {
        Some(s) => s,
        None => return,
    };

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
        Ok(Some(user)) if user.status == UserStatus::Active && user.email_verified => Some(user_id),
        Ok(Some(user)) => {
            tracing::debug!(
                user_id = %user_id,
                status = ?user.status,
                email_verified = user.email_verified,
                "asset cookie user is not active or email-verified; falling back to bearer auth"
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

pub struct AvatarAssetAccess {
    pub user_id: UserId,
}

pub struct DocumentAssetAccess {
    pub user_id: UserId,
}

pub struct TtsAssetAccess {
    pub user_id: UserId,
}

async fn extract_cookie_or_permission<P: AccessPolicy>(
    parts: &mut Parts,
    state: &AppState,
) -> Result<UserId, ApiError> {
    if let Some(user_id) = resolve_asset_cookie_user(
        &parts.headers,
        state.config.asset_cookie_secret.as_deref(),
        state.user_lookup.as_ref(),
    )
    .await
    {
        return Ok(user_id);
    }

    let access = PermissionAccess::<P>::from_request_parts(parts, state).await?;
    Ok(access.principal.user_id)
}

impl FromRequestParts<AppState> for AvatarAssetAccess {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(user_id) = resolve_asset_cookie_user(
            &parts.headers,
            state.config.asset_cookie_secret.as_deref(),
            state.user_lookup.as_ref(),
        )
        .await
        {
            return Ok(AvatarAssetAccess { user_id });
        }

        let RequireUserAccessJwt(principal) =
            RequireUserAccessJwt::from_request_parts(parts, state).await?;
        Ok(AvatarAssetAccess {
            user_id: principal.user_id,
        })
    }
}

impl FromRequestParts<AppState> for DocumentAssetAccess {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            user_id: extract_cookie_or_permission::<DocumentAssetPolicy>(parts, state).await?,
        })
    }
}

impl FromRequestParts<AppState> for TtsAssetAccess {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            user_id: extract_cookie_or_permission::<AiReadAndLibraryReadPolicy>(parts, state)
                .await?,
        })
    }
}
