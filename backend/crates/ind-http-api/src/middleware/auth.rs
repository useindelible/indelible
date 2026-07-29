use std::convert::Infallible;

use axum::extract::FromRequestParts;
use http::HeaderMap;
use http::request::Parts;
use ind_auth::{JwtClaims, TokenScope, has_scope};
use ind_domain::{ApiToken, ClientType, User, UserId, UserStatus};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub enum AuthMethod {
    Jwt(JwtClaims),
    ApiToken(ApiToken),
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user: User,
    pub user_id: UserId,
    pub auth_method: AuthMethod,
}

pub struct OptionalAuthUser(pub Option<AuthUser>);

pub struct AccountAccess(pub AuthUser);

pub struct RequireWebAccess(pub AuthUser);
pub struct RequireExtensionAccess(pub AuthUser);
pub struct RequireMobileAccess(pub AuthUser);
pub struct RequireApiToken(pub AuthUser);
pub struct RequireObsidianPluginScope(pub AuthUser);

/// Accepts any valid, active user regardless of client type (web, extension, mobile, API token).
pub struct ContentAccess(pub AuthUser);

fn require_active(user: User) -> Result<User, ApiError> {
    if user.status != UserStatus::Active {
        return Err(ApiError::Unauthorized {
            message: "account is not active".to_string(),
        });
    }
    Ok(user)
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(http::header::AUTHORIZATION)?.to_str().ok()?;
    value.strip_prefix("Bearer ").map(|v| v.to_string())
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(bearer_token) = extract_bearer_token(&parts.headers) {
            if bearer_token.starts_with("ind_") {
                // Personal API token
                let validated = state
                    .token_validator
                    .validate_api_token(&bearer_token)
                    .await
                    .map_err(|e| ApiError::Unauthorized {
                        message: e.to_string(),
                    })?;

                let user = state
                    .user_lookup
                    .get_user_by_id(validated.token.user_id)
                    .await
                    .map_err(|e| ApiError::Unauthorized {
                        message: e.to_string(),
                    })?
                    .ok_or_else(|| ApiError::Unauthorized {
                        message: "user not found".to_string(),
                    })?;
                let user = require_active(user)?;
                enforce_user_rate_limit(state, user.id).await?;

                return Ok(AuthUser {
                    user_id: user.id,
                    user,
                    auth_method: AuthMethod::ApiToken(validated.token),
                });
            }

            // JWT access token
            let claims = ind_auth::validate_access_token(&bearer_token, &state.jwt_secret)
                .map_err(|e| ApiError::Unauthorized {
                    message: e.to_string(),
                })?;

            let user_id = claims.user_id().map_err(|e| ApiError::Unauthorized {
                message: e.to_string(),
            })?;

            let user = state
                .user_lookup
                .get_user_by_id(user_id)
                .await
                .map_err(|e| ApiError::Unauthorized {
                    message: e.to_string(),
                })?
                .ok_or_else(|| ApiError::Unauthorized {
                    message: "user not found".to_string(),
                })?;
            let user = require_active(user)?;
            enforce_user_rate_limit(state, user.id).await?;

            return Ok(AuthUser {
                user_id: user.id,
                user,
                auth_method: AuthMethod::Jwt(claims),
            });
        }

        Err(ApiError::Unauthorized {
            message: "authentication required".to_string(),
        })
    }
}

/// Per-user authenticated rate limit (L.1/L.3/L.5). Checked once the caller is
/// resolved so it applies to every authenticated route regardless of extractor.
async fn enforce_user_rate_limit(state: &AppState, user_id: UserId) -> Result<(), ApiError> {
    match state.user_rate_limiter.check(&user_id.to_string()).await {
        Ok(()) => Ok(()),
        Err(_retry_after) => Err(ApiError::RateLimited),
    }
}

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(OptionalAuthUser(
            AuthUser::from_request_parts(parts, state).await.ok(),
        ))
    }
}

impl FromRequestParts<AppState> for AccountAccess {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;

        // Extension tokens are scoped to content APIs only
        if let AuthMethod::Jwt(ref claims) = auth_user.auth_method
            && claims.client_type() == ClientType::Extension
        {
            return Err(ApiError::Forbidden {
                message: "insufficient access".to_string(),
            });
        }

        // Integration-plugin PATs (obsidian_plugin) never grant general account
        // access, even when paired with broader scopes. Routes that need them must
        // opt in via a dedicated scope extractor (e.g. RequireObsidianPluginScope).
        if let AuthMethod::ApiToken(ref token) = auth_user.auth_method
            && token
                .scopes
                .iter()
                .any(|s| s == TokenScope::ObsidianPlugin.as_str())
        {
            return Err(ApiError::Forbidden {
                message: "insufficient scope".to_string(),
            });
        }

        if !auth_user.user.email_verified {
            return Err(ApiError::Forbidden {
                message: "email verification required".to_string(),
            });
        }

        Ok(AccountAccess(auth_user))
    }
}

impl FromRequestParts<AppState> for RequireWebAccess {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        match &auth_user.auth_method {
            AuthMethod::Jwt(claims) if claims.client_type() == ClientType::Web => {
                Ok(RequireWebAccess(auth_user))
            }
            _ => Err(ApiError::Forbidden {
                message: "web access required".to_string(),
            }),
        }
    }
}

impl FromRequestParts<AppState> for RequireExtensionAccess {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        match &auth_user.auth_method {
            AuthMethod::Jwt(claims) if claims.client_type() == ClientType::Extension => {
                Ok(RequireExtensionAccess(auth_user))
            }
            _ => Err(ApiError::Forbidden {
                message: "extension access required".to_string(),
            }),
        }
    }
}

impl FromRequestParts<AppState> for RequireMobileAccess {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        match &auth_user.auth_method {
            AuthMethod::Jwt(claims)
                if matches!(claims.client_type(), ClientType::Ios | ClientType::Android) =>
            {
                Ok(RequireMobileAccess(auth_user))
            }
            _ => Err(ApiError::Forbidden {
                message: "mobile access required".to_string(),
            }),
        }
    }
}

impl FromRequestParts<AppState> for ContentAccess {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        Ok(ContentAccess(auth_user))
    }
}

impl FromRequestParts<AppState> for RequireApiToken {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        match &auth_user.auth_method {
            AuthMethod::ApiToken(_) => Ok(RequireApiToken(auth_user)),
            _ => Err(ApiError::Forbidden {
                message: "API token required".to_string(),
            }),
        }
    }
}

impl FromRequestParts<AppState> for RequireObsidianPluginScope {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        let token = match &auth_user.auth_method {
            AuthMethod::ApiToken(token) => token,
            _ => {
                return Err(ApiError::Forbidden {
                    message: "obsidian_plugin scope required".to_string(),
                });
            }
        };

        if !has_scope(token, TokenScope::ObsidianPlugin) {
            return Err(ApiError::Forbidden {
                message: "obsidian_plugin scope required".to_string(),
            });
        }

        Ok(RequireObsidianPluginScope(auth_user))
    }
}
