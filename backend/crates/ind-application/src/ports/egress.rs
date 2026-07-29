use async_trait::async_trait;

/// Reason a user-supplied URL was refused by the outbound egress guard.
///
/// Transport-neutral so it can cross the hexagon boundary; handlers map both
/// variants to a domain validation error (HTTP 422). The concrete
/// implementation lives in `ind-ingest` over the `ind-egress` crate.
#[derive(Debug, Clone, thiserror::Error)]
pub enum UrlGuardError {
    /// The URL is syntactically invalid or uses a disallowed scheme/shape.
    #[error("{0}")]
    Invalid(String),
    /// The URL is well-formed but targets a private/internal/unresolvable host.
    #[error("{0}")]
    Disallowed(String),
}

impl UrlGuardError {
    pub fn message(&self) -> &str {
        match self {
            UrlGuardError::Invalid(m) | UrlGuardError::Disallowed(m) => m,
        }
    }
}

/// Port: validates that a user-supplied URL is safe to fetch server-side.
///
/// Implementations enforce the scheme allowlist and resolve the host, rejecting
/// private, loopback, link-local, and cloud-metadata targets (SSRF defense).
#[async_trait]
pub trait OutboundUrlGuard: Send + Sync {
    /// Validate a URL for ingest-class fetches (article/feed). Performs DNS
    /// resolution and rejects any host that resolves to a blocked address.
    async fn check_url(&self, url: &str) -> Result<(), UrlGuardError>;
}
