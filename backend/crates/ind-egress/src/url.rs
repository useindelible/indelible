use std::net::IpAddr;

use ::url::{Host, Url};

use crate::error::EgressError;
use crate::policy::{EgressPolicy, UrlRules};

/// Synchronously validate a URL string against a surface's [`UrlRules`] and the
/// active [`EgressPolicy`]. Returns the parsed [`Url`] on success.
///
/// This rejects disallowed schemes, embedded credentials, fragments (where
/// disallowed), missing hosts, `localhost`, and IP-literal hosts that classify
/// as private/internal. Domain hosts that need DNS resolution are checked later
/// by the resolver layer.
pub fn validate_url(
    raw: &str,
    rules: &UrlRules,
    policy: &EgressPolicy,
) -> Result<Url, EgressError> {
    let trimmed = raw.trim();
    let url = Url::parse(trimmed).map_err(|_| EgressError::InvalidUrl)?;

    if !rules.allowed_schemes().contains(&url.scheme()) {
        return Err(EgressError::DisallowedScheme {
            scheme: url.scheme().to_string(),
        });
    }

    // BYOK-secret protection: plain http is permitted only when the host is
    // unambiguously local/private (localhost or a private/loopback IP literal).
    // A public host — even with `allow_private_targets` on — must use https so
    // credentials are never sent in cleartext.
    if rules.https_required_unless_private()
        && url.scheme() == "http"
        && !host_is_local_or_private(&url)
    {
        return Err(EgressError::SchemeRequiresHttps);
    }

    if !url.username().is_empty() || url.password().is_some() {
        return Err(EgressError::CredentialsInUrl);
    }

    if !rules.allow_fragment() && url.fragment().is_some() {
        return Err(EgressError::FragmentNotAllowed);
    }

    match url.host() {
        None => return Err(EgressError::MissingHost),
        Some(Host::Domain(domain)) => {
            if is_localhost_domain(domain) && !policy.allow_private_targets {
                return Err(EgressError::HostNotAllowed {
                    host: domain.to_string(),
                });
            }
        }
        Some(Host::Ipv4(v4)) => {
            let ip = IpAddr::V4(v4);
            if !policy.ip_permitted(ip) {
                return Err(EgressError::PrivateAddress {
                    host: v4.to_string(),
                    ip,
                });
            }
        }
        Some(Host::Ipv6(v6)) => {
            let ip = IpAddr::V6(v6);
            if !policy.ip_permitted(ip) {
                return Err(EgressError::PrivateAddress {
                    host: v6.to_string(),
                    ip,
                });
            }
        }
    }

    Ok(url)
}

/// True when the URL host is unambiguously local/private (without DNS): a
/// `localhost` name or a private/loopback/link-local IP literal.
fn host_is_local_or_private(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(d)) => is_localhost_domain(d),
        Some(Host::Ipv4(v4)) => crate::ip::is_blocked_ip(IpAddr::V4(v4)),
        Some(Host::Ipv6(v6)) => crate::ip::is_blocked_ip(IpAddr::V6(v6)),
        None => false,
    }
}

/// `localhost` or any `*.localhost` name (RFC 6761 reserves these to loopback).
fn is_localhost_domain(domain: &str) -> bool {
    let domain = domain.trim_end_matches('.');
    domain.eq_ignore_ascii_case("localhost")
        || domain
            .rsplit_once('.')
            .is_some_and(|(_, tld)| tld.eq_ignore_ascii_case("localhost"))
}
