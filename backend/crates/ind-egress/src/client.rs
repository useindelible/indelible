use std::sync::Arc;
use std::time::Duration;

use reqwest::header::HeaderMap;
use reqwest::{Client, Method, RequestBuilder};

use crate::error::EgressError;
use crate::policy::{EgressPolicy, UrlRules};
use crate::redirect::guarded_redirect_policy;
use crate::resolver::GuardedDnsResolver;

/// Construction options for a [`GuardedHttpClient`].
#[derive(Debug, Clone)]
pub struct GuardedClientOptions {
    pub rules: UrlRules,
    pub policy: EgressPolicy,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub max_redirects: usize,
    pub user_agent: Option<String>,
    pub default_headers: HeaderMap,
    pub pool_idle_timeout: Option<Duration>,
    pub pool_max_idle_per_host: Option<usize>,
}

impl GuardedClientOptions {
    pub fn new(rules: UrlRules, policy: EgressPolicy) -> Self {
        Self {
            rules,
            policy,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            max_redirects: 5,
            user_agent: None,
            default_headers: HeaderMap::new(),
            pool_idle_timeout: None,
            pool_max_idle_per_host: None,
        }
    }

    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    pub fn max_redirects(mut self, value: usize) -> Self {
        self.max_redirects = value;
        self
    }

    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    pub fn default_headers(mut self, value: HeaderMap) -> Self {
        self.default_headers = value;
        self
    }

    pub fn pool_idle_timeout(mut self, value: Duration) -> Self {
        self.pool_idle_timeout = Some(value);
        self
    }

    pub fn pool_max_idle_per_host(mut self, value: usize) -> Self {
        self.pool_max_idle_per_host = Some(value);
        self
    }
}

/// A reqwest client whose DNS resolution, redirect following, and per-request
/// URL parsing are all forced through the egress guard. Obtain request builders
/// via [`GuardedHttpClient::get`] / [`post`](GuardedHttpClient::post) /
/// [`request`](GuardedHttpClient::request); each runs synchronous URL
/// validation (including IP-literal classification) before the request is built.
#[derive(Clone)]
pub struct GuardedHttpClient {
    client: Client,
    rules: UrlRules,
    policy: Arc<EgressPolicy>,
}

/// Build a [`GuardedHttpClient`] from options.
pub fn build_guarded_client(opts: GuardedClientOptions) -> Result<GuardedHttpClient, EgressError> {
    let policy = Arc::new(opts.policy);

    let mut builder = Client::builder()
        .connect_timeout(opts.connect_timeout)
        .timeout(opts.request_timeout)
        // Disable system/env proxies: a proxy would resolve the target host and
        // open the connection itself, bypassing GuardedDnsResolver entirely.
        .no_proxy()
        .dns_resolver(Arc::new(GuardedDnsResolver::new(policy.clone())))
        .redirect(guarded_redirect_policy(
            opts.rules,
            policy.clone(),
            opts.max_redirects,
        ));

    if let Some(ua) = opts.user_agent {
        builder = builder.user_agent(ua);
    }
    if !opts.default_headers.is_empty() {
        builder = builder.default_headers(opts.default_headers);
    }
    if let Some(idle) = opts.pool_idle_timeout {
        builder = builder.pool_idle_timeout(idle);
    }
    if let Some(max_idle) = opts.pool_max_idle_per_host {
        builder = builder.pool_max_idle_per_host(max_idle);
    }

    let client = builder
        .build()
        .map_err(|e| EgressError::Transport(e.to_string()))?;

    Ok(GuardedHttpClient {
        client,
        rules: opts.rules,
        policy,
    })
}

impl GuardedHttpClient {
    /// Validate `url` and return a [`RequestBuilder`] for `method`.
    pub fn request(&self, method: Method, url: &str) -> Result<RequestBuilder, EgressError> {
        let validated = crate::url::validate_url(url, &self.rules, &self.policy)?;
        Ok(self.client.request(method, validated))
    }

    pub fn get(&self, url: &str) -> Result<RequestBuilder, EgressError> {
        self.request(Method::GET, url)
    }

    pub fn post(&self, url: &str) -> Result<RequestBuilder, EgressError> {
        self.request(Method::POST, url)
    }

    /// The active policy (for surfaces that need to pre-validate a URL string
    /// without building a request).
    pub fn policy(&self) -> &EgressPolicy {
        &self.policy
    }

    /// The underlying reqwest client. Prefer the validating accessors above; use
    /// this only when a caller must reuse the configured connection pool with an
    /// already-validated [`reqwest::Url`].
    pub fn inner(&self) -> &Client {
        &self.client
    }
}
