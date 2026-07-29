use chrono::{Duration, Utc};
use ind_domain::{PasswordResetToken, User};

use crate::crypto::{generate_password_reset_token, hash_password, hash_token};
use crate::error::AuthError;
use crate::service::AuthService;

impl AuthService {
    pub async fn forgot_password(&self, email: &str) -> Result<Option<String>, AuthError> {
        let normalized = User::normalize_email(email);

        let user = match self.user_repo.find_by_email(&normalized).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        self.password_reset_repo
            .delete_all_for_user(user.id)
            .await?;

        let raw_token = generate_password_reset_token();
        let now = Utc::now();
        let token = PasswordResetToken {
            id: uuid::Uuid::now_v7(),
            user_id: user.id,
            token_hash: hash_token(&raw_token),
            expires_at: now + Duration::hours(1),
            used_at: None,
            created_at: now,
        };

        self.password_reset_repo.create(token).await?;

        Ok(Some(raw_token))
    }

    pub async fn reset_password(
        &self,
        raw_token: &str,
        new_password: &str,
    ) -> Result<User, AuthError> {
        if new_password.len() < 8 || new_password.len() > 2048 {
            return Err(AuthError::PasswordTooWeak);
        }

        let token_hash = hash_token(raw_token);

        let token = self
            .password_reset_repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(AuthError::TokenInvalid)?;

        if token.used_at.is_some() {
            return Err(AuthError::TokenAlreadyUsed);
        }

        if token.expires_at <= Utc::now() {
            return Err(AuthError::TokenExpired);
        }

        if self.user_repo.find_by_id(token.user_id).await?.is_none() {
            return Err(AuthError::AccountNotFound);
        }

        let password_clone = new_password.to_string();
        let new_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
            .await
            .map_err(|_| AuthError::HashError("hashing task failed".into()))??;

        let user = self
            .user_repo
            .update_password_hash(token.user_id, new_hash)
            .await?;

        self.password_reset_repo.mark_used(token.id).await?;
        self.refresh_token_repo
            .revoke_all_for_user(token.user_id)
            .await?;

        Ok(user)
    }
}
