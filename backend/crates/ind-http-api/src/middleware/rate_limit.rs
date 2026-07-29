use std::collections::HashMap;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use tokio::sync::Mutex;

use super::ip_extract::{TrustedProxies, extract_client_ip};

const X_RATE_LIMIT_LIMIT: &str = "x-ratelimit-limit";
const X_RATE_LIMIT_REMAINING: &str = "x-ratelimit-remaining";
const X_RATE_LIMIT_RESET: &str = "x-ratelimit-reset";

#[derive(Debug, Clone, Copy)]
pub struct RateLimitRule {
    pub limit: NonZeroU32,
    pub window: Duration,
}

impl RateLimitRule {
    #[expect(
        clippy::expect_used,
        reason = "boot-time rate-limit construction; limit comes from startup config with non-zero defaults and is never built on the request path"
    )]
    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            limit: NonZeroU32::new(limit).expect("rate limit must be non-zero"),
            window,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub login: RateLimitRule,
    pub registration: RateLimitRule,
    pub password_reset: RateLimitRule,
    pub user_api: RateLimitRule,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            login: RateLimitRule::new(5, Duration::from_secs(30)),
            registration: RateLimitRule::new(3, Duration::from_secs(60)),
            password_reset: RateLimitRule::new(3, Duration::from_secs(15 * 60)),
            user_api: RateLimitRule::new(USER_API_LIMIT, USER_API_WINDOW),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BucketState {
    started_at_system: SystemTime,
    started_at_instant: Instant,
    used: u32,
}

impl BucketState {
    fn new(now_system: SystemTime, now_instant: Instant) -> Self {
        Self {
            started_at_system: now_system,
            started_at_instant: now_instant,
            used: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RateLimitDecision {
    limit: u32,
    remaining: u32,
    reset_epoch_secs: u64,
    retry_after_secs: Option<u64>,
}

/// Hard ceiling on tracked buckets per limiter. Keys for per-account / per-user
/// limiters are attacker-influenced (email, user id), and IPv6 sources are
/// effectively unbounded, so without a cap the bucket map is a memory-exhaustion
/// vector. Stale buckets are pruned every window; if a flood fills the map with
/// still-active keys, the limiter fails CLOSED for unseen keys (denies without
/// inserting) so existing throttle state is never reset (the upstream per-IP
/// limit still applies and buckets rebuild after a window).
const MAX_TRACKED_BUCKETS: usize = 100_000;

#[derive(Debug)]
struct LimiterState<K: Eq + std::hash::Hash> {
    buckets: HashMap<K, BucketState>,
    last_prune: Instant,
}

#[derive(Debug)]
struct EndpointLimiter<K: Eq + std::hash::Hash = IpAddr> {
    rule: RateLimitRule,
    max_buckets: usize,
    state: Mutex<LimiterState<K>>,
}

impl<K: Eq + std::hash::Hash> EndpointLimiter<K> {
    fn new(rule: RateLimitRule) -> Self {
        Self::with_max_buckets(rule, MAX_TRACKED_BUCKETS)
    }

    fn with_max_buckets(rule: RateLimitRule, max_buckets: usize) -> Self {
        Self {
            rule,
            max_buckets,
            state: Mutex::new(LimiterState {
                buckets: HashMap::new(),
                last_prune: Instant::now(),
            }),
        }
    }

    async fn evaluate(&self, key: K) -> RateLimitDecision {
        let now_system = SystemTime::now();
        let now_instant = Instant::now();
        let window = self.rule.window;
        let limit = self.rule.limit.get();
        let mut state = self.state.lock().await;

        // Periodic GC: drop buckets whose window has fully elapsed (they would be
        // reset on next access anyway) so high-cardinality keys can't grow the
        // map without bound. Runs at most once per window.
        if now_instant.duration_since(state.last_prune) >= window {
            state
                .buckets
                .retain(|_, b| now_instant.duration_since(b.started_at_instant) < window);
            state.last_prune = now_instant;
        }

        // Hard-cap backstop: once the map is full of still-active keys, FAIL
        // CLOSED for unseen keys (deny without inserting) rather than evicting or
        // clearing existing buckets. Clearing would let a high-cardinality flood
        // predictably reset everyone's throttle state; denying unseen keys keeps
        // existing limits intact. Reached only under a genuine flood.
        if state.buckets.len() >= self.max_buckets && !state.buckets.contains_key(&key) {
            tracing::warn!(
                tracked = state.buckets.len(),
                "rate-limit bucket cap reached; denying unseen key"
            );
            let reset_time = now_system.checked_add(window).unwrap_or(now_system);
            return RateLimitDecision {
                limit,
                remaining: 0,
                reset_epoch_secs: system_time_to_epoch_secs(reset_time),
                retry_after_secs: Some(window.as_secs().max(1)),
            };
        }

        let bucket = state
            .buckets
            .entry(key)
            .or_insert_with(|| BucketState::new(now_system, now_instant));

        if now_instant.duration_since(bucket.started_at_instant) >= self.rule.window {
            *bucket = BucketState::new(now_system, now_instant);
        }

        let reset_time = bucket
            .started_at_system
            .checked_add(self.rule.window)
            .unwrap_or(bucket.started_at_system);
        let reset_epoch_secs = system_time_to_epoch_secs(reset_time);

        if bucket.used < limit {
            bucket.used += 1;
            let remaining = limit - bucket.used;
            RateLimitDecision {
                limit,
                remaining,
                reset_epoch_secs,
                retry_after_secs: None,
            }
        } else {
            let elapsed = now_instant.duration_since(bucket.started_at_instant);
            let retry_after = self.rule.window.saturating_sub(elapsed).as_secs().max(1);
            RateLimitDecision {
                limit,
                remaining: 0,
                reset_epoch_secs,
                retry_after_secs: Some(retry_after),
            }
        }
    }
}

fn system_time_to_epoch_secs(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// Default per-user ceiling on authenticated API requests. Generous enough that
// bursty UIs (parallel requests on page load, batch saves) are unaffected, but a
// runaway loop or scripted abuse from one account is bounded — covering general
// API hammering (L.1), save floods (L.3), and upload floods (L.5). In-memory
// and per-instance: in multi-instance hosted deployments the effective ceiling
// is this value times the instance count, which is acceptable as an abuse stop.
// Operators override via USER_API_RATE_LIMIT_REQUESTS / _WINDOW_SECS.
const USER_API_LIMIT: u32 = 1000;
const USER_API_WINDOW: Duration = Duration::from_secs(60);

/// Per-user authenticated request limiter, checked from the auth extractor so it
/// covers every authenticated route. Keyed by user id.
#[derive(Clone)]
pub struct UserRateLimiter {
    inner: Arc<EndpointLimiter<String>>,
}

impl UserRateLimiter {
    pub fn new(rule: RateLimitRule) -> Self {
        Self {
            inner: Arc::new(EndpointLimiter::new(rule)),
        }
    }

    /// `Err(retry_after_secs)` when the user is over their ceiling.
    pub async fn check(&self, user_id: &str) -> Result<(), u64> {
        match self
            .inner
            .evaluate(user_id.to_owned())
            .await
            .retry_after_secs
        {
            Some(retry) => Err(retry),
            None => Ok(()),
        }
    }
}

impl Default for UserRateLimiter {
    fn default() -> Self {
        Self::new(RateLimitRule::new(USER_API_LIMIT, USER_API_WINDOW))
    }
}

// Per-account (per-email) throttles defeat credential-stuffing that rotates
// source IPs against a single account. Thresholds are intentionally generous so
// normal mistyping isn't disruptive, while a distributed brute force still hits
// the per-account ceiling. Not operator-tunable for v1.
const LOGIN_PER_ACCOUNT_LIMIT: u32 = 10;
const LOGIN_PER_ACCOUNT_WINDOW: Duration = Duration::from_secs(15 * 60);
const FORGOT_PER_ACCOUNT_LIMIT: u32 = 5;
const FORGOT_PER_ACCOUNT_WINDOW: Duration = Duration::from_secs(60 * 60);

/// Max auth request body we will buffer to extract the account key. Login /
/// forgot-password payloads are tiny; anything larger is rejected downstream.
const MAX_AUTH_BODY_BYTES: usize = 64 * 1024;

#[derive(Clone)]
pub struct RateLimiters {
    login: Arc<EndpointLimiter>,
    registration: Arc<EndpointLimiter>,
    password_reset: Arc<EndpointLimiter>,
    login_account: Arc<EndpointLimiter<String>>,
    password_reset_account: Arc<EndpointLimiter<String>>,
    trusted_proxies: TrustedProxies,
}

impl RateLimiters {
    pub fn new(config: RateLimitConfig, trusted_proxies: TrustedProxies) -> Self {
        Self {
            login: Arc::new(EndpointLimiter::new(config.login)),
            registration: Arc::new(EndpointLimiter::new(config.registration)),
            password_reset: Arc::new(EndpointLimiter::new(config.password_reset)),
            login_account: Arc::new(EndpointLimiter::new(RateLimitRule::new(
                LOGIN_PER_ACCOUNT_LIMIT,
                LOGIN_PER_ACCOUNT_WINDOW,
            ))),
            password_reset_account: Arc::new(EndpointLimiter::new(RateLimitRule::new(
                FORGOT_PER_ACCOUNT_LIMIT,
                FORGOT_PER_ACCOUNT_WINDOW,
            ))),
            trusted_proxies,
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "header values are decimal string renderings of integer counters, which are always valid header bytes"
)]
fn apply_rate_limit_headers(headers: &mut http::HeaderMap, decision: &RateLimitDecision) {
    headers.insert(
        http::header::HeaderName::from_static(X_RATE_LIMIT_LIMIT),
        http::HeaderValue::from_str(&decision.limit.to_string()).expect("valid limit header"),
    );
    headers.insert(
        http::header::HeaderName::from_static(X_RATE_LIMIT_REMAINING),
        http::HeaderValue::from_str(&decision.remaining.to_string())
            .expect("valid remaining header"),
    );
    headers.insert(
        http::header::HeaderName::from_static(X_RATE_LIMIT_RESET),
        http::HeaderValue::from_str(&decision.reset_epoch_secs.to_string())
            .expect("valid reset header"),
    );
}

#[expect(
    clippy::expect_used,
    reason = "retry-after header value is a decimal string rendering of an integer, which is always valid header bytes"
)]
fn rate_limit_response(decision: &RateLimitDecision) -> Response {
    let body = serde_json::json!({
        "error": {
            "code": "rate_limited",
            "message": format!("Too many attempts. Please try again in {} seconds.", decision.retry_after_secs.unwrap_or(1)),
            "retryAfter": decision.retry_after_secs.unwrap_or(1)
        }
    });

    let mut response = (StatusCode::TOO_MANY_REQUESTS, body.to_string()).into_response();
    response.headers_mut().insert(
        http::header::RETRY_AFTER,
        http::HeaderValue::from_str(&decision.retry_after_secs.unwrap_or(1).to_string())
            .expect("valid retry-after header"),
    );
    response.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/json"),
    );
    apply_rate_limit_headers(response.headers_mut(), decision);
    response
}

pub async fn login_rate_limit(
    axum::extract::State(limiters): axum::extract::State<RateLimiters>,
    req: Request<Body>,
    next: Next,
) -> Response {
    check_rate_limit(&limiters.login, &limiters.trusted_proxies, req, next).await
}

pub async fn registration_rate_limit(
    axum::extract::State(limiters): axum::extract::State<RateLimiters>,
    req: Request<Body>,
    next: Next,
) -> Response {
    check_rate_limit(&limiters.registration, &limiters.trusted_proxies, req, next).await
}

pub async fn password_reset_rate_limit(
    axum::extract::State(limiters): axum::extract::State<RateLimiters>,
    req: Request<Body>,
    next: Next,
) -> Response {
    check_rate_limit(
        &limiters.password_reset,
        &limiters.trusted_proxies,
        req,
        next,
    )
    .await
}

/// Per-account login throttle. Buffers the body to read the `email`, then keys
/// the limiter on the normalized address so IP rotation can't defeat it.
pub async fn login_account_rate_limit(
    axum::extract::State(limiters): axum::extract::State<RateLimiters>,
    req: Request<Body>,
    next: Next,
) -> Response {
    account_rate_limit(&limiters.login_account, req, next).await
}

/// Per-account forgot-password throttle (limits reset-email spam to one account).
pub async fn password_reset_account_rate_limit(
    axum::extract::State(limiters): axum::extract::State<RateLimiters>,
    req: Request<Body>,
    next: Next,
) -> Response {
    account_rate_limit(&limiters.password_reset_account, req, next).await
}

async fn account_rate_limit(
    limiter: &EndpointLimiter<String>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let (parts, body) = req.into_parts();
    let bytes = match axum::body::to_bytes(body, MAX_AUTH_BODY_BYTES).await {
        Ok(bytes) => bytes,
        // Unbufferable/oversized body: let the handler's own limits reject it.
        Err(_) => return next.run(Request::from_parts(parts, Body::empty())).await,
    };

    let account_key = serde_json::from_slice::<serde_json::Value>(&bytes)
        .ok()
        .and_then(|v| {
            v.get("email")
                .and_then(|e| e.as_str())
                .map(|e| e.trim().to_ascii_lowercase())
        })
        .filter(|e| !e.is_empty());

    let rebuilt = Request::from_parts(parts, Body::from(bytes));

    if let Some(key) = account_key {
        let decision = limiter.evaluate(key).await;
        if decision.retry_after_secs.is_some() {
            tracing::warn!(
                retry_after_secs = decision.retry_after_secs.unwrap_or(1),
                "per-account auth rate limit exceeded"
            );
            return rate_limit_response(&decision);
        }
    }

    next.run(rebuilt).await
}

async fn check_rate_limit(
    limiter: &EndpointLimiter,
    trusted_proxies: &TrustedProxies,
    req: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_client_ip(&req, trusted_proxies);
    let decision = limiter.evaluate(ip).await;

    if decision.retry_after_secs.is_some() {
        tracing::warn!(
            ip = %ip,
            retry_after_secs = decision.retry_after_secs.unwrap_or(1),
            "rate limit exceeded"
        );
        return rate_limit_response(&decision);
    }

    let mut response = next.run(req).await;
    apply_rate_limit_headers(response.headers_mut(), &decision);
    response
}

#[cfg(test)]
mod tests;
