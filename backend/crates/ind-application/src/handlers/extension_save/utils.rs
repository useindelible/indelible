use bytes::Bytes;

use crate::error::AppError;
use ind_domain::CanonicalizationConfig;

pub(super) fn canonicalize_url(raw: &str) -> String {
    ind_domain::canonicalize_url(raw, &CanonicalizationConfig::default())
        .map(|c| c.into_string())
        .unwrap_or_else(|_| raw.to_string())
}
pub(super) fn resolved_canonical_url(url: &str, page_canonical_url: Option<&str>) -> String {
    let url_canonical = canonicalize_url(url);

    let Some(page_canonical_url) = page_canonical_url
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return url_canonical;
    };

    let page_canonical = canonicalize_url(page_canonical_url);
    if is_trusted_page_canonical(&url_canonical, &page_canonical) {
        page_canonical
    } else {
        tracing::debug!(
            url = %url,
            page_canonical_url = %page_canonical_url,
            resolved_canonical_url = %url_canonical,
            "ignoring untrusted page canonical URL from extension"
        );
        url_canonical
    }
}
pub(super) fn is_trusted_page_canonical(url_canonical: &str, page_canonical: &str) -> bool {
    let Ok(url) = url::Url::parse(url_canonical) else {
        return false;
    };
    let Ok(page) = url::Url::parse(page_canonical) else {
        return false;
    };

    if url.host_str() != page.host_str() {
        return false;
    }

    normalized_path(url.path()) == normalized_path(page.path())
}
pub(super) fn normalized_path(path: &str) -> &str {
    if path.len() > 1 {
        path.trim_end_matches('/')
    } else {
        path
    }
}
pub(super) fn base64_decode(input: &str) -> Result<Bytes, AppError> {
    use ind_domain::DomainError;

    // Strip data URI prefix if present (e.g. "data:text/html;base64,...")
    let raw = if let Some(pos) = input.find(",") {
        &input[pos + 1..]
    } else {
        input
    };

    let decoded = base64_engine_decode(raw).map_err(|_| {
        AppError::Domain(DomainError::Validation {
            field: "base64".into(),
            message: "invalid base64 encoding".into(),
        })
    })?;

    Ok(Bytes::from(decoded))
}
pub(super) fn base64_engine_decode(input: &str) -> Result<Vec<u8>, ()> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|_| ())
}
pub(super) fn extract_domain(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
}

/// Reject non-http(s) `lead_image_url` values (e.g. `javascript:`, `data:`).
/// The value is stored as metadata and later bound to `<img src>` by web/mobile
/// clients; restricting the scheme prevents script/tracking-pixel sinks (U.3).
pub(super) fn validate_lead_image_url(lead_image_url: &Option<String>) -> Result<(), AppError> {
    use ind_domain::DomainError;

    let Some(raw) = lead_image_url else {
        return Ok(());
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let parsed = url::Url::parse(trimmed).map_err(|_| {
        AppError::Domain(DomainError::Validation {
            field: "lead_image_url".into(),
            message: "must be a valid URL".into(),
        })
    })?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::Domain(DomainError::Validation {
            field: "lead_image_url".into(),
            message: "must be an http(s) URL".into(),
        }));
    }
    Ok(())
}
