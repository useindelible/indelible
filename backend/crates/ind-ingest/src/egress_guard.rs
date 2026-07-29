use async_trait::async_trait;
use ind_egress::{EgressError, EgressPolicy, UrlRules, resolve_and_validate};

use ind_application::ports::{OutboundUrlGuard, UrlGuardError};

/// Adapter implementing the [`OutboundUrlGuard`] port over `ind-egress`. Used by
/// application services to reject user-supplied URLs that are unsafe to fetch
/// (private/internal/metadata hosts) at request time, before any work is queued.
pub struct EgressUrlGuard {
    policy: EgressPolicy,
}

impl EgressUrlGuard {
    pub fn new(policy: EgressPolicy) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl OutboundUrlGuard for EgressUrlGuard {
    async fn check_url(&self, url: &str) -> Result<(), UrlGuardError> {
        resolve_and_validate(url, &UrlRules::ingest(), &self.policy)
            .await
            .map_err(map_error)
    }
}

fn map_error(err: EgressError) -> UrlGuardError {
    let message = err.client_message().to_string();
    match err {
        EgressError::InvalidUrl
        | EgressError::DisallowedScheme { .. }
        | EgressError::SchemeRequiresHttps
        | EgressError::CredentialsInUrl
        | EgressError::FragmentNotAllowed
        | EgressError::MissingHost => UrlGuardError::Invalid(message),
        EgressError::HostNotAllowed { .. }
        | EgressError::PrivateAddress { .. }
        | EgressError::ResolutionFailed { .. }
        | EgressError::TooManyRedirects
        | EgressError::Transport(_) => UrlGuardError::Disallowed(message),
    }
}
