use crate::error::AppError;
use ind_domain::{OAuthIdentity, OAuthIdentityId, OAuthProvider, UserId};

#[async_trait::async_trait]
pub trait OAuthIdentityRepository: Send + Sync {
    async fn create(&self, identity: OAuthIdentity) -> Result<OAuthIdentity, AppError>;
    async fn find_by_provider_user_id(
        &self,
        provider: OAuthProvider,
        provider_user_id: &str,
    ) -> Result<Option<OAuthIdentity>, AppError>;
    async fn find_by_id(&self, id: OAuthIdentityId) -> Result<Option<OAuthIdentity>, AppError>;
    async fn find_by_user_and_provider(
        &self,
        user_id: UserId,
        provider: OAuthProvider,
    ) -> Result<Option<OAuthIdentity>, AppError>;
    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<OAuthIdentity>, AppError>;
    async fn count_by_user_id(&self, user_id: UserId) -> Result<u64, AppError>;
    async fn update_tokens(
        &self,
        id: OAuthIdentityId,
        access_token: Option<Vec<u8>>,
        refresh_token: Option<Vec<u8>>,
    ) -> Result<(), AppError>;
    async fn delete(&self, id: OAuthIdentityId) -> Result<(), AppError>;
}
