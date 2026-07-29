//! Outbound egress guarding for SSRF defense.
//!
//! This crate is the single source of truth for deciding whether the server may
//! make an HTTP request to a user-influenced URL. It provides three layers:
//!
//! 1. [`validate_url`] — synchronous URL-string validation (scheme allowlist,
//!    credential/fragment rejection, and classification of IP-literal hosts).
//!    This is mandatory because the connector never consults the DNS resolver
//!    for IP-literal hosts, so `http://127.0.0.1/` would otherwise bypass it.
//! 2. [`GuardedHttpClient`] / [`resolve_host`] — a [`reqwest::dns::Resolve`]
//!    implementation that resolves every host and rejects it if *any* resolved
//!    address is private/loopback/link-local/metadata. Because the connection
//!    only ever uses the vetted addresses, DNS rebinding is structurally
//!    defeated and every redirect hop is covered.
//! 3. A custom redirect policy that re-validates each hop's scheme/credentials
//!    and IP-literal host (see [`check_hop_url`]).
//!
//! The `client` feature (default-on) pulls in `reqwest`. Consumers that only
//! need validation + async resolution (e.g. the Chromium renderer) depend with
//! `default-features = false`.

mod error;
mod ip;
mod policy;
mod resolve;
mod url;

pub use error::EgressError;
pub use ip::is_blocked_ip;
pub use policy::{EgressPolicy, UrlRules};
pub use resolve::{resolve_and_validate, resolve_host};
pub use url::validate_url;

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
mod redirect;
#[cfg(feature = "client")]
mod resolver;

#[cfg(feature = "client")]
pub use client::{GuardedClientOptions, GuardedHttpClient, build_guarded_client};
#[cfg(feature = "client")]
pub use redirect::check_hop_url;
