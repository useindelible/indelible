use crate::error::AppError;
use ind_domain::{ApiToken, ApiTokenId, UserId};

#[async_trait::async_trait]
pub trait ApiTokenRepository: Send + Sync {
    async fn create(&self, token: ApiToken) -> Result<ApiToken, AppError>;
    async fn find_by_id(
        &self,
        id: ApiTokenId,
        user_id: UserId,
    ) -> Result<Option<ApiToken>, AppError>;
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<ApiToken>, AppError>;
    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<ApiToken>, AppError>;
    async fn update_last_used(&self, id: ApiTokenId) -> Result<(), AppError>;
    async fn delete(&self, id: ApiTokenId, user_id: UserId) -> Result<(), AppError>;
}
