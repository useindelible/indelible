use std::net::IpAddr;

use crate::ip::is_blocked_ip;

/// Runtime decision inputs for the guard.
///
/// `allow_private_targets` is the self-host escape hatch (default off; hosted
/// SaaS never sets it). `extra_allowed_ips` pins specific addresses through the
/// block list — used by tests to permit a loopback listener while still
/// rejecting every other private address.
#[derive(Debug, Clone, Default)]
pub struct EgressPolicy {
    pub allow_private_targets: bool,
    pub extra_allowed_ips: Vec<IpAddr>,
}

impl EgressPolicy {
    /// Block all private/internal targets (production default).
    pub fn strict() -> Self {
        Self::default()
    }

    /// Permit every target. Used by the test harness, whose fixtures bind to
    /// loopback.
    pub fn permissive() -> Self {
        Self {
            allow_private_targets: true,
            extra_allowed_ips: Vec::new(),
        }
    }

    /// Returns `true` when the policy permits a connection to `ip`.
    pub fn ip_permitted(&self, ip: IpAddr) -> bool {
        self.allow_private_targets || self.extra_allowed_ips.contains(&ip) || !is_blocked_ip(ip)
    }
}

/// Per-surface URL syntax rules applied before any network access.
#[derive(Debug, Clone, Copy)]
pub struct UrlRules {
    allowed_schemes: &'static [&'static str],
    allow_fragment: bool,
    /// When set, plain `http` is rejected unless `allow_private_targets` is on
    /// (a private/loopback self-host endpoint). Protects BYOK secrets from
    /// being sent in cleartext to a public host.
    https_required_unless_private: bool,
}

const HTTP_HTTPS: &[&str] = &["http", "https"];
const HTTPS_ONLY: &[&str] = &["https"];

impl UrlRules {
    /// Article/feed ingestion: http or https.
    pub fn ingest() -> Self {
        Self {
            allowed_schemes: HTTP_HTTPS,
            allow_fragment: true,
            https_required_unless_private: false,
        }
    }

    /// Outbound webhook delivery: https only, no fragment.
    pub fn webhook() -> Self {
        Self {
            allowed_schemes: HTTPS_ONLY,
            allow_fragment: false,
            https_required_unless_private: false,
        }
    }

    /// AI provider endpoint (BYOK): https for public hosts; http only for a
    /// private/loopback self-host endpoint when `allow_private_targets` is on.
    pub fn ai_endpoint() -> Self {
        Self {
            allowed_schemes: HTTP_HTTPS,
            allow_fragment: true,
            https_required_unless_private: true,
        }
    }

    pub(crate) fn allowed_schemes(&self) -> &'static [&'static str] {
        self.allowed_schemes
    }

    pub(crate) fn allow_fragment(&self) -> bool {
        self.allow_fragment
    }

    pub(crate) fn https_required_unless_private(&self) -> bool {
        self.https_required_unless_private
    }
}
