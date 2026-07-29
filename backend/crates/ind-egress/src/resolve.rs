use std::net::IpAddr;

use crate::error::EgressError;
use crate::policy::{EgressPolicy, UrlRules};

/// Resolve `host` to addresses via the system resolver and reject the host if
/// *any* resolved address is not permitted by `policy`. Returns the vetted
/// addresses. IP-literal hosts are classified directly without a DNS lookup.
pub async fn resolve_host(
    host: &str,
    port: u16,
    policy: &EgressPolicy,
) -> Result<Vec<IpAddr>, EgressError> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        if !policy.ip_permitted(ip) {
            return Err(EgressError::PrivateAddress {
                host: host.to_string(),
                ip,
            });
        }
        return Ok(vec![ip]);
    }

    let resolved =
        tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| EgressError::ResolutionFailed {
                host: host.to_string(),
            })?;

    let mut ips = Vec::new();
    for sa in resolved {
        let ip = sa.ip();
        if !policy.ip_permitted(ip) {
            return Err(EgressError::PrivateAddress {
                host: host.to_string(),
                ip,
            });
        }
        ips.push(ip);
    }

    if ips.is_empty() {
        return Err(EgressError::ResolutionFailed {
            host: host.to_string(),
        });
    }

    Ok(ips)
}

/// Async pre-flight: validate a URL string and confirm its host resolves only
/// to permitted addresses. Used where no request is issued through a
/// [`crate::GuardedHttpClient`] (e.g. the Chromium renderer validating a
/// navigation target before `page.goto`).
pub async fn resolve_and_validate(
    raw: &str,
    rules: &UrlRules,
    policy: &EgressPolicy,
) -> Result<(), EgressError> {
    let url = crate::url::validate_url(raw, rules, policy)?;
    let host = url.host_str().ok_or(EgressError::MissingHost)?;
    let port = url.port_or_known_default().unwrap_or(0);
    resolve_host(host, port, policy).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn localhost_domain_rejected_at_validation() {
        let err = resolve_and_validate(
            "http://localhost:9/x",
            &UrlRules::ingest(),
            &EgressPolicy::strict(),
        )
        .await;
        assert!(err.is_err());
    }
}
