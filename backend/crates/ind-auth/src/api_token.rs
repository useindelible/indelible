use chrono::Utc;
use ind_application::repos::api_token::ApiTokenRepository;
pub use ind_domain::TokenScope;
use ind_domain::{ApiToken, ApiTokenId, UserId};

use crate::crypto;
use crate::error::AuthError;
use validator::Validate as _;

pub fn scopes_to_strings(scopes: &[TokenScope]) -> Vec<String> {
    scopes.iter().map(|s| s.as_str().to_owned()).collect()
}

pub fn strings_to_scopes(strings: &[String]) -> Result<Vec<TokenScope>, AuthError> {
    strings
        .iter()
        .map(|scope| scope.parse().map_err(|_| AuthError::TokenInvalid))
        .collect()
}

// -- Request / Response DTOs --

#[derive(validator::Validate)]
pub struct CreateApiTokenRequest {
    pub user_id: UserId,
    #[validate(custom(function = "crate::validation::trimmed_non_blank"))]
    #[validate(custom(function = "crate::validation::trimmed_max_api_token_name_length"))]
    pub name: String,
    #[validate(custom(function = "crate::validation::non_empty_token_scopes"))]
    pub scopes: Vec<TokenScope>,
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
            scopes: scopes_to_strings(&request.scopes),
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

/// Checks whether the given API token contains the required scope.
pub fn has_scope(token: &ApiToken, required: TokenScope) -> bool {
    token.scopes.iter().any(|s| s == required.as_str())
}
