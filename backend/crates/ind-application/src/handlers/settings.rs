use std::sync::Arc;

use chrono::Utc;
use futures::future::BoxFuture;
use ind_domain::{ArchivalSettings, NotificationPreferences, PreferencesSettings, Theme, UserId};

use crate::AppError;
use crate::ports::content::SettingsOperations;
use crate::repos::notification_preferences::NotificationPreferencesRepository;
use crate::repos::user::UserRepository;
use crate::repos::user_preferences::UserPreferencesRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferencesSection {
    pub theme: Theme,
    pub settings: PreferencesSettings,
}

pub struct SettingsService {
    user_repo: Arc<dyn UserRepository>,
    user_preferences_repo: Arc<dyn UserPreferencesRepository>,
    notification_preferences_repo: Arc<dyn NotificationPreferencesRepository>,
}

impl SettingsService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        user_preferences_repo: Arc<dyn UserPreferencesRepository>,
        notification_preferences_repo: Arc<dyn NotificationPreferencesRepository>,
    ) -> Self {
        Self {
            user_repo,
            user_preferences_repo,
            notification_preferences_repo,
        }
    }

    pub async fn get_preferences(&self, user_id: UserId) -> Result<PreferencesSection, AppError> {
        let user = self.user_repo.find_by_id(user_id).await?.ok_or_else(|| {
            AppError::Domain(ind_domain::DomainError::NotFound {
                entity: "user",
                id: user_id.to_string(),
            })
        })?;

        let settings = self
            .user_preferences_repo
            .get_preferences(user_id)
            .await?
            .unwrap_or_default();

        Ok(PreferencesSection {
            theme: user.theme,
            settings,
        })
    }

    pub async fn update_preferences(
        &self,
        user_id: UserId,
        section: PreferencesSection,
    ) -> Result<PreferencesSection, AppError> {
        let user = self.user_repo.find_by_id(user_id).await?.ok_or_else(|| {
            AppError::Domain(ind_domain::DomainError::NotFound {
                entity: "user",
                id: user_id.to_string(),
            })
        })?;

        self.user_repo
            .update_profile_fields(
                user_id,
                user.display_name,
                user.avatar_url,
                user.locale,
                user.timezone,
                section.theme,
            )
            .await?;

        let settings = self
            .user_preferences_repo
            .upsert_preferences(user_id, &normalize_preferences(section.settings))
            .await?;

        Ok(PreferencesSection {
            theme: section.theme,
            settings,
        })
    }

    pub async fn get_notifications(
        &self,
        user_id: UserId,
    ) -> Result<NotificationPreferences, AppError> {
        Ok(self
            .notification_preferences_repo
            .get_by_user(user_id)
            .await?
            .unwrap_or_else(|| default_notification_preferences(user_id)))
    }

    pub async fn update_notifications(
        &self,
        user_id: UserId,
        preferences: NotificationPreferences,
    ) -> Result<NotificationPreferences, AppError> {
        self.notification_preferences_repo
            .upsert(&normalize_notification_preferences(user_id, preferences))
            .await
    }

    pub async fn get_archival(&self, user_id: UserId) -> Result<ArchivalSettings, AppError> {
        let settings = self
            .user_preferences_repo
            .get_archival(user_id)
            .await?
            .unwrap_or_default();

        Ok(settings)
    }

    pub async fn update_archival(
        &self,
        user_id: UserId,
        settings: ArchivalSettings,
    ) -> Result<ArchivalSettings, AppError> {
        let normalized = normalize_archival(settings);
        self.user_preferences_repo
            .upsert_archival(user_id, &normalized)
            .await
    }
}

impl SettingsOperations for SettingsService {
    fn get_preferences(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<PreferencesSection, AppError>> {
        Box::pin(self.get_preferences(user_id))
    }

    fn update_preferences(
        &self,
        user_id: UserId,
        theme: Theme,
        settings: PreferencesSettings,
    ) -> BoxFuture<'_, Result<PreferencesSection, AppError>> {
        Box::pin(self.update_preferences(user_id, PreferencesSection { theme, settings }))
    }

    fn get_notifications(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<NotificationPreferences, AppError>> {
        Box::pin(self.get_notifications(user_id))
    }

    fn update_notifications(
        &self,
        user_id: UserId,
        settings: NotificationPreferences,
    ) -> BoxFuture<'_, Result<NotificationPreferences, AppError>> {
        Box::pin(self.update_notifications(user_id, settings))
    }

    fn get_archival(&self, user_id: UserId) -> BoxFuture<'_, Result<ArchivalSettings, AppError>> {
        Box::pin(self.get_archival(user_id))
    }

    fn update_archival(
        &self,
        user_id: UserId,
        settings: ArchivalSettings,
    ) -> BoxFuture<'_, Result<ArchivalSettings, AppError>> {
        Box::pin(self.update_archival(user_id, settings))
    }
}

fn normalize_preferences(mut settings: PreferencesSettings) -> PreferencesSettings {
    settings.ai.custom_prompt = settings
        .ai
        .custom_prompt
        .and_then(|prompt| (!prompt.trim().is_empty()).then(|| prompt.trim().to_string()));
    settings
}

fn normalize_notification_preferences(
    user_id: UserId,
    mut preferences: NotificationPreferences,
) -> NotificationPreferences {
    preferences.user_id = user_id;
    if preferences.daily_review_reminder_time.trim().is_empty() {
        preferences.daily_review_reminder_time = "09:00".to_string();
    } else {
        preferences.daily_review_reminder_time =
            preferences.daily_review_reminder_time.trim().to_string();
    }
    preferences.updated_at = Utc::now();
    preferences
}

fn normalize_archival(mut settings: ArchivalSettings) -> ArchivalSettings {
    settings.archive_formats.readable_html = true;
    settings.proxy.url = settings
        .proxy
        .url
        .and_then(|url| (!url.trim().is_empty()).then(|| url.trim().to_string()));
    if settings.proxy.url.is_none() {
        settings.proxy.all_requests = false;
    }
    settings.processing.browser_timeout_secs = settings.processing.browser_timeout_secs.max(30);
    settings.processing.max_concurrent_archives =
        settings.processing.max_concurrent_archives.clamp(1, 10);
    settings
}

fn default_notification_preferences(user_id: UserId) -> NotificationPreferences {
    NotificationPreferences {
        user_id,
        daily_review_reminder_enabled: true,
        daily_review_reminder_time: "09:00".to_string(),
        weekly_digest_enabled: true,
        new_highlights_sync: true,
        feed_updates: true,
        marketing_emails: false,
        updated_at: Utc::now(),
    }
}
