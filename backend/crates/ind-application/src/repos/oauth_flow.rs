use chrono::{DateTime, Utc};

use crate::error::AppError;

#[async_trait::async_trait]
pub trait OAuthFlowRepository: Send + Sync {
    async fn insert_strict(
        &self,
        state_hash: &str,
        provider: &str,
        flow_kind: &str,
        sealed_flow: Vec<u8>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn upsert(
        &self,
        state_hash: &str,
        provider: &str,
        flow_kind: &str,
        sealed_flow: Vec<u8>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn consume(&self, state_hash: &str) -> Result<Option<Vec<u8>>, AppError>;

    async fn consume_scoped(
        &self,
        state_hash: &str,
        provider: &str,
        flow_kind: &str,
    ) -> Result<Option<Vec<u8>>, AppError>;
}
