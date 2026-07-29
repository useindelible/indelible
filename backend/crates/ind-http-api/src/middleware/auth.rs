use http::HeaderMap;
use http::request::Parts;
use ind_domain::{ApiPermission, ApiTokenId, ClientType, User, UserId, UserStatus};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub enum ApiCredential {
    UserAccessJwt {
        client_type: ClientType,
    },
    PersonalAccessToken {
        token_id: ApiTokenId,
        permissions: Vec<ApiPermission>,
    },
}

#[derive(Debug, Clone)]
pub struct Principal {
    pub user: User,
    pub user_id: UserId,
    pub credential: ApiCredential,
}

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
    value.strip_prefix("Bearer ").map(ToOwned::to_owned)
}

pub(super) async fn extract_principal(
    parts: &mut Parts,
    state: &AppState,
) -> Result<Principal, ApiError> {
    let bearer_token =
        extract_bearer_token(&parts.headers).ok_or_else(|| ApiError::Unauthorized {
            message: "authentication required".to_string(),
        })?;

    let (user_id, credential) =
        if bearer_token.starts_with("ind_") {
            let validated = state
                .token_validator
                .validate_api_token(&bearer_token)
                .await
                .map_err(|error| ApiError::Unauthorized {
                    message: error.to_string(),
                })?;
            let token = validated.token;
            (
                token.user_id,
                ApiCredential::PersonalAccessToken {
                    token_id: token.id,
                    permissions: token.permissions,
                },
            )
        } else {
            let claims = ind_auth::validate_access_token(&bearer_token, &state.jwt_secret)
                .map_err(|error| ApiError::Unauthorized {
                    message: error.to_string(),
                })?;
            let user_id = claims.user_id().map_err(|error| ApiError::Unauthorized {
                message: error.to_string(),
            })?;
            let client_type = claims
                .client_type()
                .map_err(|error| ApiError::Unauthorized {
                    message: error.to_string(),
                })?;
            (user_id, ApiCredential::UserAccessJwt { client_type })
        };

    let user = state
        .user_lookup
        .get_user_by_id(user_id)
        .await
        .map_err(|error| ApiError::Unauthorized {
            message: error.to_string(),
        })?
        .ok_or_else(|| ApiError::Unauthorized {
            message: "user not found".to_string(),
        })?;
    let user = require_active(user)?;
    enforce_user_rate_limit(state, user.id).await?;

    Ok(Principal {
        user_id: user.id,
        user,
        credential,
    })
}

async fn enforce_user_rate_limit(state: &AppState, user_id: UserId) -> Result<(), ApiError> {
    match state.user_rate_limiter.check(&user_id.to_string()).await {
        Ok(()) => Ok(()),
        Err(_retry_after) => Err(ApiError::RateLimited),
    }
}
