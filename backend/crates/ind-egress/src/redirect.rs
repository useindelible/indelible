use std::sync::Arc;

use reqwest::redirect::{Attempt, Policy};

use crate::error::EgressError;
use crate::policy::{EgressPolicy, UrlRules};

/// Pure per-hop check used by the redirect policy. Re-validates a redirect
/// target's scheme, credentials, and IP-literal host. Domain hosts are covered
/// at connect time by the guarded DNS resolver.
pub fn check_hop_url(
    url: &::url::Url,
    rules: &UrlRules,
    policy: &EgressPolicy,
) -> Result<(), EgressError> {
    crate::url::validate_url(url.as_str(), rules, policy).map(|_| ())
}

/// Build a redirect [`Policy`] that caps hops at `max_hops` (0 = follow none)
/// and re-validates every hop. `max_hops == 0` is the correct choice for
/// webhook delivery, where receivers should not redirect.
pub(crate) fn guarded_redirect_policy(
    rules: UrlRules,
    policy: Arc<EgressPolicy>,
    max_hops: usize,
) -> Policy {
    Policy::custom(move |attempt: Attempt| {
        if attempt.previous().len() > max_hops {
            return attempt.error(EgressError::TooManyRedirects);
        }
        match check_hop_url(attempt.url(), &rules, &policy) {
            Ok(()) => attempt.follow(),
            Err(e) => attempt.error(e),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hop_validation_preserves_webhook_downgrade_and_public_host_boundaries() {
        let url = ::url::Url::parse("http://example.com/x").unwrap();
        let err = check_hop_url(&url, &UrlRules::webhook(), &EgressPolicy::strict());
        assert!(matches!(err, Err(EgressError::DisallowedScheme { .. })));
        let url = ::url::Url::parse("https://example.com/x").unwrap();
        assert!(check_hop_url(&url, &UrlRules::ingest(), &EgressPolicy::strict()).is_ok());
    }
}
