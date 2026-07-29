use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{EmailAliasId, EmailDestination, UserId};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailAliasStatus {
    #[default]
    Active,
    Retired,
}

impl EmailAliasStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Retired => "retired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAlias {
    pub id: EmailAliasId,
    pub user_id: UserId,
    pub destination: EmailDestination,
    pub local_part: String,
    pub status: EmailAliasStatus,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub retire_at: Option<DateTime<Utc>>,
    pub retired_at: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AliasLocalPartError {
    #[error("local part must be 3-32 characters")]
    Length,
    #[error("local part may only contain a-z, 0-9, dot, underscore, hyphen")]
    Charset,
    #[error("local part may not start or end with a dot")]
    Format,
    #[error("local part is reserved")]
    Reserved,
}

const RESERVED_LOCAL_PARTS: &[&str] = &[
    "admin",
    "support",
    "noreply",
    "no-reply",
    "abuse",
    "postmaster",
    "hostmaster",
    "webmaster",
];

pub fn validate_local_part(s: &str) -> Result<String, AliasLocalPartError> {
    let lower = s.trim().to_lowercase();
    if lower.len() < 3 || lower.len() > 32 {
        return Err(AliasLocalPartError::Length);
    }
    if !lower
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err(AliasLocalPartError::Charset);
    }
    if lower.starts_with('.') || lower.ends_with('.') {
        return Err(AliasLocalPartError::Format);
    }
    if RESERVED_LOCAL_PARTS.contains(&lower.as_str()) {
        return Err(AliasLocalPartError::Reserved);
    }
    Ok(lower)
}
