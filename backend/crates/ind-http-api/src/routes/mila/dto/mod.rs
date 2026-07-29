use crate::error::FieldError;
use ind_domain::{AiPromptAction, MessageRole, MilaSessionType};

mod config;
mod presets;
mod sessions;

pub use config::*;
pub use presets::*;
pub use sessions::*;

// HTTP DTOs stay explicit even when they mirror application outputs so ind-application never depends on utoipa.
pub const VALID_PROMPT_ACTIONS: &[&str] = &["summary", "tags", "entities", "chat", "custom"];
pub const VALID_MILA_SESSION_TYPES: &[&str] = &["single_document", "cross_item", "collection"];

pub fn parse_prompt_action(value: &str) -> Option<AiPromptAction> {
    match value.trim().to_ascii_lowercase().as_str() {
        "summary" => Some(AiPromptAction::Summary),
        "tags" => Some(AiPromptAction::Tags),
        "entities" => Some(AiPromptAction::Entities),
        "chat" => Some(AiPromptAction::Chat),
        "custom" => Some(AiPromptAction::Custom),
        _ => None,
    }
}

pub fn parse_mila_session_type(value: &str) -> Option<MilaSessionType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "single_document" => Some(MilaSessionType::SingleDocument),
        "cross_item" => Some(MilaSessionType::CrossItem),
        "collection" => Some(MilaSessionType::Collection),
        _ => None,
    }
}

pub fn format_prompt_action(value: AiPromptAction) -> &'static str {
    match value {
        AiPromptAction::Summary => "summary",
        AiPromptAction::Tags => "tags",
        AiPromptAction::Entities => "entities",
        AiPromptAction::Chat => "chat",
        AiPromptAction::Custom => "custom",
    }
}

fn format_session_type(value: MilaSessionType) -> &'static str {
    match value {
        MilaSessionType::SingleDocument => "single_document",
        MilaSessionType::CrossItem => "cross_item",
        MilaSessionType::Collection => "collection",
    }
}

fn format_message_role(value: MessageRole) -> &'static str {
    match value {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
    }
}

fn parse_optional_id<T>(field: &str, raw: Option<String>) -> Result<Option<T>, Vec<FieldError>>
where
    T: std::str::FromStr,
{
    let Some(raw) = raw else {
        return Ok(None);
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    trimmed.parse::<T>().map(Some).map_err(|_| {
        vec![FieldError {
            field: field.into(),
            message: format!("must be a valid {field}"),
        }]
    })
}

fn validate_required(field: &str, value: &str, errors: &mut Vec<FieldError>) {
    if value.trim().is_empty() {
        errors.push(FieldError {
            field: field.into(),
            message: "must not be empty".into(),
        });
    }
}

fn validate_positive(field: &str, value: i32, errors: &mut Vec<FieldError>) {
    if value <= 0 {
        errors.push(FieldError {
            field: field.into(),
            message: "must be greater than 0".into(),
        });
    }
}

/// Reject obviously-malformed provider URLs early. The runtime SSRF guard
/// (`ind-egress`) still enforces the resolved-IP/scheme policy on every call,
/// including the connectivity test; this only catches non-URL / non-http(s)
/// input at request time for a clearer error.
fn validate_http_url(field: &str, value: &str, errors: &mut Vec<FieldError>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return;
    }
    let is_http = url::Url::parse(trimmed)
        .map(|u| matches!(u.scheme(), "http" | "https"))
        .unwrap_or(false);
    if !is_http {
        errors.push(FieldError {
            field: field.into(),
            message: "must be an http(s) URL".into(),
        });
    }
}
