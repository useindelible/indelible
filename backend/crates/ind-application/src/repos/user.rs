use std::any::Any;

use crate::error::AppError;
use ind_domain::{OAuthIdentity, Theme, User, UserId};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn find_by_email_token(&self, token: &str) -> Result<Option<User>, AppError>;
    async fn create(&self, user: User) -> Result<User, AppError>;

    async fn has_any_users(&self) -> Result<bool, AppError>;

    /// Insert `user` iff the users table is empty, serialized by a transaction-scoped
    /// advisory lock so concurrent first-run signups cannot both succeed. Returns
    /// `Ok(None)` when a user already exists.
    async fn create_first_user(&self, user: User) -> Result<Option<User>, AppError>;

    /// Insert `user` and its first OAuth identity iff there are no non-deleted
    /// users, in one repository-managed transaction. Returns `Ok(None)` when a user
    /// already exists.
    async fn create_first_user_with_oauth_identity(
        &self,
        user: User,
        identity: OAuthIdentity,
    ) -> Result<Option<User>, AppError>;

    async fn update_profile_fields(
        &self,
        id: UserId,
        display_name: String,
        avatar_url: Option<String>,
        locale: String,
        timezone: String,
        theme: Theme,
    ) -> Result<User, AppError>;

    async fn update_onboarding(
        &self,
        id: UserId,
        onboarding_step: i16,
        onboarding_completed: bool,
    ) -> Result<User, AppError>;

    async fn update_password_hash(
        &self,
        id: UserId,
        password_hash: String,
    ) -> Result<User, AppError>;

    async fn update_email_verified(
        &self,
        id: UserId,
        email_verified: bool,
    ) -> Result<User, AppError>;

    async fn update_email_and_verification(
        &self,
        id: UserId,
        email: String,
        email_verified: bool,
    ) -> Result<User, AppError>;

    async fn soft_delete(&self, id: UserId) -> Result<(), AppError>;
}
