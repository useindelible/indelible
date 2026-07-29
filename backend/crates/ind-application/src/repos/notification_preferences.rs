use crate::error::AppError;
use ind_domain::{NotificationPreferences, UserId};

#[async_trait::async_trait]
pub trait NotificationPreferencesRepository: Send + Sync {
    async fn get_by_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<NotificationPreferences>, AppError>;
    async fn upsert(
        &self,
        preferences: &NotificationPreferences,
    ) -> Result<NotificationPreferences, AppError>;
}
