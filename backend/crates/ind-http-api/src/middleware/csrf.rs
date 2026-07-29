use axum::body::Body;
use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;
use http::{HeaderMap, Method, Request};

use crate::error::ApiError;
use crate::state::{AppConfig, AppState};

pub struct CsrfToken(pub String);

const REFRESH_COOKIE_NAME: &str = "refresh";
const PROTECTED_PATHS: &[&str] = &["/api/v1/auth/refresh", "/api/v1/auth/logout"];

fn is_safe_method(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

fn carries_refresh_cookie(headers: &HeaderMap) -> bool {
    headers
        .get(http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .map(|cookies| {
            cookies
                .split(';')
                .map(str::trim)
                .any(|cookie| cookie.starts_with(&format!("{REFRESH_COOKIE_NAME}=")))
        })
        .unwrap_or(false)
}

fn is_protected_path(path: &str) -> bool {
    PROTECTED_PATHS.contains(&path)
}

fn is_trusted_origin(origin: &str, config: &AppConfig) -> bool {
    let origin = origin.trim_end_matches('/');
    [config.base_url.as_str(), config.frontend_url.as_str()]
        .into_iter()
        .chain(config.cors_origins.iter().map(String::as_str))
        .any(|trusted| trusted.trim_end_matches('/') == origin)
}

fn has_allowed_fetch_site(headers: &HeaderMap) -> bool {
    headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok())
        .map(|site| matches!(site, "same-origin" | "same-site" | "none"))
        .unwrap_or(false)
}

fn validate_cookie_csrf(
    method: &Method,
    path: &str,
    headers: &HeaderMap,
    config: &AppConfig,
) -> Result<(), ApiError> {
    let has_refresh_cookie = carries_refresh_cookie(headers);

    if is_safe_method(method) || !is_protected_path(path) || !has_refresh_cookie {
        return Ok(());
    }

    if let Some(origin) = headers
        .get(http::header::ORIGIN)
        .and_then(|value| value.to_str().ok())
    {
        if is_trusted_origin(origin, config) {
            return Ok(());
        }

        return Err(ApiError::Forbidden {
            message: "cross-site request blocked".to_string(),
        });
    }

    if has_allowed_fetch_site(headers) {
        return Ok(());
    }

    Err(ApiError::Forbidden {
        message: "browser origin proof required".to_string(),
    })
}

pub async fn csrf_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    validate_cookie_csrf(
        request.method(),
        request.uri().path(),
        request.headers(),
        &state.config,
    )?;
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AppConfig {
        AppConfig {
            base_url: "https://ind.example.com".to_string(),
            frontend_url: "https://app.example.com".to_string(),
            cors_origins: vec![
                "https://app.example.com".to_string(),
                "https://ind.tail1234.ts.net".to_string(),
            ],
            ..AppConfig::default()
        }
    }

    #[test]
    fn base_and_frontend_urls_are_trusted() {
        assert!(is_trusted_origin("https://ind.example.com", &config()));
        assert!(is_trusted_origin("https://app.example.com", &config()));
    }

    #[test]
    fn a_second_configured_hostname_is_trusted() {
        // LAN name plus tailnet name is a normal homelab setup: the operator lists
        // both in CORS_ORIGINS, so refresh must not 403 on the one that is neither
        // base_url nor frontend_url.
        assert!(is_trusted_origin("https://ind.tail1234.ts.net", &config()));
    }

    #[test]
    fn trailing_slashes_in_config_do_not_break_matching() {
        let config = AppConfig {
            base_url: "https://ind.example.com/".to_string(),
            ..config()
        };
        assert!(is_trusted_origin("https://ind.example.com", &config));
    }

    #[test]
    fn unknown_origins_are_rejected() {
        assert!(!is_trusted_origin("https://evil.example.com", &config()));
    }
}
