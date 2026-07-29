use std::time::Duration;

use chrono::{DateTime, Utc};
use ind_application::ports::TtsAdapterError;
use reqwest::StatusCode;
use serde::Deserialize;

/// Shared request timeouts for TTS provider adapters. Values are conservative
/// defaults that can be tuned per-deployment if latency characteristics
/// diverge; adapters intentionally share the same defaults so the timeout
/// semantics of `TtsAdapterError::Timeout` stay consistent.
#[derive(Debug, Clone, Copy)]
pub struct TtsHttpTimeouts {
    pub connect: Duration,
    pub request: Duration,
}

impl Default for TtsHttpTimeouts {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(10),
            request: Duration::from_secs(60),
        }
    }
}

/// Build a reqwest client configured with the adapter's shared timeout and
/// connection-pool settings. Separate helper so every adapter uses identical
/// settings without duplicating the config.
pub fn build_client(timeouts: TtsHttpTimeouts) -> Result<reqwest::Client, TtsAdapterError> {
    reqwest::Client::builder()
        .connect_timeout(timeouts.connect)
        .timeout(timeouts.request)
        .build()
        .map_err(|e| TtsAdapterError::ProviderUnreachable(format!("client build failed: {e}")))
}

/// Strip a trailing `/api/v1` suffix from a provider base URL so the adapter
/// can append its own path segments without double-writing the version
/// prefix. DashScope deployments are often pasted as either
/// `https://dashscope-intl.aliyuncs.com` or
/// `https://dashscope-intl.aliyuncs.com/api/v1`; both must work.
pub fn normalize_dashscope_base(base: &str) -> String {
    let trimmed = base.trim().trim_end_matches('/');
    if let Some(without_v1) = trimmed.strip_suffix("/api/v1") {
        without_v1.to_string()
    } else {
        trimmed.to_string()
    }
}

/// Map a reqwest transport-layer error to the right `TtsAdapterError`
/// variant. Adapters reuse this so timeout vs. network vs. decode errors are
/// categorized identically.
pub fn classify_transport_error(err: reqwest::Error) -> TtsAdapterError {
    if err.is_timeout() {
        TtsAdapterError::Timeout
    } else if err.is_connect() || err.is_request() {
        TtsAdapterError::ProviderUnreachable(err.to_string())
    } else if err.is_decode() || err.is_body() {
        TtsAdapterError::MalformedResponse(err.to_string())
    } else {
        TtsAdapterError::ProviderUnreachable(err.to_string())
    }
}

/// Map an HTTP response status to the right `TtsAdapterError` variant,
/// pulling a `Retry-After` header into the structured `RateLimited` variant
/// when present.
///
/// DashScope returns `403` in two overlapping situations: truly forbidden
/// calls (bad key / revoked access) and quota-exhausted ones (the error body
/// then carries a `Throttling.*` or `LimitExceeded` code). The two need
/// different retry semantics for the session orchestrator, so this function
/// peeks into the body to disambiguate.
pub fn classify_status_error(
    status: StatusCode,
    retry_after_ms: Option<u64>,
    body_message: String,
) -> TtsAdapterError {
    match status {
        StatusCode::UNAUTHORIZED => TtsAdapterError::AuthenticationFailed(body_message),
        StatusCode::FORBIDDEN => {
            if is_quota_code(&body_message) {
                TtsAdapterError::QuotaExhausted
            } else {
                TtsAdapterError::AuthenticationFailed(body_message)
            }
        }
        StatusCode::TOO_MANY_REQUESTS => TtsAdapterError::RateLimited { retry_after_ms },
        StatusCode::PAYMENT_REQUIRED => TtsAdapterError::QuotaExhausted,
        StatusCode::REQUEST_TIMEOUT | StatusCode::GATEWAY_TIMEOUT => TtsAdapterError::Timeout,
        StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
            TtsAdapterError::InvalidRequest(body_message)
        }
        s => TtsAdapterError::ProviderError {
            status_code: s.as_u16(),
            message: body_message,
        },
    }
}

#[derive(Debug, Deserialize)]
struct ProviderErrorBody {
    #[serde(default)]
    code: Option<String>,
}

fn is_quota_code(body: &str) -> bool {
    let parsed: Option<ProviderErrorBody> = serde_json::from_str(body).ok();
    match parsed.and_then(|p| p.code) {
        Some(code) => {
            let lowered = code.to_ascii_lowercase();
            lowered.starts_with("throttling.") || lowered == "limitexceeded"
        }
        None => false,
    }
}

/// Parse a `Retry-After` header into milliseconds. Supports the
/// delta-seconds form (RFC 7231 §7.1.3) and the HTTP-date form (IMF-fixdate
/// per RFC 7231 §7.1.1.1 — always GMT). DashScope predominantly emits
/// seconds, but upstream proxies and some Unreal edge caches emit an IMF
/// date; both resolve to the same delay semantics. Returns `None` when the
/// header is absent, empty, or malformed.
pub fn parse_retry_after_ms(value: Option<&str>) -> Option<u64> {
    let raw = value?.trim();
    if raw.is_empty() {
        return None;
    }
    if let Ok(seconds) = raw.parse::<u64>() {
        return Some(seconds.saturating_mul(1000));
    }
    // HTTP-date IMF-fixdate: "Sun, 06 Nov 1994 08:49:37 GMT". chrono parses
    // this via NaiveDateTime and we then stamp UTC on it (the spec fixes the
    // zone at GMT, which equals UTC for our purposes).
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(raw, "%a, %d %b %Y %H:%M:%S GMT") {
        let when: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
        let delta = when - Utc::now();
        let ms = delta.num_milliseconds();
        if ms <= 0 {
            return Some(0);
        }
        return Some(ms as u64);
    }
    if let Ok(when) = DateTime::parse_from_rfc2822(raw) {
        let delta = when.with_timezone(&Utc) - Utc::now();
        let ms = delta.num_milliseconds();
        if ms <= 0 {
            return Some(0);
        }
        return Some(ms as u64);
    }
    None
}
