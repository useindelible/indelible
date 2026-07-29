use std::sync::Arc;

use ind_application::repos::email_verification::EmailVerificationTokenRepository;
use ind_application::repos::password_reset::PasswordResetTokenRepository;
use ind_application::repos::refresh_token::RefreshTokenRepository;
use ind_application::repos::user::UserRepository;

pub struct AuthService {
    pub(crate) user_repo: Arc<dyn UserRepository>,
    pub(crate) refresh_token_repo: Arc<dyn RefreshTokenRepository>,
    pub(crate) email_verification_repo: Arc<dyn EmailVerificationTokenRepository>,
    pub(crate) password_reset_repo: Arc<dyn PasswordResetTokenRepository>,
    pub(crate) jwt_secret: Vec<u8>,
    pub(crate) allow_signups: bool,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        refresh_token_repo: Arc<dyn RefreshTokenRepository>,
        email_verification_repo: Arc<dyn EmailVerificationTokenRepository>,
        password_reset_repo: Arc<dyn PasswordResetTokenRepository>,
        jwt_secret: Vec<u8>,
        allow_signups: bool,
    ) -> Self {
        Self {
            user_repo,
            refresh_token_repo,
            email_verification_repo,
            password_reset_repo,
            jwt_secret,
            allow_signups,
        }
    }

    pub async fn has_any_users(&self) -> Result<bool, ind_application::AppError> {
        self.user_repo.has_any_users().await
    }
}
