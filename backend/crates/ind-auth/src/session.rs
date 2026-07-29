use chrono::{Duration, Utc};
pub use ind_application::ports::RefreshResult;
use ind_domain::{ClientType, RefreshToken, RefreshTokenId, UserId};

use crate::crypto::{generate_refresh_token, hash_token};
use crate::error::AuthError;
use crate::jwt;
use crate::service::AuthService;

const IDLE_TIMEOUT_DAYS: i64 = 30;
const ABSOLUTE_LIFETIME_DAYS: i64 = 90;
const GRACE_WINDOW_SECS: i64 = 10;

impl AuthService {
    pub async fn refresh(
        &self,
        raw_refresh_token: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<RefreshResult, AuthError> {
        let token_hash = hash_token(raw_refresh_token);
        let token = self
            .refresh_token_repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(AuthError::TokenInvalid)?;

        if token.revoked_at.is_some() {
            return Err(AuthError::TokenRevoked);
        }

        if let Some(replaced_by_id) = token.replaced_by {
            let replacement = self.refresh_token_repo.find_by_id(replaced_by_id).await?;

            if let Some(ref replacement) = replacement {
                let elapsed = Utc::now() - replacement.created_at;
                if elapsed.num_seconds() < GRACE_WINDOW_SECS {
                    // Within grace window: concurrent refresh race. Return a fresh
                    // access token. The refresh token is NOT rotated again — the
                    // winning concurrent request already set the new cookie/token.
                    let (access_token, expires_at) =
                        jwt::sign_access_token(token.user_id, token.client_type, &self.jwt_secret)?;

                    // Return the replacement token hash's raw value is not available,
                    // so we return an empty string to signal "no new refresh token".
                    // The HTTP handler should not set a new refresh cookie in this case.
                    return Ok(RefreshResult {
                        access_token,
                        expires_at,
                        raw_refresh_token: String::new(),
                    });
                }
            }

            self.refresh_token_repo
                .revoke_family(token.family_id)
                .await?;
            return Err(AuthError::TokenRevoked);
        }

        let now = Utc::now();

        if token.expires_at < now || token.absolute_expires_at < now {
            return Err(AuthError::TokenExpired);
        }

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

        let new_refresh = self.refresh_token_repo.create(new_refresh).await?;

        self.refresh_token_repo
            .set_replaced_by(token.id, new_refresh.id)
            .await?;

        let (access_token, expires_at) =
            jwt::sign_access_token(token.user_id, token.client_type, &self.jwt_secret)?;

        Ok(RefreshResult {
            access_token,
            expires_at,
            raw_refresh_token: raw_new_refresh,
        })
    }

    pub async fn logout_by_refresh_token(&self, raw_refresh_token: &str) -> Result<(), AuthError> {
        let token_hash = hash_token(raw_refresh_token);
        let token = self
            .refresh_token_repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(AuthError::TokenInvalid)?;

        self.refresh_token_repo
            .revoke_family(token.family_id)
            .await?;
        Ok(())
    }

    pub async fn logout_all(&self, user_id: UserId) -> Result<u64, AuthError> {
        let count = self.refresh_token_repo.revoke_all_for_user(user_id).await?;
        Ok(count)
    }

    pub async fn list_active_refresh_families(
        &self,
        user_id: UserId,
    ) -> Result<Vec<RefreshToken>, AuthError> {
        Ok(self
            .refresh_token_repo
            .list_active_families(user_id)
            .await?)
    }

    /// Creates a refresh token + JWT for a user authenticated via OAuth.
    pub async fn create_tokens_for_user(
        &self,
        user_id: UserId,
        client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(RefreshToken, String, String, i64), AuthError> {
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
        let refresh_token = self.refresh_token_repo.create(refresh_token).await?;

        let (access_token, expires_at) =
            jwt::sign_access_token(user_id, client_type, &self.jwt_secret)?;

        Ok((refresh_token, raw_refresh, access_token, expires_at))
    }
}
