use chrono::{DateTime, Utc};

use crate::error::AppError;
use ind_domain::{IntegrationOAuthProvider, IntegrationOAuthToken, UserId};

#[async_trait::async_trait]
pub trait IntegrationOAuthTokenRepository: Send + Sync {
    async fn upsert(
        &self,
        user_id: UserId,
        provider: IntegrationOAuthProvider,
        access_token_enc: Vec<u8>,
        refresh_token_enc: Option<Vec<u8>>,
        token_expires_at: Option<DateTime<Utc>>,
        extra: serde_json::Value,
    ) -> Result<IntegrationOAuthToken, AppError>;

    async fn find_by_user_provider(
        &self,
        user_id: UserId,
        provider: IntegrationOAuthProvider,
    ) -> Result<Option<IntegrationOAuthToken>, AppError>;

    async fn set_tokens(
        &self,
        id: uuid::Uuid,
        access_token_enc: Vec<u8>,
        refresh_token_enc: Option<Vec<u8>>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), AppError>;

    async fn delete_by_user_provider(
        &self,
        user_id: UserId,
        provider: IntegrationOAuthProvider,
    ) -> Result<u64, AppError>;
}
