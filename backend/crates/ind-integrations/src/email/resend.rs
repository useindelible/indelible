use std::collections::HashMap;

use chrono::Utc;
use futures::StreamExt;
use http::HeaderMap;
use serde::Deserialize;

use super::provider::InboundEmailProvider;
use super::types::{EmailIngestError, EmailIngestProvider, InboundEmail, WebhookMetadata};

/// Hard cap on a fetched full-email payload. Resend caps inbound email at ~40MB;
/// this bounds memory if the API returns (or is coerced into returning) an
/// oversized or unbounded body.
const MAX_EMAIL_FETCH_BYTES: usize = 30 * 1024 * 1024;

pub struct ResendProvider {
    webhook: svix::webhooks::Webhook,
    api_key: String,
    http_client: reqwest::Client,
}

impl ResendProvider {
    pub fn new(webhook_secret: &str, api_key: String) -> Result<Self, EmailIngestError> {
        let webhook = svix::webhooks::Webhook::new(webhook_secret).map_err(|e| {
            EmailIngestError::MalformedPayload(format!("invalid webhook secret: {e}"))
        })?;
        Ok(Self {
            webhook,
            api_key,
            http_client: reqwest::Client::new(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct ResendWebhookPayload {
    data: ResendWebhookData,
}

#[derive(Debug, Deserialize)]
struct ResendWebhookData {
    email_id: String,
    to: Vec<String>,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    headers: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ResendFullEmail {
    from: String,
    to: Vec<String>,
    subject: String,
    html: Option<String>,
    text: Option<String>,
    #[serde(default)]
    headers: serde_json::Value,
}

fn parse_from_field(from: &str) -> (String, Option<String>) {
    if let Some(start) = from.rfind('<')
        && let Some(end) = from.rfind('>')
        && end > start
    {
        let address = from[start + 1..end].trim().to_string();
        let display = from[..start].trim().trim_matches('"').to_string();
        let display_name = if display.is_empty() {
            None
        } else {
            Some(display)
        };
        return (address, display_name);
    }
    (from.trim().to_string(), None)
}

/// Resend ships `List-Unsubscribe` and `List-Unsubscribe-Post` as a single
/// `list` header whose value is a JSON object (`{"unsubscribe": {"url", "mail"},
/// "unsubscribe-post": {"name"}}`). Flat RFC 2369 headers, when present, win.
fn resolve_list_unsubscribe(headers: &HashMap<String, String>) -> (Option<String>, Option<String>) {
    let flat_unsub = headers.get("list-unsubscribe").cloned();
    let flat_post = headers.get("list-unsubscribe-post").cloned();
    if flat_unsub.is_some() {
        return (flat_unsub, flat_post);
    }

    let Some(raw) = headers.get("list") else {
        return (None, flat_post);
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(raw) else {
        return (None, flat_post);
    };

    let mut parts: Vec<String> = Vec::new();
    if let Some(url) = parsed.pointer("/unsubscribe/url").and_then(|v| v.as_str()) {
        parts.push(format!("<{url}>"));
    }
    if let Some(mail) = parsed.pointer("/unsubscribe/mail").and_then(|v| v.as_str()) {
        parts.push(format!("<mailto:{mail}>"));
    }
    let list_unsubscribe = (!parts.is_empty()).then(|| parts.join(", "));

    let list_unsubscribe_post = flat_post.or_else(|| {
        parsed
            .pointer("/unsubscribe-post/name")
            .and_then(|v| v.as_str())
            .map(str::to_string)
    });

    (list_unsubscribe, list_unsubscribe_post)
}

/// Collects all headers into a HashMap with lowercase keys. Accepts both
/// Resend's array shape (`[{name, value}, ...]`) and the object shape
/// (`{name: value}`). On duplicate names, last value wins.
fn build_headers_map(headers: &serde_json::Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(arr) = headers.as_array() {
        for header in arr {
            if let (Some(name), Some(value)) = (
                header.get("name").and_then(|n| n.as_str()),
                header.get("value").and_then(|v| v.as_str()),
            ) {
                map.insert(name.to_ascii_lowercase(), value.to_string());
            }
        }
    } else if let Some(obj) = headers.as_object() {
        for (name, value) in obj {
            if let Some(s) = value.as_str() {
                map.insert(name.to_ascii_lowercase(), s.to_string());
            }
        }
    }
    map
}

#[async_trait::async_trait]
impl InboundEmailProvider for ResendProvider {
    fn provider(&self) -> EmailIngestProvider {
        EmailIngestProvider::Resend
    }

    fn verify_signature(&self, body: &[u8], headers: &HeaderMap) -> Result<(), EmailIngestError> {
        self.webhook
            .verify(body, headers)
            .map_err(|_| EmailIngestError::InvalidSignature)?;
        Ok(())
    }

    fn parse_webhook_metadata(&self, body: &[u8]) -> Result<WebhookMetadata, EmailIngestError> {
        let payload: ResendWebhookPayload = serde_json::from_slice(body)
            .map_err(|e| EmailIngestError::MalformedPayload(e.to_string()))?;

        if payload.data.email_id.is_empty() {
            return Err(EmailIngestError::MalformedPayload(
                "missing email_id".to_string(),
            ));
        }

        let headers = build_headers_map(&payload.data.headers);
        let list_id = headers.get("list-id").cloned();

        Ok(WebhookMetadata {
            provider_email_id: payload.data.email_id,
            to_addresses: payload.data.to,
            from_address: payload.data.from.unwrap_or_default(),
            list_id,
        })
    }

    async fn resolve_full_email(
        &self,
        metadata: &WebhookMetadata,
        _raw_payload: &[u8],
    ) -> Result<InboundEmail, EmailIngestError> {
        let url = format!(
            "https://api.resend.com/emails/receiving/{}",
            metadata.provider_email_id
        );

        let resp = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| EmailIngestError::ProviderApi(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(EmailIngestError::ProviderApi(format!(
                "Resend API returned {}",
                resp.status()
            )));
        }

        let body = read_capped_body(resp, MAX_EMAIL_FETCH_BYTES).await?;
        let full: ResendFullEmail = serde_json::from_slice(&body)
            .map_err(|e| EmailIngestError::ProviderApi(e.to_string()))?;

        Ok(build_inbound(full, metadata.provider_email_id.clone()))
    }
}

/// Read a response body into memory, refusing to buffer more than `max_bytes`.
/// Content-Length is rejected up front when it declares an oversized body, then
/// the body is streamed so an absent or understated length cannot blow past the
/// cap before deserialization.
async fn read_capped_body(
    resp: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, EmailIngestError> {
    if let Some(len) = resp.content_length()
        && len > max_bytes as u64
    {
        return Err(EmailIngestError::ProviderApi(format!(
            "email payload too large: {len} bytes (max {max_bytes})"
        )));
    }

    let mut stream = resp.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| EmailIngestError::ProviderApi(e.to_string()))?;
        if body.len() + chunk.len() > max_bytes {
            return Err(EmailIngestError::ProviderApi(
                "email payload exceeds size limit".to_string(),
            ));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn build_inbound(full: ResendFullEmail, provider_email_id: String) -> InboundEmail {
    let (from_address, from_display_name) = parse_from_field(&full.from);
    let headers = build_headers_map(&full.headers);
    let message_id = headers.get("message-id").cloned();
    let list_id = headers.get("list-id").cloned();
    let (list_unsubscribe, list_unsubscribe_post) = resolve_list_unsubscribe(&headers);

    InboundEmail {
        provider_email_id,
        from_address,
        from_display_name,
        to_addresses: full.to,
        subject: full.subject,
        html_body: full.html,
        text_body: full.text,
        message_id,
        headers,
        list_id,
        list_unsubscribe,
        list_unsubscribe_post,
        received_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests;
