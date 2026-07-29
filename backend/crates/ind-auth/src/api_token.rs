use chrono::Utc;
use ind_application::repos::api_token::ApiTokenRepository;
pub use ind_domain::ApiPermission;
use ind_domain::{ApiToken, ApiTokenId, UserId};

use crate::crypto;
use crate::error::AuthError;
use validator::Validate as _;

pub const ALL_API_PERMISSIONS: &[ApiPermission] = &[
    ApiPermission::LibraryRead,
    ApiPermission::LibraryWrite,
    ApiPermission::FeedsRead,
    ApiPermission::FeedsWrite,
    ApiPermission::IntegrationsRead,
    ApiPermission::IntegrationsWrite,
    ApiPermission::WebhooksRead,
    ApiPermission::WebhooksWrite,
    ApiPermission::AiRead,
    ApiPermission::AiWrite,
    ApiPermission::AiUse,
    ApiPermission::ObsidianSync,
];

pub fn canonicalize_permissions(
    permissions: &[ApiPermission],
) -> Result<Vec<ApiPermission>, AuthError> {
    if permissions.is_empty() {
        return Err(AuthError::TokenInvalid);
    }

    let mut expanded = permissions.to_vec();
    for permission in permissions {
        if let Some(read_permission) = read_permission_for_write(*permission) {
            expanded.push(read_permission);
        }
    }

    Ok(ALL_API_PERMISSIONS
        .iter()
        .copied()
        .filter(|permission| expanded.contains(permission))
        .collect())
}

const fn read_permission_for_write(permission: ApiPermission) -> Option<ApiPermission> {
    match permission {
        ApiPermission::LibraryWrite => Some(ApiPermission::LibraryRead),
        ApiPermission::FeedsWrite => Some(ApiPermission::FeedsRead),
        ApiPermission::IntegrationsWrite => Some(ApiPermission::IntegrationsRead),
        ApiPermission::WebhooksWrite => Some(ApiPermission::WebhooksRead),
        ApiPermission::AiWrite => Some(ApiPermission::AiRead),
        _ => None,
    }
}

// -- Request / Response DTOs --

#[derive(validator::Validate)]
pub struct CreateApiTokenRequest {
    pub user_id: UserId,
    #[validate(custom(function = "crate::validation::trimmed_non_blank"))]
    #[validate(custom(function = "crate::validation::trimmed_max_api_token_name_length"))]
    pub name: String,
    #[validate(custom(function = "crate::validation::non_empty_token_permissions"))]
    pub permissions: Vec<ApiPermission>,
    #[validate(custom(function = "crate::validation::valid_api_token_expiry"))]
    pub expires_in: Option<chrono::Duration>,
}

pub struct CreateApiTokenResponse {
    pub token: ApiToken,
    pub raw_token: String,
}

pub struct ValidatedToken {
    pub token: ApiToken,
}

// -- Service --

pub struct ApiTokenService<R: ApiTokenRepository> {
    repo: R,
}

impl<R: ApiTokenRepository> ApiTokenService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_api_token(
        &self,
        request: CreateApiTokenRequest,
    ) -> Result<CreateApiTokenResponse, AuthError> {
        if request.validate().is_err() {
            return Err(AuthError::TokenInvalid);
        }

        let permissions = canonicalize_permissions(&request.permissions)?;

        let trimmed_name = request.name.trim();

        let raw_token = crypto::generate_api_token();
        let token_hash = crypto::hash_token(&raw_token);
        let prefix = raw_token[..8].to_string();
        let now = Utc::now();
        let expires_at = request.expires_in.map(|d| now + d);

        let api_token = ApiToken {
            id: ApiTokenId::new(),
            user_id: request.user_id,
            name: trimmed_name.to_owned(),
            token_hash,
            prefix,
            permissions,
            last_used_at: None,
            expires_at,
            created_at: now,
        };

        let persisted = self.repo.create(api_token).await?;

        Ok(CreateApiTokenResponse {
            token: persisted,
            raw_token,
        })
    }

    pub async fn list_api_tokens(&self, user_id: UserId) -> Result<Vec<ApiToken>, AuthError> {
        Ok(self.repo.list_by_user(user_id).await?)
    }

    pub async fn revoke_api_token(
        &self,
        user_id: UserId,
        token_id: ApiTokenId,
    ) -> Result<(), AuthError> {
        let token = self
            .repo
            .find_by_id(token_id, user_id)
            .await?
            .ok_or(AuthError::TokenInvalid)?;

        if token.user_id != user_id {
            return Err(AuthError::TokenInvalid);
        }

        self.repo.delete(token_id, user_id).await?;
        Ok(())
    }

    pub async fn revoke_all_api_tokens(&self, user_id: UserId) -> Result<u64, AuthError> {
        let tokens = self.repo.list_by_user(user_id).await?;
        let count = tokens.len() as u64;
        for token in &tokens {
            self.repo.delete(token.id, user_id).await?;
        }
        Ok(count)
    }

    pub async fn validate_api_token(&self, raw_token: &str) -> Result<ValidatedToken, AuthError> {
        if !raw_token.starts_with("ind_") {
            return Err(AuthError::TokenInvalid);
        }

        let token_hash = crypto::hash_token(raw_token);
        let token = self
            .repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(AuthError::TokenInvalid)?;

        if let Some(expires_at) = token.expires_at
            && expires_at < Utc::now()
        {
            return Err(AuthError::TokenExpired);
        }

        self.repo.update_last_used(token.id).await?;

        Ok(ValidatedToken { token })
    }
}
