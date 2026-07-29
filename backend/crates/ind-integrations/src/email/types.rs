use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use ind_domain::EmailDestination;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailIngestProvider {
    Resend,
    Ses,
    Postmark,
    SendGrid,
    Mailgun,
}

impl EmailIngestProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Resend => "resend",
            Self::Ses => "ses",
            Self::Postmark => "postmark",
            Self::SendGrid => "sendgrid",
            Self::Mailgun => "mailgun",
        }
    }
}

impl std::fmt::Display for EmailIngestProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct WebhookMetadata {
    pub provider_email_id: String,
    pub to_addresses: Vec<String>,
    /// Verbatim `From:` header (e.g. `"Display Name <addr@host>"`). Empty if the
    /// provider does not include the from address in its webhook payload — the
    /// claim path will still record the delivery but cannot enforce sender blocks.
    pub from_address: String,
    /// Verbatim RFC 2919 `List-ID:` header (with brackets), if present.
    pub list_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InboundEmail {
    pub provider_email_id: String,
    pub from_address: String,
    pub from_display_name: Option<String>,
    pub to_addresses: Vec<String>,
    pub subject: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
    pub message_id: Option<String>,
    /// All headers from the provider, with lowercase keys.
    pub headers: HashMap<String, String>,
    /// RFC 2919 List-ID, verbatim from the header (including `<...>` brackets).
    pub list_id: Option<String>,
    /// RFC 2369 List-Unsubscribe, verbatim.
    pub list_unsubscribe: Option<String>,
    /// RFC 8058 List-Unsubscribe-Post (one-click), verbatim.
    pub list_unsubscribe_post: Option<String>,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum EmailIngestError {
    #[error("invalid webhook signature")]
    InvalidSignature,

    #[error("malformed webhook payload: {0}")]
    MalformedPayload(String),

    #[error("provider API error: {0}")]
    ProviderApi(String),

    #[error("provider not configured")]
    NotConfigured,

    #[error("provider not implemented: {0}")]
    NotImplemented(String),
}
