use http::HeaderMap;

use crate::state::{AppConfig, Environment};

const REFRESH_COOKIE_NAME: &str = "refresh";
const REFRESH_MAX_AGE_SECS: u64 = 30 * 24 * 60 * 60; // 30 days

pub fn set_refresh_cookie(headers: &mut HeaderMap, token: &str, config: &AppConfig) {
    let secure = config.environment != Environment::Development;
    let domain = cookie_domain_attribute(config);
    let cookie = format!(
        "{}={}; Path=/api/v1/auth; HttpOnly; SameSite=Lax; Max-Age={}{}{}",
        REFRESH_COOKIE_NAME,
        token,
        REFRESH_MAX_AGE_SECS,
        domain,
        if secure { "; Secure" } else { "" },
    );

    #[expect(
        clippy::expect_used,
        reason = "cookie string is assembled from a server-generated opaque token, an integer max-age, and config-derived attributes that contain only header-safe ASCII"
    )]
    let header_value = cookie.parse().expect("valid cookie header value");
    headers.append(http::header::SET_COOKIE, header_value);
}

pub fn clear_refresh_cookie(headers: &mut HeaderMap, config: &AppConfig) {
    let domain = cookie_domain_attribute(config);
    let cookie = format!(
        "{}=; Path=/api/v1/auth; HttpOnly; SameSite=Lax; Max-Age=0{}",
        REFRESH_COOKIE_NAME, domain,
    );
    #[expect(
        clippy::expect_used,
        reason = "cookie string is a static template plus a config-derived domain attribute that contains only header-safe ASCII"
    )]
    let header_value = cookie.parse().expect("valid cookie header value");
    headers.append(http::header::SET_COOKIE, header_value);
}

fn cookie_domain_attribute(config: &AppConfig) -> String {
    config
        .cookie_domain
        .as_ref()
        .map(|domain| format!("; Domain={domain}"))
        .unwrap_or_default()
}

pub fn extract_refresh_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get(http::header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|cookie| {
            let cookie = cookie.trim();
            cookie.strip_prefix("refresh=").map(|v| v.to_string())
        })
}
