use std::marker::PhantomData;

use axum::extract::FromRequestParts;
use http::request::Parts;
use ind_domain::{ApiPermission, ClientType};

use crate::error::ApiError;
use crate::middleware::auth::extract_principal;
use crate::middleware::{ApiCredential, Principal};
use crate::state::AppState;

pub trait AccessPolicy: Send + Sync + 'static {
    const REQUIRED: &'static [ApiPermission];
    const ALLOW_EXTENSION_JWT: bool = false;
}

pub struct LibraryReadPolicy;

impl AccessPolicy for LibraryReadPolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::LibraryRead];
}

pub struct LibraryWritePolicy;

impl AccessPolicy for LibraryWritePolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::LibraryWrite];
}

pub struct DocumentAssetPolicy;

impl AccessPolicy for DocumentAssetPolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::LibraryRead];
    const ALLOW_EXTENSION_JWT: bool = true;
}

pub struct FeedsReadPolicy;

impl AccessPolicy for FeedsReadPolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::FeedsRead];
}

pub struct FeedsWritePolicy;

impl AccessPolicy for FeedsWritePolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::FeedsWrite];
}

pub struct IntegrationsReadPolicy;

impl AccessPolicy for IntegrationsReadPolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::IntegrationsRead];
}

pub struct IntegrationsWritePolicy;

impl AccessPolicy for IntegrationsWritePolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::IntegrationsWrite];
}

pub struct WebhooksReadPolicy;

impl AccessPolicy for WebhooksReadPolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::WebhooksRead];
}

pub struct WebhooksWritePolicy;

impl AccessPolicy for WebhooksWritePolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::WebhooksWrite];
}

pub struct AiReadPolicy;

impl AccessPolicy for AiReadPolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::AiRead];
}

pub struct AiWritePolicy;

impl AccessPolicy for AiWritePolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::AiWrite];
}

pub struct AiUsePolicy;

impl AccessPolicy for AiUsePolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::AiUse];
}

pub struct ObsidianSyncPolicy;

impl AccessPolicy for ObsidianSyncPolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::ObsidianSync];
}

pub struct AiReadAndLibraryReadPolicy;

impl AccessPolicy for AiReadAndLibraryReadPolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::AiRead, ApiPermission::LibraryRead];
}

pub struct AiUseAndLibraryReadPolicy;

impl AccessPolicy for AiUseAndLibraryReadPolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::AiUse, ApiPermission::LibraryRead];
}

pub struct AiWriteAndAiUsePolicy;

impl AccessPolicy for AiWriteAndAiUsePolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::AiWrite, ApiPermission::AiUse];
}

pub struct AiWriteAndAiUseAndLibraryReadPolicy;

impl AccessPolicy for AiWriteAndAiUseAndLibraryReadPolicy {
    const REQUIRED: &'static [ApiPermission] = &[
        ApiPermission::AiWrite,
        ApiPermission::AiUse,
        ApiPermission::LibraryRead,
    ];
}

pub struct PermissionAccess<P: AccessPolicy> {
    pub principal: Principal,
    marker: PhantomData<P>,
}

pub type RequireLibraryRead = PermissionAccess<LibraryReadPolicy>;
pub type RequireLibraryWrite = PermissionAccess<LibraryWritePolicy>;
pub type RequireDocumentAssetRead = PermissionAccess<DocumentAssetPolicy>;
pub type RequireFeedsRead = PermissionAccess<FeedsReadPolicy>;
pub type RequireFeedsWrite = PermissionAccess<FeedsWritePolicy>;
pub type RequireIntegrationsRead = PermissionAccess<IntegrationsReadPolicy>;
pub type RequireIntegrationsWrite = PermissionAccess<IntegrationsWritePolicy>;
pub type RequireWebhooksRead = PermissionAccess<WebhooksReadPolicy>;
pub type RequireWebhooksWrite = PermissionAccess<WebhooksWritePolicy>;
pub type RequireAiRead = PermissionAccess<AiReadPolicy>;
pub type RequireAiWrite = PermissionAccess<AiWritePolicy>;
pub type RequireAiUse = PermissionAccess<AiUsePolicy>;
pub type RequireObsidianSync = PermissionAccess<ObsidianSyncPolicy>;
pub type RequireAiReadAndLibraryRead = PermissionAccess<AiReadAndLibraryReadPolicy>;
pub type RequireAiUseAndLibraryRead = PermissionAccess<AiUseAndLibraryReadPolicy>;
pub type RequireAiWriteAndAiUse = PermissionAccess<AiWriteAndAiUsePolicy>;
pub type RequireAiWriteAndAiUseAndLibraryRead =
    PermissionAccess<AiWriteAndAiUseAndLibraryReadPolicy>;

pub(crate) fn authorize_permission_access<P: AccessPolicy>(
    principal: &Principal,
) -> Result<(), ApiError> {
    match &principal.credential {
        ApiCredential::UserAccessJwt {
            client_type: ClientType::Extension,
        } if !P::ALLOW_EXTENSION_JWT => {
            return Err(ApiError::Forbidden {
                message: "extension access is not permitted for this resource".to_string(),
            });
        }
        ApiCredential::PersonalAccessToken { permissions, .. }
            if P::REQUIRED.is_empty()
                || !P::REQUIRED
                    .iter()
                    .all(|required| permissions.contains(required)) =>
        {
            return Err(ApiError::InsufficientPermissions {
                required: P::REQUIRED.to_vec(),
            });
        }
        ApiCredential::UserAccessJwt { .. } | ApiCredential::PersonalAccessToken { .. } => {}
    }

    if !principal.user.email_verified {
        return Err(ApiError::Forbidden {
            message: "email verification required".to_string(),
        });
    }

    Ok(())
}

impl<P: AccessPolicy> FromRequestParts<AppState> for PermissionAccess<P> {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let principal = extract_principal(parts, state).await?;
        authorize_permission_access::<P>(&principal)?;
        Ok(Self {
            principal,
            marker: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests;
