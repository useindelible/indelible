use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: UserId,
    pub daily_review_reminder_enabled: bool,
    pub daily_review_reminder_time: String,
    pub weekly_digest_enabled: bool,
    pub new_highlights_sync: bool,
    pub feed_updates: bool,
    pub marketing_emails: bool,
    pub updated_at: DateTime<Utc>,
}
