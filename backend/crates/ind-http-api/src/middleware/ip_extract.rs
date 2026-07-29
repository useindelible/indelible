use std::convert::Infallible;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{ConnectInfo, FromRequestParts};
use http::request::Parts;
use http::{HeaderMap, Request};
use ipnet::IpNet;

use crate::state::AppState;

const FALLBACK_IP: IpAddr = IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1));

/// Allowlist of reverse-proxy addresses whose forwarded headers we trust.
/// Empty (the default) means: trust no forwarded headers — use the direct peer.
#[derive(Debug, Clone, Default)]
pub struct TrustedProxies {
    entries: Arc<Vec<IpNet>>,
}

impl TrustedProxies {
    /// Parse entries as bare IPs or CIDR blocks (`10.0.0.5`, `10.0.0.0/8`,
    /// `2001:db8::/32`). Unparseable entries are skipped with a warning so a
    /// single typo doesn't silently trust everything.
    pub fn from_entries<I, S>(entries: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let parsed = entries
            .into_iter()
            .filter_map(|e| {
                let e = e.as_ref().trim();
                match parse_proxy_entry(e) {
                    Some(net) => Some(net),
                    None => {
                        tracing::warn!(entry = %e, "ignoring invalid trusted_proxies entry");
                        None
                    }
                }
            })
            .collect();
        Self {
            entries: Arc::new(parsed),
        }
    }

    fn trusts(&self, ip: IpAddr) -> bool {
        let ip = normalize_mapped_v4(ip);
        self.entries.iter().any(|net| net.contains(&ip))
    }
}

/// Parse a trusted-proxy entry as a CIDR block, falling back to a bare host
/// address (treated as a /32 or /128).
fn parse_proxy_entry(raw: &str) -> Option<IpNet> {
    raw.parse::<IpNet>()
        .ok()
        .or_else(|| raw.parse::<IpAddr>().ok().map(IpNet::from))
}

/// Collapse an IPv4-mapped IPv6 address (`::ffff:10.0.0.5`) to its IPv4 form so
/// it matches IPv4 CIDR entries; dual-stack listeners can surface peers this
/// way. Other addresses pass through unchanged.
fn normalize_mapped_v4(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(ip),
        v4 => v4,
    }
}

/// Extracts the real client IP address.
///
/// Forwarded headers are honored only when the direct peer is a configured
/// trusted proxy; otherwise the direct socket peer is used.
///
/// `X-Forwarded-For` is parsed RIGHT-TO-LEFT: the rightmost entries are the ones
/// our own trusted proxies appended (e.g. nginx `$proxy_add_x_forwarded_for`),
/// so we skip trusted-proxy hops and return the first untrusted hop — the real
/// client. A client can forge the leftmost entries, but it cannot forge the hops
/// our proxies append after it, so taking the leftmost value would let any client
/// spoof its IP (bypassing IP rate limits / polluting audit attribution). If no
/// untrusted hop is found we fall back to `X-Real-IP`, then the direct peer.
pub fn extract_client_ip<B>(req: &Request<B>, trusted: &TrustedProxies) -> IpAddr {
    let peer = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip());
    resolve_client_ip(req.headers(), peer, trusted)
}

/// Core client-IP resolution shared by the middleware (`extract_client_ip`) and
/// the [`ClientIp`] extractor. See `extract_client_ip` for the right-to-left
/// trust rules.
pub fn resolve_client_ip(
    headers: &HeaderMap,
    peer: Option<IpAddr>,
    trusted: &TrustedProxies,
) -> IpAddr {
    let trust_headers = peer.is_some_and(|p| trusted.trusts(p));

    if trust_headers {
        if let Some(forwarded_for) = headers.get("x-forwarded-for")
            && let Ok(value) = forwarded_for.to_str()
            && let Some(client) = rightmost_untrusted(value, trusted)
        {
            return client;
        }
        if let Some(real_ip) = headers.get("x-real-ip")
            && let Ok(value) = real_ip.to_str()
            && let Ok(ip) = value.trim().parse::<IpAddr>()
        {
            return ip;
        }
    }

    peer.unwrap_or(FALLBACK_IP)
}

/// Axum extractor yielding the resolved client IP (trusted-proxy aware, same
/// rules as [`extract_client_ip`]). Holds `None` when there is no socket peer to
/// attribute (e.g. unit tests without `ConnectInfo`), so audit/session writes
/// record a real address or nothing — never a spoofable header value.
pub struct ClientIp(pub Option<IpAddr>);

impl ClientIp {
    /// String form for audit/session persistence (`None` when unattributable).
    pub fn audit_string(&self) -> Option<String> {
        self.0.map(|ip| ip.to_string())
    }
}

impl FromRequestParts<AppState> for ClientIp {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0.ip());
        let ip = peer.map(|p| resolve_client_ip(&parts.headers, Some(p), &state.trusted_proxies));
        Ok(ClientIp(ip))
    }
}

/// Walks `X-Forwarded-For` from rightmost to leftmost, skipping entries that are
/// themselves trusted proxies, and returns the first untrusted (client) IP.
/// Stops at the first unparseable hop so a malformed entry can't expose a
/// client-forged value to its left.
fn rightmost_untrusted(header: &str, trusted: &TrustedProxies) -> Option<IpAddr> {
    for hop in header.rsplit(',') {
        let ip = hop.trim().parse::<IpAddr>().ok()?;
        if !trusted.trusts(ip) {
            return Some(ip);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(value: &str) -> IpAddr {
        value.parse().unwrap()
    }

    #[test]
    fn forwarded_chain_is_honored_only_from_trusted_rightmost_hops() {
        let trusted = TrustedProxies::from_entries(["10.0.0.0/8", "::ffff:10.0.0.5"]);
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "1.2.3.4, 203.0.113.7, 10.0.0.2".parse().unwrap(),
        );
        for (peer, expected) in [
            ("10.0.0.5", "203.0.113.7"),
            ("198.51.100.9", "198.51.100.9"),
            ("::ffff:10.0.0.5", "203.0.113.7"),
        ] {
            assert_eq!(
                resolve_client_ip(&headers, Some(ip(peer)), &trusted),
                ip(expected)
            );
        }
    }

    #[test]
    fn malformed_or_absent_attribution_fails_closed() {
        let trusted = TrustedProxies::from_entries(["10.0.0.5"]);
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "spoof, invalid".parse().unwrap());
        headers.insert("x-real-ip", "198.51.100.1".parse().unwrap());
        assert_eq!(
            resolve_client_ip(&headers, Some(ip("10.0.0.5")), &trusted),
            ip("198.51.100.1")
        );
        assert_eq!(resolve_client_ip(&headers, None, &trusted), FALLBACK_IP);
    }
}
