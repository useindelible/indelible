pub mod cleaner;
pub mod gmail_confirm;
pub mod parsing;
pub mod provider;
pub mod resend;
pub mod types;
pub mod unsubscribe;

pub use cleaner::{clean_email_html, prepare_email_for_reader};
pub use parsing::{
    ContentMode, ParsedIngestAddress, detect_content_mode, extract_primary_url,
    parse_ingest_address,
};
pub use provider::InboundEmailProvider;
pub use resend::ResendProvider;
pub use types::{
    EmailDestination, EmailIngestError, EmailIngestProvider, InboundEmail, WebhookMetadata,
};
pub use unsubscribe::{UnsubscribeTargets, parse_unsubscribe_targets};
