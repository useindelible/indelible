use std::any::Any;

use crate::error::AppError;
use ind_domain::{EmailVerificationToken, UserId};

#[async_trait::async_trait]
pub trait EmailVerificationTokenRepository: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;

    async fn create(
        &self,
        token: EmailVerificationToken,
    ) -> Result<EmailVerificationToken, AppError>;
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<EmailVerificationToken>, AppError>;
    async fn find_latest_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<EmailVerificationToken>, AppError>;
    async fn delete(&self, id: uuid::Uuid) -> Result<(), AppError>;
    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, AppError>;
    async fn delete_expired(&self) -> Result<u64, AppError>;
}
