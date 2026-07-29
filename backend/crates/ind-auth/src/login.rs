use chrono::{Duration, Utc};
pub use ind_application::ports::{LoginRequest, LoginResponse};
use ind_domain::{ClientType, RefreshToken, User, UserStatus};

use crate::crypto::{DUMMY_HASH, generate_refresh_token, hash_token, verify_password};
use crate::error::AuthError;
use crate::jwt;
use crate::service::AuthService;

const IDLE_TIMEOUT_DAYS: i64 = 30;
const ABSOLUTE_LIFETIME_DAYS: i64 = 90;

impl AuthService {
    pub async fn login(
        &self,
        req: LoginRequest,
        client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<LoginResponse, AuthError> {
        let email = User::normalize_email(&req.email);

        let maybe_user = self.user_repo.find_by_email(&email).await?;

        let user = match maybe_user {
            Some(user) => user,
            None => {
                let password_clone = req.password.clone();
                let dummy = DUMMY_HASH.clone();
                let _ =
                    tokio::task::spawn_blocking(move || verify_password(&password_clone, &dummy))
                        .await
                        .map_err(|_| AuthError::HashError("verification task failed".into()))?;
                return Err(AuthError::InvalidCredentials);
            }
        };

        match user.status {
            UserStatus::Deactivated | UserStatus::Deleted => {
                return Err(AuthError::AccountDisabled);
            }
            UserStatus::Active => {}
        }

        let password_hash = user
            .password_hash
            .as_deref()
            .ok_or(AuthError::InvalidCredentials)?;

        let password_clone = req.password.clone();
        let hash_clone = password_hash.to_string();
        let valid =
            tokio::task::spawn_blocking(move || verify_password(&password_clone, &hash_clone))
                .await
                .map_err(|_| AuthError::HashError("verification task failed".into()))??;
        if !valid {
            return Err(AuthError::InvalidCredentials);
        }

        let now = Utc::now();
        let family_id = uuid::Uuid::now_v7();
        let raw_refresh = generate_refresh_token();
        let token_hash = hash_token(&raw_refresh);

        let refresh_token = RefreshToken {
            id: ind_domain::RefreshTokenId::new(),
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

        let (access_token, expires_at) =
            jwt::sign_access_token(user.id, client_type, &self.jwt_secret)?;

        Ok(LoginResponse {
            user,
            access_token,
            expires_at,
            raw_refresh_token: raw_refresh,
            refresh_token,
        })
    }
}
