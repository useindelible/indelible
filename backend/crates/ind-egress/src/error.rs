use std::net::IpAddr;

use thiserror::Error;

/// Reasons an outbound request may be refused before it leaves the process.
#[derive(Debug, Error)]
pub enum EgressError {
    #[error("must be a valid URL")]
    InvalidUrl,

    #[error("URL scheme '{scheme}' is not allowed")]
    DisallowedScheme { scheme: String },

    #[error("URL must use https")]
    SchemeRequiresHttps,

    #[error("URL must not include credentials")]
    CredentialsInUrl,

    #[error("URL must not include a fragment")]
    FragmentNotAllowed,

    #[error("URL must include a host")]
    MissingHost,

    #[error("host '{host}' is not allowed")]
    HostNotAllowed { host: String },

    #[error("URL resolves to a private or internal address")]
    PrivateAddress { host: String, ip: IpAddr },

    #[error("DNS resolution failed")]
    ResolutionFailed { host: String },

    #[error("too many redirects")]
    TooManyRedirects,

    #[error("request failed: {0}")]
    Transport(String),
}

impl EgressError {
    /// A safe, generic message suitable for returning to API clients. It never
    /// echoes the resolved IP or other internal-topology detail.
    pub fn client_message(&self) -> &'static str {
        match self {
            EgressError::InvalidUrl => "must be a valid URL",
            EgressError::DisallowedScheme { .. } => "URL scheme is not allowed",
            EgressError::SchemeRequiresHttps => "URL must use https",
            EgressError::CredentialsInUrl => "URL must not include credentials",
            EgressError::FragmentNotAllowed => "URL must not include a fragment",
            EgressError::MissingHost => "URL must include a host",
            EgressError::HostNotAllowed { .. } => "URL host is not allowed",
            EgressError::PrivateAddress { .. } => "URL resolves to a private or internal address",
            EgressError::ResolutionFailed { .. } => "could not resolve the URL host",
            EgressError::TooManyRedirects => "too many redirects",
            EgressError::Transport(_) => "outbound request failed",
        }
    }
}
