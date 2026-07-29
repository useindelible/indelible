pub use ind_application::ports::{
    ChangePasswordRequest, DeleteAccountRequest, UpdateProfileRequest, UserProfile,
};
use ind_domain::{User, UserId, UserStatus, validate_password};

use crate::crypto::{hash_password, verify_password};
use crate::error::AuthError;
use crate::service::AuthService;
use crate::validation;

fn build_profile(user: &User) -> UserProfile {
    UserProfile {
        id: user.id,
        email: user.email.clone(),
        display_name: user.display_name.clone(),
        avatar_url: user.avatar_url.clone(),
        locale: user.locale.clone(),
        timezone: user.timezone.clone(),
        theme: user.theme,
        email_verified: user.email_verified,
        onboarding_completed: user.onboarding_completed,
        has_password: user.password_hash.is_some(),
        email_token: user.email_token.clone(),
        created_at: user.created_at,
        updated_at: user.updated_at,
    }
}

fn validate_update_profile_request(req: &UpdateProfileRequest) -> Result<(), AuthError> {
    if let Some(display_name) = req.display_name.as_deref() {
        validation::optional_trimmed_non_blank(display_name)
            .map_err(|err| validation_error("display_name", err))?;
        validation::optional_trimmed_max_display_name_length(display_name)
            .map_err(|err| validation_error("display_name", err))?;
    }
    if let Some(Some(avatar_url)) = req.avatar_url.as_ref() {
        validation::optional_avatar_reference(avatar_url)
            .map_err(|err| validation_error("avatar_url", err))?;
    }
    if let Some(locale) = req.locale.as_deref() {
        validation::optional_locale(locale).map_err(|err| validation_error("locale", err))?;
    }
    if let Some(timezone) = req.timezone.as_deref() {
        validation::optional_timezone(timezone).map_err(|err| validation_error("timezone", err))?;
    }

    Ok(())
}

fn validation_error(field: &'static str, err: validator::ValidationError) -> AuthError {
    AuthError::ValidationError {
        field: field.to_string(),
        message: err
            .message
            .map(|message| message.to_string())
            .unwrap_or_else(|| "is invalid".to_string()),
    }
}

impl AuthService {
    pub async fn get_profile(&self, user_id: UserId) -> Result<UserProfile, AuthError> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::AccountNotFound)?;

        Ok(build_profile(&user))
    }

    pub async fn update_profile(
        &self,
        user_id: UserId,
        req: UpdateProfileRequest,
    ) -> Result<UserProfile, AuthError> {
        validate_update_profile_request(&req)?;

        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::AccountNotFound)?;

        let display_name = req
            .display_name
            .map(|n| n.trim().to_string())
            .unwrap_or(user.display_name);
        let avatar_url = req.avatar_url.unwrap_or(user.avatar_url);
        if let Some(ref avatar_ref) = avatar_url
            && validation::is_internal_avatar_key(avatar_ref)
            && !validation::avatar_key_belongs_to_user(&user_id, avatar_ref)
        {
            return Err(AuthError::ValidationError {
                field: "avatar_url".to_string(),
                message: "must reference an avatar owned by the current user".to_string(),
            });
        }
        let locale = req.locale.unwrap_or(user.locale);
        let timezone = req.timezone.unwrap_or(user.timezone);
        let theme = req.theme.unwrap_or(user.theme);

        let user = self
            .user_repo
            .update_profile_fields(user_id, display_name, avatar_url, locale, timezone, theme)
            .await?;

        Ok(build_profile(&user))
    }

    pub async fn change_password(
        &self,
        user_id: UserId,
        req: ChangePasswordRequest,
    ) -> Result<(), AuthError> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::AccountNotFound)?;

        if let Some(ref existing_hash) = user.password_hash {
            let pw_clone = req.current_password.clone();
            let hash_clone = existing_hash.clone();
            let valid =
                tokio::task::spawn_blocking(move || verify_password(&pw_clone, &hash_clone))
                    .await
                    .map_err(|_| AuthError::HashError("verification task failed".into()))??;
            if !valid {
                return Err(AuthError::InvalidCredentials);
            }
        }

        if validate_password(&req.new_password).is_err() {
            return Err(AuthError::PasswordTooWeak);
        }

        let password_clone = req.new_password.clone();
        let new_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
            .await
            .map_err(|_| AuthError::HashError("hashing task failed".into()))??;

        self.user_repo
            .update_password_hash(user_id, new_hash)
            .await?;
        // Revoke all refresh tokens (caller's current JWT remains valid until expiry)
        self.refresh_token_repo.revoke_all_for_user(user_id).await?;

        Ok(())
    }

    pub async fn delete_account(
        &self,
        user_id: UserId,
        req: DeleteAccountRequest,
    ) -> Result<(), AuthError> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::AccountNotFound)?;

        if req.confirmation != user.email {
            return Err(AuthError::ConfirmationRequired);
        }

        if user.status == UserStatus::Deleted {
            return Err(AuthError::AccountDisabled);
        }

        // Refresh tokens ride the purge transaction's cascade: sessions die
        // exactly when the account does. Revoking separately beforehand would
        // leave a failed purge with a live account and no sessions.
        let outcome = self.account_purge_repo.purge_account(user_id).await?;
        tracing::info!(
            user_id = %user_id,
            documents = outcome.documents_deleted,
            "account permanently purged"
        );

        Ok(())
    }
}
