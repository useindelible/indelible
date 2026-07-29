use std::net::SocketAddr;
use std::sync::Arc;

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use crate::policy::EgressPolicy;

/// A [`reqwest::dns::Resolve`] implementation that filters every resolved
/// address through the [`EgressPolicy`]. Because the connector only ever
/// connects to the addresses this resolver returns, a host that resolves to any
/// blocked address is refused — defeating DNS rebinding and covering every
/// redirect hop. IP-literal hosts never reach the resolver, so the synchronous
/// URL validation must classify those (see [`crate::validate_url`]).
pub(crate) struct GuardedDnsResolver {
    policy: Arc<EgressPolicy>,
}

impl GuardedDnsResolver {
    pub(crate) fn new(policy: Arc<EgressPolicy>) -> Self {
        Self { policy }
    }
}

impl Resolve for GuardedDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let policy = self.policy.clone();
        let host = name.as_str().to_string();
        Box::pin(async move {
            let resolved = tokio::net::lookup_host((host.as_str(), 0)).await?;
            let mut out: Vec<SocketAddr> = Vec::new();
            for sa in resolved {
                if !policy.ip_permitted(sa.ip()) {
                    return Err(format!(
                        "egress blocked: host '{host}' resolves to disallowed address {}",
                        sa.ip()
                    )
                    .into());
                }
                out.push(sa);
            }
            if out.is_empty() {
                return Err(format!("egress blocked: host '{host}' did not resolve").into());
            }
            let addrs: Addrs = Box::new(out.into_iter());
            Ok(addrs)
        })
    }
}
