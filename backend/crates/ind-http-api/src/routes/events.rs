use std::collections::VecDeque;
use std::convert::Infallible;
use std::str::FromStr;
use std::time::Duration;

use axum::Router;
use axum::extract::{RawQuery, State};
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use futures::stream;
use http::HeaderMap;
use ind_application::repos::event::EventRepository;
use ind_application::webhooks::is_known_webhook_event;
use ind_domain::{DomainEvent, DomainEventId, UserId};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::error::{ApiError, FieldError};
use crate::middleware::AccountAccess;
use crate::realtime::RealtimeSubscription;
use crate::state::AppState;

const DRAIN_LIMIT: i64 = 100;
const FALLBACK_POLL_SECONDS: u64 = 30;
const READ_SAFETY_WINDOW_MS: i64 = 500;
const REALTIME_ONLY_EVENT_TYPES: &[&str] = &["ai.output.completed", "ai.output.failed"];

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct EventStreamParams {
    pub cursor: Option<String>,
    #[serde(default)]
    pub event_type: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RealtimeEventPayloadSchema {
    pub document_id: Option<String>,
    pub highlight_id: Option<String>,
    pub changed: Option<Vec<String>>,
    pub triage_state: Option<String>,
    pub is_favorite: Option<bool>,
    pub is_shortlisted: Option<bool>,
    pub tag_ids: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub source: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RealtimeEventResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = RealtimeEventPayloadSchema)]
    pub payload: serde_json::Value,
}

impl From<DomainEvent> for RealtimeEventResponse {
    fn from(event: DomainEvent) -> Self {
        Self {
            id: event.id.to_string(),
            event_type: event.event_type,
            aggregate_type: event.aggregate_type,
            aggregate_id: event.aggregate_id.to_string(),
            created_at: event.created_at,
            payload: event.payload,
        }
    }
}

pub fn event_routes() -> Router<AppState> {
    Router::new().route("/api/v1/events/stream", get(stream_events))
}

#[utoipa::path(
    get,
    path = "/api/v1/events/stream",
    params(EventStreamParams),
    responses(
        (status = 200, description = "User-scoped stream of domain events", content_type = "text/event-stream", body = RealtimeEventResponse),
        (status = 401, description = "Authentication required"),
        (status = 429, description = "Too many open event streams"),
        (status = 422, description = "Invalid cursor"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Events",
)]
pub async fn stream_events(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let params = parse_event_stream_params(raw_query.as_deref());
    let cursor = parse_cursor(&headers, params.cursor.as_deref())?;
    let event_types = parse_event_types(&params.event_type)?;
    let subscription = state
        .realtime_hub
        .subscribe(auth_user.user_id)
        .map_err(|_| ApiError::RateLimited)?;
    let event_repo = state
        .event_repo
        .as_ref()
        .cloned()
        .ok_or_else(|| ApiError::Internal {
            message: "event repository not configured".to_string(),
        })?;

    let last_emitted_event_id = match cursor {
        Some(cursor) => Some(cursor),
        None => current_tail(event_repo.as_ref(), auth_user.user_id, &event_types).await?,
    };

    let stream = event_stream(
        event_repo,
        auth_user.user_id,
        event_types,
        subscription,
        last_emitted_event_id,
    );
    let sse = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(25))
            .text("keep-alive"),
    );

    Ok((crate::routes::sse::stream_headers(), sse))
}

fn event_stream(
    event_repo: std::sync::Arc<dyn EventRepository>,
    user_id: UserId,
    event_types: Vec<String>,
    subscription: RealtimeSubscription,
    last_emitted_event_id: Option<DomainEventId>,
) -> impl futures::Stream<Item = Result<Event, Infallible>> {
    struct State {
        event_repo: std::sync::Arc<dyn EventRepository>,
        user_id: UserId,
        event_types: Vec<String>,
        subscription: RealtimeSubscription,
        last_emitted_event_id: Option<DomainEventId>,
        pending: VecDeque<DomainEvent>,
        fallback_poll_interval: Duration,
    }

    let fallback_jitter = Duration::from_millis(rand::random::<u64>() % 5_000);
    stream::unfold(
        State {
            event_repo,
            user_id,
            event_types,
            subscription,
            last_emitted_event_id,
            pending: VecDeque::new(),
            fallback_poll_interval: Duration::from_secs(FALLBACK_POLL_SECONDS) + fallback_jitter,
        },
        |mut state| async move {
            loop {
                if let Some(event) = state.pending.pop_front() {
                    state.last_emitted_event_id = Some(event.id);
                    let response = RealtimeEventResponse::from(event);
                    let data =
                        serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
                    let event = Event::default()
                        .event("domain_event")
                        .id(response.id)
                        .data(data);
                    return Some((Ok(event), state));
                }

                match drain_events(
                    state.event_repo.as_ref(),
                    state.user_id,
                    state.last_emitted_event_id,
                    &state.event_types,
                )
                .await
                {
                    Ok(events) if !events.is_empty() => {
                        state.pending = events.into();
                        continue;
                    }
                    Ok(_) => {}
                    Err(err) => {
                        tracing::warn!(error = %err, "failed to drain domain events for SSE");
                    }
                }

                tokio::select! {
                    notification = state.subscription.recv() => {
                        match notification {
                            Ok(notification) => {
                                if state
                                    .last_emitted_event_id
                                    .is_some_and(|last| notification.event_id.as_uuid() <= last.as_uuid())
                                {
                                    continue;
                                }
                                // Let the visibility window elapse before draining so UUIDv7 cursor order
                                // does not skip older events that commit just after this notification.
                                tokio::time::sleep(Duration::from_millis(READ_SAFETY_WINDOW_MS as u64)).await;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                    _ = tokio::time::sleep(state.fallback_poll_interval) => {}
                }
            }
        },
    )
}

fn parse_event_stream_params(raw_query: Option<&str>) -> EventStreamParams {
    let mut params = EventStreamParams {
        cursor: None,
        event_type: Vec::new(),
    };

    let Some(raw_query) = raw_query else {
        return params;
    };

    for (key, value) in url::form_urlencoded::parse(raw_query.as_bytes()) {
        match key.as_ref() {
            "cursor" => params.cursor = Some(value.into_owned()),
            "event_type" => params.event_type.push(value.into_owned()),
            _ => {}
        }
    }

    params
}

fn parse_event_types(raw_event_types: &[String]) -> Result<Vec<String>, ApiError> {
    let mut event_types = Vec::new();

    for raw in raw_event_types {
        let event_type = raw.trim();
        if event_type.is_empty() {
            return Err(invalid_event_type("must not be empty"));
        }
        if !is_known_webhook_event(event_type) && !REALTIME_ONLY_EVENT_TYPES.contains(&event_type) {
            return Err(invalid_event_type("is not a supported domain event type"));
        }
        if !event_types.iter().any(|known| known == event_type) {
            event_types.push(event_type.to_string());
        }
    }

    if event_types.is_empty() {
        return Err(invalid_event_type(
            "at least one event_type query parameter is required",
        ));
    }

    Ok(event_types)
}

fn invalid_event_type(message: impl Into<String>) -> ApiError {
    ApiError::ValidationError {
        errors: vec![FieldError {
            field: "event_type".into(),
            message: message.into(),
        }],
    }
}

fn parse_cursor(
    headers: &HeaderMap,
    query_cursor: Option<&str>,
) -> Result<Option<DomainEventId>, ApiError> {
    let raw = headers
        .get("last-event-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .or(query_cursor);

    raw.map(|cursor| {
        DomainEventId::from_str(cursor).map_err(|_| ApiError::ValidationError {
            errors: vec![FieldError {
                field: "cursor".into(),
                message: "must be a valid UUIDv7 domain event ID".into(),
            }],
        })
    })
    .transpose()
}

async fn current_tail(
    event_repo: &dyn EventRepository,
    user_id: UserId,
    event_types: &[String],
) -> Result<Option<DomainEventId>, ApiError> {
    let visible_before = Utc::now() - ChronoDuration::milliseconds(READ_SAFETY_WINDOW_MS);
    event_repo
        .current_tail(user_id, visible_before, event_types)
        .await
        .map_err(ApiError::from)
}

async fn drain_events(
    event_repo: &dyn EventRepository,
    user_id: UserId,
    cursor: Option<DomainEventId>,
    event_types: &[String],
) -> Result<Vec<DomainEvent>, ApiError> {
    let visible_before = Utc::now() - ChronoDuration::milliseconds(READ_SAFETY_WINDOW_MS);
    event_repo
        .drain_events_after(user_id, cursor, visible_before, event_types, DRAIN_LIMIT)
        .await
        .map_err(ApiError::from)
}
