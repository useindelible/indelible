use super::*;

pub trait SettingsOperations: Send + Sync {
    fn get_preferences(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<PreferencesSection, AppError>>;

    fn update_preferences(
        &self,
        user_id: UserId,
        theme: Theme,
        settings: PreferencesSettings,
    ) -> BoxFuture<'_, Result<PreferencesSection, AppError>>;

    fn get_notifications(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<NotificationPreferences, AppError>>;

    fn update_notifications(
        &self,
        user_id: UserId,
        settings: NotificationPreferences,
    ) -> BoxFuture<'_, Result<NotificationPreferences, AppError>>;

    fn get_archival(&self, user_id: UserId) -> BoxFuture<'_, Result<ArchivalSettings, AppError>>;

    fn update_archival(
        &self,
        user_id: UserId,
        settings: ArchivalSettings,
    ) -> BoxFuture<'_, Result<ArchivalSettings, AppError>>;
}

pub trait HomeOperations: Send + Sync {
    fn get_dashboard<'a>(
        &'a self,
        user_id: UserId,
        widgets: Option<Vec<HomeWidgetKind>>,
    ) -> BoxFuture<'a, Result<HomeDashboardData, AppError>>;

    fn get_widget_config<'a>(
        &'a self,
        user_id: UserId,
    ) -> BoxFuture<'a, Result<Option<serde_json::Value>, AppError>>;

    fn set_widget_config<'a>(
        &'a self,
        user_id: UserId,
        config: serde_json::Value,
    ) -> BoxFuture<'a, Result<(), AppError>>;
}
