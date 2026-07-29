use chrono::{Duration, Utc};
pub use ind_application::ports::{RegisterRequest, RegisterResponse};
use ind_domain::{
    ClientType, EmailVerificationToken, RefreshToken, RefreshTokenId, User, UserId, UserStatus,
    validate_password,
};

use crate::crypto::{
    generate_email_token, generate_refresh_token, generate_verification_token, hash_password,
    hash_token,
};
use crate::error::AuthError;
use crate::jwt;
use crate::service::AuthService;

const IDLE_TIMEOUT_DAYS: i64 = 30;
const ABSOLUTE_LIFETIME_DAYS: i64 = 90;

impl AuthService {
    pub async fn register(
        &self,
        req: RegisterRequest,
        client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<RegisterResponse, AuthError> {
        let email = User::normalize_email(&req.email);

        if validate_password(&req.password).is_err() {
            return Err(AuthError::PasswordTooWeak);
        }

        if !self.allow_signups && self.user_repo.has_any_users().await? {
            return Err(AuthError::SignupsDisabled);
        }

        if self.user_repo.find_by_email(&email).await?.is_some() {
            return Err(AuthError::EmailAlreadyExists);
        }

        let password_clone = req.password.clone();
        let password_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
            .await
            .map_err(|_| AuthError::HashError("hashing task failed".into()))??;
        let now = Utc::now();

        let email_token = generate_email_token();

        let user = User {
            id: UserId::new(),
            email,
            password_hash: Some(password_hash),
            display_name: req.display_name,
            avatar_url: None,
            locale: "en".to_string(),
            timezone: "UTC".to_string(),
            theme: Default::default(),
            email_verified: false,
            onboarding_completed: false,
            onboarding_step: 0,
            email_token,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
        };

        let user = if self.allow_signups {
            self.user_repo.create(user).await?
        } else {
            self.user_repo
                .create_first_user(user)
                .await?
                .ok_or(AuthError::SignupsDisabled)?
        };

        let raw_verification_token = generate_verification_token();
        let verification_token_hash = hash_token(&raw_verification_token);
        let verification_token = EmailVerificationToken {
            id: uuid::Uuid::now_v7(),
            user_id: user.id,
            token_hash: verification_token_hash,
            expires_at: now + Duration::hours(24),
            created_at: now,
        };
        self.email_verification_repo
            .create(verification_token)
            .await?;

        let family_id = uuid::Uuid::now_v7();
        let raw_refresh = generate_refresh_token();
        let token_hash = hash_token(&raw_refresh);

        let refresh_token = RefreshToken {
            id: RefreshTokenId::new(),
            family_id,
            user_id: user.id,
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

        let scopes = vec!["read".to_string(), "write".to_string()];
        let (access_token, expires_at) =
            jwt::sign_access_token(user.id, client_type, &scopes, &self.jwt_secret)?;

        Ok(RegisterResponse {
            user,
            access_token,
            expires_at,
            raw_refresh_token: raw_refresh,
            refresh_token,
            verification_token_sent: true,
        })
    }
}
