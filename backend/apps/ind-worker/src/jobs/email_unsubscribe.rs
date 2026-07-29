use std::net::IpAddr;
use std::time::Duration;

use ind_application::AppError;
use ind_domain::ops::EmailUnsubscribeJob;
use ind_domain::{DomainError, EmailSenderId, GenericJobEnvelope};
use tracing::{info, warn};
use url::Url;

use crate::context::EmailJobDeps;

pub(crate) const ONE_CLICK_BODY: &str = "List-Unsubscribe=One-Click";
const ONE_CLICK_TIMEOUT_SECS: u64 = 10;
const ONE_CLICK_MAX_REDIRECTS: usize = 2;

/// Policy gates for the one-click URL validator.
///
/// Production wires `Self::strict()` (the `Default`). Loopback HTTP mock
/// servers used by integration tests construct a permissive variant.
#[derive(Debug, Clone, Copy)]
pub struct OneClickPolicy {
    pub allow_http: bool,
    pub allow_private_ips: bool,
}

impl OneClickPolicy {
    pub const fn strict() -> Self {
        Self {
            allow_http: false,
            allow_private_ips: false,
        }
    }
}

impl Default for OneClickPolicy {
    fn default() -> Self {
        Self::strict()
    }
}

/// Validate a sender-supplied one-click unsubscribe URL before we POST to it.
///
/// Defenses applied (in strict mode):
///   - RFC 8058 §3.1: the URL must be HTTPS.
///   - Reject IP literals that fall in private, loopback, or link-local
///     ranges (incl. AWS metadata service `169.254.169.254`).
///
/// Hostname-based SSRF (e.g. a public CNAME pointing into a private subnet)
/// requires a custom DNS resolver and is tracked as a follow-up.
pub fn validate_one_click_url(raw: &str, policy: OneClickPolicy) -> Result<Url, AppError> {
    let parsed = Url::parse(raw).map_err(|err| {
        AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid unsubscribe url: {err}"),
        })
    })?;

    if !policy.allow_http && parsed.scheme() != "https" {
        return Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!(
                "unsubscribe url must use https scheme, got: {}",
                parsed.scheme()
            ),
        }));
    }

    let ip = match parsed.host() {
        Some(url::Host::Ipv4(v4)) => Some(IpAddr::V4(v4)),
        Some(url::Host::Ipv6(v6)) => Some(IpAddr::V6(v6)),
        Some(url::Host::Domain(_)) => None,
        None => {
            return Err(AppError::Domain(DomainError::InvariantViolation {
                message: "unsubscribe url has no host".into(),
            }));
        }
    };

    if !policy.allow_private_ips
        && let Some(ip) = ip
        && ind_egress::is_blocked_ip(ip)
    {
        return Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("unsubscribe url targets a disallowed ip: {ip}"),
        }));
    }

    Ok(parsed)
}

pub async fn dispatch_generic_job(
    ctx: &EmailJobDeps,
    envelope: GenericJobEnvelope,
) -> Result<Option<()>, AppError> {
    match envelope.job_type.as_str() {
        "email.unsubscribe" => {
            let job: EmailUnsubscribeJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::Repository(Box::new(e)))?;
            handle_email_unsubscribe(ctx, job).await?;
            Ok(Some(()))
        }
        _ => Ok(None),
    }
}

pub async fn handle_email_unsubscribe(
    ctx: &EmailJobDeps,
    job: EmailUnsubscribeJob,
) -> Result<(), AppError> {
    let target_repo = ctx.email_unsubscribe_target_repo.as_ref().ok_or_else(|| {
        AppError::Domain(ind_domain::DomainError::InvariantViolation {
            message: "email unsubscribe target repo not configured".into(),
        })
    })?;

    let Some(targets) = target_repo.find_by_sender(job.sender_id).await? else {
        warn!(
            sender_id = %job.sender_id,
            "no unsubscribe targets recorded for sender; skipping"
        );
        return Ok(());
    };

    if let Some(url) = targets.one_click_post_url.as_deref() {
        execute_one_click_post(
            url,
            job.sender_id,
            ctx.email_unsubscribe_url_policy,
            ctx.egress_policy.clone(),
        )
        .await?;
        return Ok(());
    }

    if let Some(addr) = targets.mailto_addr.as_deref() {
        warn!(
            sender_id = %job.sender_id,
            mailto = %addr,
            "mailto unsubscribe not yet automated; local block still applied"
        );
        return Ok(());
    }

    if let Some(url) = targets.web_url.as_deref() {
        info!(
            sender_id = %job.sender_id,
            web_url = %url,
            "web-only unsubscribe target requires manual action; local block still applied"
        );
        return Ok(());
    }

    warn!(
        sender_id = %job.sender_id,
        "unsubscribe job dispatched but no actionable target found"
    );
    Ok(())
}

async fn execute_one_click_post(
    url: &str,
    sender_id: EmailSenderId,
    policy: OneClickPolicy,
    egress_policy: ind_egress::EgressPolicy,
) -> Result<(), AppError> {
    // RFC 8058 https enforcement (+ IP-literal defense) stays here; the guarded
    // client adds resolved-IP SSRF blocking and redirect re-validation, closing
    // the hostname-rebinding hole for sender-controlled List-Unsubscribe URLs.
    let validated = validate_one_click_url(url, policy)?;

    let client = ind_egress::build_guarded_client(
        ind_egress::GuardedClientOptions::new(ind_egress::UrlRules::ingest(), egress_policy)
            .request_timeout(Duration::from_secs(ONE_CLICK_TIMEOUT_SECS))
            .max_redirects(ONE_CLICK_MAX_REDIRECTS),
    )
    .map_err(|e| AppError::ExternalService {
        service: "email_unsubscribe".into(),
        message: format!("failed to build http client: {}", e.client_message()),
    })?;

    let response = client
        .post(validated.as_str())
        .map_err(|e| {
            AppError::Domain(ind_domain::DomainError::Validation {
                field: "unsubscribe_url".into(),
                message: format!("unsubscribe target blocked: {}", e.client_message()),
            })
        })?
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(ONE_CLICK_BODY)
        .send()
        .await
        .map_err(|e| AppError::ExternalService {
            service: "email_unsubscribe".into(),
            message: format!("one-click POST failed: {e}"),
        })?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::ExternalService {
            service: "email_unsubscribe".into(),
            message: format!("one-click POST returned non-2xx status: {status}"),
        });
    }

    info!(
        sender_id = %sender_id,
        host = %validated.host_str().unwrap_or(""),
        status = %status,
        "one-click unsubscribe POST completed"
    );
    Ok(())
}
