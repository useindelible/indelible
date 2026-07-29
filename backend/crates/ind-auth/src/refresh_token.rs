use std::sync::Arc;

use chrono::{Duration, Utc};
use ind_application::repos::refresh_token::RefreshTokenRepository;
use ind_domain::{ClientType, RefreshToken, RefreshTokenId, UserId};

use crate::crypto::{generate_refresh_token, hash_token};
use crate::error::AuthError;
use crate::jwt;

const IDLE_TIMEOUT_DAYS: i64 = 30;
const ABSOLUTE_LIFETIME_DAYS: i64 = 90;
const GRACE_WINDOW_SECS: i64 = 10;
const DEFAULT_SCOPES: &[&str] = &["read", "write"];

#[derive(Debug)]
pub struct RefreshResult {
    pub access_token: String,
    pub expires_at: i64,
    pub raw_refresh_token: String,
}

#[derive(Debug)]
pub struct TokenPair {
    pub access_token: String,
    pub expires_at: i64,
    pub raw_refresh_token: String,
    pub refresh_token: RefreshToken,
}

pub struct RefreshTokenService {
    repo: Arc<dyn RefreshTokenRepository>,
    jwt_secret: Vec<u8>,
}

impl RefreshTokenService {
    pub fn new(repo: Arc<dyn RefreshTokenRepository>, jwt_secret: Vec<u8>) -> Self {
        Self { repo, jwt_secret }
    }

    pub async fn issue_tokens(
        &self,
        user_id: UserId,
        client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<TokenPair, AuthError> {
        let now = Utc::now();
        let family_id = uuid::Uuid::now_v7();

        let raw_refresh = generate_refresh_token();
        let token_hash = hash_token(&raw_refresh);

        let refresh_token = RefreshToken {
            id: RefreshTokenId::new(),
            family_id,
            user_id,
            token_hash,
            client_type,
            ip_address: ip,
            user_agent,
            replaced_by: None,
            revoked_at: None,
            expires_at: now + Duration::days(IDLE_TIMEOUT_DAYS),
            absolute_expires_at: now + Duration::days(ABSOLUTE_LIFETIME_DAYS),
            last_used_at: now,
            created_at: now,
        };

        let refresh_token = self.repo.create(refresh_token).await?;

        let scopes: Vec<String> = DEFAULT_SCOPES.iter().map(|s| (*s).to_string()).collect();
        let (access_token, expires_at) =
            jwt::sign_access_token(user_id, client_type, &scopes, &self.jwt_secret)?;

        Ok(TokenPair {
            access_token,
            expires_at,
            raw_refresh_token: raw_refresh,
            refresh_token,
        })
    }

    pub async fn rotate(
        &self,
        raw_refresh_token: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<RefreshResult, AuthError> {
        let token_hash = hash_token(raw_refresh_token);
        let token = self
            .repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(AuthError::TokenInvalid)?;

        if token.revoked_at.is_some() {
            return Err(AuthError::TokenRevoked);
        }

        if let Some(replaced_by_id) = token.replaced_by {
            // Possible replay — check grace window
            let replacement = self.repo.find_by_id(replaced_by_id).await?;

            if let Some(ref replacement) = replacement {
                let elapsed = Utc::now() - replacement.created_at;
                if elapsed.num_seconds() < GRACE_WINDOW_SECS {
                    let scopes: Vec<String> =
                        DEFAULT_SCOPES.iter().map(|s| (*s).to_string()).collect();
                    let (access_token, expires_at) = jwt::sign_access_token(
                        token.user_id,
                        token.client_type,
                        &scopes,
                        &self.jwt_secret,
                    )?;

                    return Ok(RefreshResult {
                        access_token,
                        expires_at,
                        raw_refresh_token: String::new(),
                    });
                }
            }

            // Replay detected outside grace window — revoke the entire family
            self.repo.revoke_family(token.family_id).await?;
            return Err(AuthError::TokenRevoked);
        }

        let now = Utc::now();

        if token.expires_at < now || token.absolute_expires_at < now {
            return Err(AuthError::TokenExpired);
        }

        // Create new refresh token in the same family
        let raw_new_refresh = generate_refresh_token();
        let new_token_hash = hash_token(&raw_new_refresh);

        let new_refresh = RefreshToken {
            id: RefreshTokenId::new(),
            family_id: token.family_id,
            user_id: token.user_id,
            token_hash: new_token_hash,
            client_type: token.client_type,
            ip_address: ip,
            user_agent,
            replaced_by: None,
            revoked_at: None,
            expires_at: now + Duration::days(IDLE_TIMEOUT_DAYS),
            absolute_expires_at: token.absolute_expires_at,
            last_used_at: now,
            created_at: now,
        };

        let new_refresh = self.repo.create(new_refresh).await?;

        // Mark old token as replaced
        self.repo.set_replaced_by(token.id, new_refresh.id).await?;

        let scopes: Vec<String> = DEFAULT_SCOPES.iter().map(|s| (*s).to_string()).collect();
        let (access_token, expires_at) =
            jwt::sign_access_token(token.user_id, token.client_type, &scopes, &self.jwt_secret)?;

        Ok(RefreshResult {
            access_token,
            expires_at,
            raw_refresh_token: raw_new_refresh,
        })
    }

    pub async fn revoke_family_by_token(&self, raw_refresh_token: &str) -> Result<(), AuthError> {
        let token_hash = hash_token(raw_refresh_token);
        let token = self
            .repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(AuthError::TokenInvalid)?;

        self.repo.revoke_family(token.family_id).await?;
        Ok(())
    }

    pub async fn revoke_all_for_user(&self, user_id: UserId) -> Result<u64, AuthError> {
        Ok(self.repo.revoke_all_for_user(user_id).await?)
    }

    pub async fn list_active_families(
        &self,
        user_id: UserId,
    ) -> Result<Vec<RefreshToken>, AuthError> {
        Ok(self.repo.list_active_families(user_id).await?)
    }
}
