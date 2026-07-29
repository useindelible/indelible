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
            None => DeliveryHistoryTick::Failed,
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
    let is_delivered = matches!(delivery.status_code, Some(200..=299));
    let outcome = if is_delivered {
        WebhookDeliveryOutcome::Delivered
    } else {
        WebhookDeliveryOutcome::Failed
    };
    let response_detail = delivery
        .response_body
        .as_deref()
        .map(str::trim)
        .filter(|body| !body.is_empty());
    let error = if is_delivered {
        None
    } else {
        Some(match delivery.status_code {
            Some(status) => response_detail.map_or_else(
                || format!("HTTP {status}"),
                |detail| format!("HTTP {status}: {detail}"),
            ),
            None => response_detail
                .unwrap_or("Webhook delivery failed before receiving an HTTP response")
                .to_string(),
        })
    };

    WebhookDeliveryResponse {
        id: delivery.id.to_string(),
        endpoint_id: delivery.endpoint_id.to_string(),
        event: delivery.event_type,
        target: target.into(),
        outcome,
        error,
        status_code: delivery.status_code,
        attempted_at: delivery.created_at,
        delivered_at: delivery.delivered_at,
        latency_ms: delivery.latency_ms,
        attempt: delivery.attempt_number,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use ind_domain::{
        DomainEventId, UserId, WebhookDeliveryId, WebhookDispatchId, WebhookEndpointId,
    };
    use serde_json::json;

    use super::*;

    fn delivery(
        status_code: Option<i32>,
        response_body: Option<&str>,
        delivered_at: Option<DateTime<Utc>>,
    ) -> WebhookDelivery {
        WebhookDelivery {
            id: WebhookDeliveryId::new(),
            dispatch_id: WebhookDispatchId::new(),
            domain_event_id: DomainEventId::new(),
            endpoint_id: WebhookEndpointId::new(),
            event_type: "library_entry.saved".into(),
            payload: json!({"entry_id": "ent_test"}),
            status_code,
            response_body: response_body.map(str::to_owned),
            attempt_number: 1,
            latency_ms: Some(42),
            delivered_at,
            next_retry_at: None,
            created_at: Utc.with_ymd_and_hms(2026, 8, 11, 12, 0, 0).unwrap(),
        }
    }

    fn endpoint(id: WebhookEndpointId) -> WebhookEndpoint {
        let created_at = Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap();
        WebhookEndpoint {
            id,
            user_id: UserId::new(),
            name: "Automation".into(),
            url: "https://example.com/hook".into(),
            secret_hash: "hash".into(),
            secret_ciphertext: None,
            secret_preview: "whsec_test...".into(),
            events: vec!["library_entry.saved".into()],
            is_active: true,
            created_at,
            updated_at: created_at,
        }
    }

    #[test]
    fn delivery_response_reports_confirmed_success_without_an_error() {
        let delivered_at = Utc.with_ymd_and_hms(2026, 8, 11, 12, 0, 1).unwrap();

        let response = serde_json::to_value(delivery_response(
            delivery(Some(204), Some("accepted"), Some(delivered_at)),
            "webhook endpoint",
        ))
        .unwrap();

        assert_eq!(response["outcome"], "delivered");
        assert_eq!(response["error"], serde_json::Value::Null);
        assert_eq!(response["attempted_at"], "2026-08-11T12:00:00Z");
        assert_eq!(response["delivered_at"], "2026-08-11T12:00:01Z");
    }

    #[test]
    fn delivery_response_reports_non_success_http_response_as_failed() {
        let response = serde_json::to_value(delivery_response(
            delivery(Some(503), Some("receiver unavailable"), None),
            "webhook endpoint",
        ))
        .unwrap();

        assert_eq!(response["outcome"], "failed");
        assert_eq!(response["delivered_at"], serde_json::Value::Null);
        assert_eq!(response["error"], "HTTP 503: receiver unavailable");
    }

    #[test]
    fn delivery_response_reports_pre_response_failure_reason() {
        let response = serde_json::to_value(delivery_response(
            delivery(None, Some("delivery blocked: private network target"), None),
            "webhook endpoint",
        ))
        .unwrap();

        assert_eq!(response["outcome"], "failed");
        assert_eq!(response["delivered_at"], serde_json::Value::Null);
        assert_eq!(
            response["error"],
            "delivery blocked: private network target"
        );
    }

    #[test]
    fn endpoint_history_distinguishes_failed_attempts_from_empty_padding() {
        let delivery = delivery(None, Some("connection refused"), None);
        let response = serde_json::to_value(endpoint_response(
            endpoint(delivery.endpoint_id),
            &[delivery],
        ))
        .unwrap();
        let history = response["delivery_history"].as_array().unwrap();

        assert_eq!(history[0], "failed");
        assert_eq!(history.len(), 8);
        assert!(history[1..].iter().all(|tick| tick == "pending"));
    }
}
