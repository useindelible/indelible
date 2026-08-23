use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::UserId;
use crate::string_enum::impl_string_enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Deactivated,
    Deleted,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
    #[default]
    System,
}

impl_string_enum!(Theme, "theme", {
    Light => "light",
    Dark => "dark",
    System => "system",
});

pub const PASSWORD_MIN_LENGTH: usize = 8;
pub const PASSWORD_MAX_LENGTH: usize = 2048;
pub const DISPLAY_NAME_MAX_LENGTH: usize = 100;

pub fn validate_password(value: &str) -> Result<(), crate::DomainError> {
    if value.len() < PASSWORD_MIN_LENGTH {
        return Err(crate::DomainError::Validation {
            field: "password".into(),
            message: format!("must be at least {PASSWORD_MIN_LENGTH} characters"),
        });
    }

    if value.len() > PASSWORD_MAX_LENGTH {
        return Err(crate::DomainError::Validation {
            field: "password".into(),
            message: format!("must be at most {PASSWORD_MAX_LENGTH} characters"),
        });
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvatarContentType {
    pub mime: &'static str,
    pub extension: &'static str,
}

impl AvatarContentType {
    pub const VALUES: &'static [AvatarContentType] = &[
        AvatarContentType {
            mime: "image/jpeg",
            extension: "jpg",
        },
        AvatarContentType {
            mime: "image/png",
            extension: "png",
        },
        AvatarContentType {
            mime: "image/webp",
            extension: "webp",
        },
    ];

    pub fn from_mime(mime: &str) -> Option<Self> {
        Self::VALUES
            .iter()
            .copied()
            .find(|content_type| content_type.mime == mime)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub password_hash: Option<String>,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub locale: Option<String>,
    pub timezone: String,
    pub theme: Theme,
    pub email_verified: bool,
    pub onboarding_completed: bool,
    pub onboarding_step: i16,
    pub email_token: String,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn normalize_email(raw: &str) -> String {
        raw.trim().to_lowercase()
    }
}
