use super::*;

pub(super) fn default_webhook_active() -> bool {
    true
}

pub(super) fn service(
    state: &AppState,
) -> Result<&std::sync::Arc<dyn WebhookOperations>, ApiError> {
    state
        .webhook_ops
        .as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable {
            message: "webhook operations are not configured".into(),
        })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeliveryClaimOutcome {
    Claimed,
    Duplicate,
    Ignored,
    RetryableFailure,
}

pub(super) fn status_for_claim_outcomes(
    outcomes: impl IntoIterator<Item = DeliveryClaimOutcome>,
) -> StatusCode {
    if outcomes
        .into_iter()
        .any(|outcome| matches!(outcome, DeliveryClaimOutcome::RetryableFailure))
    {
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    }
}

pub(super) fn parse_webhook_id(raw: &str) -> Result<WebhookEndpointId, ApiError> {
    raw.parse().map_err(|_| ApiError::NotFound {
        entity: "webhook",
        id: raw.to_string(),
    })
}

pub(super) fn validate_name(name: Option<&str>, fallback_url: &str) -> Result<String, ApiError> {
    let value = name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback_url)
        .trim()
        .to_string();
    if value.is_empty() {
        return Err(validation("name", "name is required"));
    }
    if value.chars().count() > 100 {
        return Err(validation("name", "name must be 100 characters or fewer"));
    }
    Ok(value)
}

pub(super) fn validate_events(events: Vec<String>) -> Result<Vec<String>, ApiError> {
    if events.is_empty() {
        return Err(validation("events", "must include at least one event"));
    }
    let mut out = Vec::with_capacity(events.len());
    for event in events {
        let trimmed = event.trim();
        if trimmed.is_empty() {
            return Err(validation("events", "event names cannot be blank"));
        }
        if !is_known_webhook_event(trimmed) {
            return Err(validation(
                "events",
                &format!("unsupported webhook event: {trimmed}"),
            ));
        }
        if !out.iter().any(|existing| existing == trimmed) {
            out.push(trimmed.to_string());
        }
    }
    Ok(out)
}

pub(super) fn validate_webhook_url(
    raw: &str,
    allow_private_targets: bool,
) -> Result<String, ApiError> {
    // Delegate scheme/credential/fragment checks and (bracket-safe) IPv6 host
    // classification to the shared egress validator — the single source of truth
    // for SSRF address ranges. This is creation-time validation; delivery
    // re-validates against the resolved IP (see `webhook_delivery`).
    let policy = ind_egress::EgressPolicy {
        allow_private_targets,
        extra_allowed_ips: Vec::new(),
    };
    ind_egress::validate_url(raw, &ind_egress::UrlRules::webhook(), &policy)
        .map(|url| url.to_string())
        .map_err(|e| validation("url", e.client_message()))
}

fn validation(field: &str, message: &str) -> ApiError {
    ApiError::ValidationError {
        errors: vec![FieldError {
            field: field.into(),
            message: message.into(),
        }],
    }
}

pub(super) fn endpoint_response(
    endpoint: WebhookEndpoint,
    deliveries: &[WebhookDelivery],
) -> WebhookEndpointResponse {
    let mut history = deliveries
        .iter()
        .take(8)
        .map(|d| match d.status_code {
            Some(200..=299) => DeliveryHistoryTick::S2xx,
            Some(400..=499) => DeliveryHistoryTick::S4xx,
            Some(500..=599) => DeliveryHistoryTick::S5xx,
            Some(_) => DeliveryHistoryTick::S5xx,
            None => DeliveryHistoryTick::Pending,
        })
        .collect::<Vec<_>>();
    while history.len() < 8 {
        history.push(DeliveryHistoryTick::Pending);
    }

    let last_status = if !endpoint.is_active {
        WebhookEndpointStatus::Paused
    } else if deliveries
        .iter()
        .take(3)
        .any(|d| !matches!(d.status_code, Some(200..=299)))
    {
        WebhookEndpointStatus::Failing
    } else {
        WebhookEndpointStatus::Healthy
    };

    WebhookEndpointResponse {
        id: endpoint.id.to_string(),
        name: endpoint.name,
        url: endpoint.url,
        events: endpoint.events,
        is_active: endpoint.is_active,
        secret_preview: endpoint.secret_preview,
        created_at: endpoint.created_at,
        updated_at: endpoint.updated_at,
        last_status,
        delivery_history: history,
    }
}

pub(super) fn delivery_response(
    delivery: WebhookDelivery,
    target: impl Into<String>,
) -> WebhookDeliveryResponse {
    WebhookDeliveryResponse {
        id: delivery.id.to_string(),
        endpoint_id: delivery.endpoint_id.to_string(),
        event: delivery.event_type,
        target: target.into(),
        status_code: delivery.status_code,
        delivered_at: delivery.delivered_at.unwrap_or(delivery.created_at),
        latency_ms: delivery.latency_ms,
        attempt: delivery.attempt_number,
    }
}
