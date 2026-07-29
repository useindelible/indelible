use crate::error::AppError;
use ind_domain::{ArchivalSettings, PreferencesSettings, UserId};

#[async_trait::async_trait]
pub trait UserPreferencesRepository: Send + Sync {
    async fn get_preferences(
        &self,
        user_id: UserId,
    ) -> Result<Option<PreferencesSettings>, AppError>;
    async fn upsert_preferences(
        &self,
        user_id: UserId,
        settings: &PreferencesSettings,
    ) -> Result<PreferencesSettings, AppError>;
    async fn get_archival(&self, user_id: UserId) -> Result<Option<ArchivalSettings>, AppError>;
    async fn upsert_archival(
        &self,
        user_id: UserId,
        settings: &ArchivalSettings,
    ) -> Result<ArchivalSettings, AppError>;
}
