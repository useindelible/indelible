use chrono::{Duration, Utc};
use ind_domain::{EmailVerificationToken, User, UserId};

use crate::crypto::{generate_verification_token, hash_token, verify_password};
use crate::error::AuthError;
use crate::service::AuthService;

impl AuthService {
    pub async fn verify_email(&self, raw_token: &str) -> Result<User, AuthError> {
        let token_hash = hash_token(raw_token);

        let token = self
            .email_verification_repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(AuthError::TokenInvalid)?;

        if token.expires_at <= Utc::now() {
            let _ = self.email_verification_repo.delete(token.id).await;
            return Err(AuthError::TokenExpired);
        }

        if self.user_repo.find_by_id(token.user_id).await?.is_none() {
            return Err(AuthError::AccountNotFound);
        }

        let user = self
            .user_repo
            .update_email_verified(token.user_id, true)
            .await?;
        self.email_verification_repo
            .delete_all_for_user(token.user_id)
            .await?;

        Ok(user)
    }

    pub async fn resend_verification(&self, user_id: &UserId) -> Result<Option<String>, AuthError> {
        let user = self
            .user_repo
            .find_by_id(*user_id)
            .await?
            .ok_or(AuthError::AccountNotFound)?;

        if user.email_verified {
            return Ok(None);
        }

        if let Some(latest) = self
            .email_verification_repo
            .find_latest_for_user(*user_id)
            .await?
        {
            let cooldown = Duration::seconds(60);
            if latest.created_at + cooldown > Utc::now() {
                return Err(AuthError::RateLimited);
            }
        }

        self.email_verification_repo
            .delete_all_for_user(*user_id)
            .await?;

        let raw_token = generate_verification_token();
        let now = Utc::now();
        let token = EmailVerificationToken {
            id: uuid::Uuid::now_v7(),
            user_id: *user_id,
            token_hash: hash_token(&raw_token),
            expires_at: now + Duration::hours(24),
            created_at: now,
        };

        self.email_verification_repo.create(token).await?;

        Ok(Some(raw_token))
    }

    pub async fn change_email(
        &self,
        user_id: &UserId,
        new_email: &str,
        current_password: &str,
    ) -> Result<String, AuthError> {
        let normalized = User::normalize_email(new_email);
        let user = self
            .user_repo
            .find_by_id(*user_id)
            .await?
            .ok_or(AuthError::AccountNotFound)?;

        let existing_hash = user
            .password_hash
            .clone()
            .ok_or(AuthError::InvalidCredentials)?;
        let password = current_password.to_string();
        let valid = tokio::task::spawn_blocking(move || verify_password(&password, &existing_hash))
            .await
            .map_err(|_| AuthError::HashError("verification task failed".into()))??;
        if !valid {
            return Err(AuthError::InvalidCredentials);
        }

        if let Some(existing_user) = self.user_repo.find_by_email(&normalized).await?
            && existing_user.id != *user_id
        {
            return Err(AuthError::EmailAlreadyExists);
        }

        self.user_repo
            .update_email_and_verification(*user_id, normalized, false)
            .await?;

        self.email_verification_repo
            .delete_all_for_user(*user_id)
            .await?;

        let raw_token = generate_verification_token();
        let now = Utc::now();
        let token = EmailVerificationToken {
            id: uuid::Uuid::now_v7(),
            user_id: *user_id,
            token_hash: hash_token(&raw_token),
            expires_at: now + Duration::hours(24),
            created_at: now,
        };

        self.email_verification_repo.create(token).await?;
        self.refresh_token_repo
            .revoke_all_for_user(*user_id)
            .await?;

        Ok(raw_token)
    }
}
