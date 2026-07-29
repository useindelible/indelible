use std::any::Any;

use crate::error::AppError;
use ind_domain::{RefreshToken, RefreshTokenId, UserId};

#[async_trait::async_trait]
pub trait RefreshTokenRepository: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;

    async fn create(&self, token: RefreshToken) -> Result<RefreshToken, AppError>;
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>, AppError>;
    async fn find_by_id(&self, id: RefreshTokenId) -> Result<Option<RefreshToken>, AppError>;
    async fn set_replaced_by(
        &self,
        id: RefreshTokenId,
        replaced_by: RefreshTokenId,
    ) -> Result<(), AppError>;
    async fn revoke_family(&self, family_id: uuid::Uuid) -> Result<u64, AppError>;
    async fn revoke_all_for_user(&self, user_id: UserId) -> Result<u64, AppError>;
    async fn list_active_families(&self, user_id: UserId) -> Result<Vec<RefreshToken>, AppError>;
    async fn update_last_used(&self, id: RefreshTokenId) -> Result<(), AppError>;
    async fn delete_expired(&self) -> Result<u64, AppError>;
}
