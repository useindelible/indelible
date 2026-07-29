use axum::extract::FromRequestParts;
use http::request::Parts;
use ind_domain::ClientType;

use crate::error::ApiError;
use crate::middleware::Principal;
use crate::middleware::auth::extract_principal;
use crate::state::AppState;

#[derive(Clone, Copy)]
struct JwtAccessPolicy {
    allowed_clients: &'static [ClientType],
    require_verified_email: bool,
    denial_message: &'static str,
}

const ORDINARY_CLIENTS: &[ClientType] = &[
    ClientType::Web,
    ClientType::Ios,
    ClientType::Android,
    ClientType::Desktop,
    ClientType::Cli,
];

const USER_ACCESS_POLICY: JwtAccessPolicy = JwtAccessPolicy {
    allowed_clients: ORDINARY_CLIENTS,
    require_verified_email: false,
    denial_message: "account session required",
};

const VERIFIED_USER_ACCESS_POLICY: JwtAccessPolicy = JwtAccessPolicy {
    allowed_clients: ORDINARY_CLIENTS,
    require_verified_email: true,
    denial_message: "user access JWT required",
};

const WEB_ACCESS_POLICY: JwtAccessPolicy = JwtAccessPolicy {
    allowed_clients: &[ClientType::Web],
    require_verified_email: false,
    denial_message: "web access required",
};

const VERIFIED_WEB_ACCESS_POLICY: JwtAccessPolicy = JwtAccessPolicy {
    allowed_clients: &[ClientType::Web],
    require_verified_email: true,
    denial_message: "web access required",
};

const EXTENSION_ACCESS_POLICY: JwtAccessPolicy = JwtAccessPolicy {
    allowed_clients: &[ClientType::Extension],
    require_verified_email: false,
    denial_message: "extension access required",
};

const MOBILE_ACCESS_POLICY: JwtAccessPolicy = JwtAccessPolicy {
    allowed_clients: &[ClientType::Ios, ClientType::Android],
    require_verified_email: false,
    denial_message: "mobile access required",
};

fn authorize_jwt_access(principal: &Principal, policy: JwtAccessPolicy) -> Result<(), ApiError> {
    let crate::middleware::ApiCredential::UserAccessJwt { client_type } = &principal.credential
    else {
        return Err(ApiError::Forbidden {
            message: policy.denial_message.to_string(),
        });
    };

    if !policy.allowed_clients.contains(client_type) {
        return Err(ApiError::Forbidden {
            message: policy.denial_message.to_string(),
        });
    }

    if policy.require_verified_email && !principal.user.email_verified {
        return Err(ApiError::Forbidden {
            message: "email verification required".to_string(),
        });
    }

    Ok(())
}

async fn extract_jwt_access(
    parts: &mut Parts,
    state: &AppState,
    policy: JwtAccessPolicy,
) -> Result<Principal, ApiError> {
    let principal = extract_principal(parts, state).await?;
    authorize_jwt_access(&principal, policy)?;
    Ok(principal)
}

pub struct RequireUserAccessJwt(pub Principal);
pub struct RequireVerifiedUserAccessJwt(pub Principal);
pub struct RequireVerifiedWebAccessJwt(pub Principal);
pub struct RequireExtensionAccessJwt(pub Principal);
pub struct RequireMobileAccessJwt(pub Principal);

pub struct RequireWebAccess(pub Principal);

impl FromRequestParts<AppState> for RequireUserAccessJwt {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(
            extract_jwt_access(parts, state, USER_ACCESS_POLICY).await?,
        ))
    }
}

impl FromRequestParts<AppState> for RequireVerifiedUserAccessJwt {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(
            extract_jwt_access(parts, state, VERIFIED_USER_ACCESS_POLICY).await?,
        ))
    }
}

impl FromRequestParts<AppState> for RequireWebAccess {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(
            extract_jwt_access(parts, state, WEB_ACCESS_POLICY).await?,
        ))
    }
}

impl FromRequestParts<AppState> for RequireVerifiedWebAccessJwt {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(
            extract_jwt_access(parts, state, VERIFIED_WEB_ACCESS_POLICY).await?,
        ))
    }
}

impl FromRequestParts<AppState> for RequireExtensionAccessJwt {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(
            extract_jwt_access(parts, state, EXTENSION_ACCESS_POLICY).await?,
        ))
    }
}

impl FromRequestParts<AppState> for RequireMobileAccessJwt {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(
            extract_jwt_access(parts, state, MOBILE_ACCESS_POLICY).await?,
        ))
    }
}

#[cfg(test)]
mod tests;
