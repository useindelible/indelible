use std::any::Any;

use crate::error::AppError;
use ind_domain::{AuthorizationCode, AuthorizationCodeId};

#[async_trait::async_trait]
pub trait AuthorizationCodeRepository: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;

    async fn create(&self, code: AuthorizationCode) -> Result<AuthorizationCode, AppError>;
    async fn find_by_code_hash(
        &self,
        code_hash: &str,
    ) -> Result<Option<AuthorizationCode>, AppError>;
    async fn consume_by_code_hash(
        &self,
        code_hash: &str,
    ) -> Result<Option<AuthorizationCode>, AppError>;
    async fn mark_used(&self, id: AuthorizationCodeId) -> Result<(), AppError>;
    async fn delete_expired(&self) -> Result<u64, AppError>;
}
