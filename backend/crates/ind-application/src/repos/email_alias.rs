use crate::error::AppError;
use ind_domain::{EmailAlias, EmailAliasId, EmailDestination, UserId};

#[derive(Debug, Clone, Copy)]
pub struct CreateEmailAlias<'a> {
    pub user_id: UserId,
    pub destination: EmailDestination,
    pub local_part: &'a str,
    pub is_default: bool,
}

#[async_trait::async_trait]
pub trait EmailAliasRepository: Send + Sync {
    async fn create(&self, input: CreateEmailAlias<'_>) -> Result<EmailAlias, AppError>;

    async fn create_with_default_rotation(
        &self,
        input: CreateEmailAlias<'_>,
        retire_grace_days: i64,
    ) -> Result<EmailAlias, AppError>;

    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<EmailAlias>, AppError>;

    async fn find_active(
        &self,
        destination: EmailDestination,
        local_part: &str,
    ) -> Result<Option<EmailAlias>, AppError>;

    async fn find_by_id_and_user(
        &self,
        user_id: UserId,
        alias_id: EmailAliasId,
    ) -> Result<Option<EmailAlias>, AppError>;

    /// Active default alias for a user on a given destination, used to identify
    /// the predecessor that should enter its 28-day grace window when a new
    /// default is created.
    async fn find_active_default(
        &self,
        user_id: UserId,
        destination: EmailDestination,
    ) -> Result<Option<EmailAlias>, AppError>;

    async fn retire(&self, alias_id: EmailAliasId) -> Result<(), AppError>;

    async fn mark_for_retire(
        &self,
        alias_id: EmailAliasId,
        grace_days: i64,
    ) -> Result<(), AppError>;
}
