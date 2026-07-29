use std::any::Any;

use crate::error::AppError;
use ind_domain::{PasswordResetToken, UserId};

#[async_trait::async_trait]
pub trait PasswordResetTokenRepository: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;

    async fn create(&self, token: PasswordResetToken) -> Result<PasswordResetToken, AppError>;
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<PasswordResetToken>, AppError>;
    async fn mark_used(&self, id: uuid::Uuid) -> Result<(), AppError>;
    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, AppError>;
    async fn delete_expired(&self) -> Result<u64, AppError>;
}
