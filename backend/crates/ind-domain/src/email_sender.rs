use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{EmailSenderId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CanonicalAddress(String);

impl CanonicalAddress {
    pub fn new(raw: &str) -> Self {
        let trimmed = raw.trim().to_lowercase();
        let value = match trimmed.rsplit_once('@') {
            Some((local, domain)) => {
                let base_local = local.split('+').next().unwrap_or(local);
                format!("{base_local}@{domain}")
            }
            None => trimmed,
        };
        Self(value)
    }

    pub fn from_canonical_unchecked(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn domain(&self) -> Option<&str> {
        self.0.rsplit_once('@').map(|(_, domain)| domain)
    }
}

/// Splits a verbatim `From:` header value into a canonical address and an optional
/// display name. Accepts both `"Display Name <addr@host>"` and bare `"addr@host"`.
pub fn parse_from_header(raw: &str) -> (CanonicalAddress, Option<String>) {
    if let Some(start) = raw.rfind('<')
        && let Some(end) = raw[start + 1..].find('>')
    {
        let address = raw[start + 1..start + 1 + end].trim();
        let display = raw[..start].trim().trim_matches('"').trim();
        let display_name = if display.is_empty() {
            None
        } else {
            Some(display.to_string())
        };
        return (CanonicalAddress::new(address), display_name);
    }
    (CanonicalAddress::new(raw), None)
}

impl std::fmt::Display for CanonicalAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailSenderRenderDefault {
    #[default]
    Reader,
    Original,
}

impl EmailSenderRenderDefault {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reader => "reader",
            Self::Original => "original",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailDestination {
    Feed,
    Library,
}

impl EmailDestination {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Feed => "feed",
            Self::Library => "library",
        }
    }
}

impl std::fmt::Display for EmailDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSender {
    pub id: EmailSenderId,
    pub user_id: UserId,
    pub canonical_addr: String,
    pub list_id: Option<String>,
    pub display_name: Option<String>,
    pub render_default: EmailSenderRenderDefault,
    pub routing_default: Option<EmailDestination>,
    pub blocked_at: Option<DateTime<Utc>>,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub delivery_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EmailSender {
    pub fn is_blocked(&self) -> bool {
        self.blocked_at.is_some()
    }
}
