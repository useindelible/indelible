use std::sync::Arc;

use chrono::Utc;
use tracing::info;

use super::apple::AppleOAuth;
use super::error::OAuthError;
use super::google::GoogleOAuth;
use super::oidc::OidcOAuth;
pub use ind_application::ports::{OAuthCallbackContext, OAuthCallbackResult};

use super::{OAuthAuthorizationUrl, OAuthConfig, OAuthProviderConfigRef, OAuthUserInfo};
use ind_application::repos::oauth_identity::OAuthIdentityRepository;
use ind_application::repos::user::UserRepository;
use ind_domain::{OAuthIdentity, OAuthIdentityId, OAuthProvider, User, UserId, UserStatus};

pub struct OAuthService {
    config: OAuthConfig,
    user_repo: Arc<dyn UserRepository>,
    oauth_repo: Arc<dyn OAuthIdentityRepository>,
    allow_signups: bool,
}

impl OAuthService {
    pub fn new(
        config: OAuthConfig,
        user_repo: Arc<dyn UserRepository>,
        oauth_repo: Arc<dyn OAuthIdentityRepository>,
        allow_signups: bool,
    ) -> Self {
        Self {
            config,
            user_repo,
            oauth_repo,
            allow_signups,
        }
    }

    pub async fn oauth_start(
        &self,
        provider: OAuthProvider,
    ) -> Result<OAuthAuthorizationUrl, OAuthError> {
        match self.config.provider(provider) {
            Some(OAuthProviderConfigRef::Google(config)) => {
                let google = GoogleOAuth::new(config)?;
                Ok(google.authorization_url())
            }
            Some(OAuthProviderConfigRef::Apple(config)) => {
                let apple = AppleOAuth::new(config)?;
                Ok(apple.authorization_url())
            }
            Some(OAuthProviderConfigRef::Oidc(config)) => {
                let oidc = OidcOAuth::new(config)?;
                oidc.authorization_url().await
            }
            None => Err(OAuthError::ProviderNotConfigured(provider)),
        }
    }

    pub async fn oauth_callback(
        &self,
        provider: OAuthProvider,
        code: &str,
        state: &str,
        context: OAuthCallbackContext,
    ) -> Result<OAuthCallbackResult, OAuthError> {
        if state != context.expected_state {
            return Err(OAuthError::InvalidState);
        }

        let user_info = match self.config.provider(provider) {
            Some(OAuthProviderConfigRef::Google(config)) => {
                let google = GoogleOAuth::new(config)?;
                google.exchange_code(code).await?
            }
            Some(OAuthProviderConfigRef::Apple(config)) => {
                let apple = AppleOAuth::new(config)?;
                apple.exchange_code(code).await?
            }
            Some(OAuthProviderConfigRef::Oidc(config)) => {
                let flow = context.oidc_flow.as_ref().ok_or_else(|| {
                    OAuthError::Exchange("missing OIDC flow verifier".to_string())
                })?;
                let oidc = OidcOAuth::new(config)?;
                oidc.exchange_code(code, flow).await?
            }
            None => return Err(OAuthError::ProviderNotConfigured(provider)),
        };

        let (user, is_new_user) = self.find_or_create_user(user_info).await?;

        Ok(OAuthCallbackResult { user, is_new_user })
    }

    async fn find_or_create_user(
        &self,
        user_info: OAuthUserInfo,
    ) -> Result<(User, bool), OAuthError> {
        if let Some(existing) = self
            .oauth_repo
            .find_by_provider_user_id(user_info.provider, &user_info.provider_user_id)
            .await
            .map_err(OAuthError::from)?
        {
            reject_if_email_unverified(&user_info)?;

            let user = self
                .user_repo
                .find_by_id(existing.user_id)
                .await
                .map_err(OAuthError::from)?
                .ok_or(OAuthError::IdentityNotFound)?;

            reject_if_not_active(&user)?;

            info!(
                user_id = %user.id,
                provider = ?user_info.provider,
                "existing OAuth identity matched"
            );

            return Ok((user, false));
        }

        reject_if_email_unverified(&user_info)?;

        if let Some(ref email) = user_info.email {
            let normalized = User::normalize_email(email);
            if let Some(existing_user) = self
                .user_repo
                .find_by_email(&normalized)
                .await
                .map_err(OAuthError::from)?
            {
                reject_if_not_active(&existing_user)?;

                let identity = OAuthIdentity {
                    id: OAuthIdentityId::new(),
                    user_id: existing_user.id,
                    provider: user_info.provider,
                    provider_user_id: user_info.provider_user_id,
                    provider_email: Some(normalized),
                    access_token_enc: None,
                    refresh_token_enc: None,
                    created_at: Utc::now(),
                };

                self.oauth_repo
                    .create(identity)
                    .await
                    .map_err(OAuthError::from)?;

                info!(
                    user_id = %existing_user.id,
                    provider = ?user_info.provider,
                    "linked OAuth identity to existing user by email"
                );

                return Ok((existing_user, false));
            }
        }

        if user_info.provider == OAuthProvider::Oidc && user_info.email.is_none() {
            return Err(OAuthError::Exchange(
                "OIDC provider did not return an email claim".to_string(),
            ));
        }

        if !user_info.allow_auto_create {
            return Err(OAuthError::Exchange(
                "new SSO users are disabled for this instance".to_string(),
            ));
        }

        let user_id = UserId::new();
        let email = user_info
            .email
            .as_deref()
            .map(User::normalize_email)
            .unwrap_or_else(|| {
                format!(
                    "{}+{}@oauth.indelible.app",
                    user_info.provider_user_id,
                    match user_info.provider {
                        OAuthProvider::Google => "google",
                        OAuthProvider::Apple => "apple",
                        OAuthProvider::Oidc => "oidc",
                    }
                )
            });

        let email_token = crate::crypto::generate_email_token();

        let new_user = User {
            id: user_id,
            email,
            password_hash: None,
            display_name: user_info.display_name.unwrap_or_else(|| "New User".into()),
            avatar_url: user_info.avatar_url,
            locale: None,
            timezone: "UTC".into(),
            theme: ind_domain::Theme::default(),
            email_verified: user_info
                .email_verified
                .unwrap_or_else(|| user_info.email.is_some()),
            onboarding_completed: false,
            onboarding_step: 0,
            email_token,
            status: ind_domain::UserStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let identity = OAuthIdentity {
            id: OAuthIdentityId::new(),
            user_id,
            provider: user_info.provider,
            provider_user_id: user_info.provider_user_id,
            provider_email: user_info.email.as_deref().map(User::normalize_email),
            access_token_enc: None,
            refresh_token_enc: None,
            created_at: Utc::now(),
        };

        let created_user = if self.allow_signups {
            let created_user = self
                .user_repo
                .create(new_user)
                .await
                .map_err(OAuthError::from)?;
            self.oauth_repo
                .create(identity)
                .await
                .map_err(OAuthError::from)?;
            created_user
        } else {
            self.user_repo
                .create_first_user_with_oauth_identity(new_user, identity)
                .await
                .map_err(OAuthError::from)?
                .ok_or_else(|| {
                    OAuthError::Exchange("new SSO users are disabled for this instance".to_string())
                })?
        };

        info!(
            user_id = %created_user.id,
            provider = ?user_info.provider,
            "created new user via OAuth"
        );

        Ok((created_user, true))
    }

    pub async fn link_oauth(&self, user_id: UserId, info: OAuthUserInfo) -> Result<(), OAuthError> {
        let existing = self
            .oauth_repo
            .find_by_provider_user_id(info.provider, &info.provider_user_id)
            .await?;

        if let Some(identity) = existing {
            if identity.user_id != user_id {
                return Err(OAuthError::IdentityAlreadyLinked);
            }
        } else {
            let already_linked = self
                .oauth_repo
                .find_by_user_and_provider(user_id, info.provider)
                .await?;

            if already_linked.is_some() {
                return Err(OAuthError::IdentityAlreadyLinked);
            }

            let identity = OAuthIdentity {
                id: OAuthIdentityId::new(),
                user_id,
                provider: info.provider,
                provider_user_id: info.provider_user_id,
                provider_email: info.email.as_deref().map(User::normalize_email),
                access_token_enc: None,
                refresh_token_enc: None,
                created_at: Utc::now(),
            };
            self.oauth_repo.create(identity).await?;
        }

        info!(user_id = %user_id, provider = ?info.provider, "OAuth provider linked");

        Ok(())
    }

    pub async fn unlink_oauth(
        &self,
        user_id: UserId,
        oauth_identity_id: OAuthIdentityId,
    ) -> Result<(), OAuthError> {
        let identity = self
            .oauth_repo
            .find_by_id(oauth_identity_id)
            .await
            .map_err(OAuthError::from)?
            .ok_or(OAuthError::IdentityNotFound)?;

        if identity.user_id != user_id {
            return Err(OAuthError::IdentityNotFound);
        }

        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(OAuthError::from)?
            .ok_or(OAuthError::IdentityNotFound)?;

        let oauth_count = self
            .oauth_repo
            .count_by_user_id(user_id)
            .await
            .map_err(OAuthError::from)?;

        let has_password = user.password_hash.is_some();

        if !has_password && oauth_count <= 1 {
            return Err(OAuthError::CannotUnlinkOnly);
        }

        self.oauth_repo
            .delete(oauth_identity_id)
            .await
            .map_err(OAuthError::from)?;

        info!(
            user_id = %user_id,
            oauth_identity_id = %oauth_identity_id,
            "unlinked OAuth identity, sessions invalidated"
        );

        Ok(())
    }
}

fn reject_if_not_active(user: &User) -> Result<(), OAuthError> {
    match user.status {
        UserStatus::Active => Ok(()),
        UserStatus::Deactivated | UserStatus::Deleted => Err(OAuthError::UserDeactivated),
    }
}

fn reject_if_email_unverified(user_info: &OAuthUserInfo) -> Result<(), OAuthError> {
    if user_info.email_verified == Some(false) {
        return Err(OAuthError::Exchange(
            "OAuth provider reported an unverified email".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests;
