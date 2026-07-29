use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use futures::StreamExt;
use ind_application::repos::event::EventRepository;
use ind_domain::{DomainEventId, NewDomainEvent};
use ind_persistence::repos::PgEventRepository;
use ind_test_support::spawn_app;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn event_stream_drains_filtered_tenant_events_from_a_durable_cursor() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let stranger = app.create_web_session().await;
    let client = app.authed_client(&owner);
    let repo = PgEventRepository::new(app.pool().clone());
    let created_at = Utc::now() - ChronoDuration::seconds(2);

    let cursor = repo
        .append_event(NewDomainEvent {
            id: DomainEventId::new(),
            event_type: "collection.updated".into(),
            aggregate_type: "collection".into(),
            aggregate_id: uuid::Uuid::now_v7(),
            user_id: owner.user.id,
            payload: json!({"marker": "cursor"}),
            created_at,
        })
        .await
        .unwrap();
    repo.append_event(NewDomainEvent {
        id: DomainEventId::new(),
        event_type: "ai.output.failed".into(),
        aggregate_type: "collection".into(),
        aggregate_id: uuid::Uuid::now_v7(),
        user_id: stranger.user.id,
        payload: json!({"marker": "stranger"}),
        created_at,
    })
    .await
    .unwrap();
    let expected = repo
        .append_event(NewDomainEvent {
            id: DomainEventId::new(),
            event_type: "ai.output.completed".into(),
            aggregate_type: "collection".into(),
            aggregate_id: uuid::Uuid::now_v7(),
            user_id: owner.user.id,
            payload: json!({"marker": "owner"}),
            created_at,
        })
        .await
        .unwrap();

    assert_eq!(
        client
            .get("/api/v1/events/stream?event_type=unknown.event")
            .await
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        client
            .get("/api/v1/events/stream?event_type=collection.created&cursor=invalid")
            .await
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let response = client
        .get(&format!(
            "/api/v1/events/stream?event_type=ai.output.completed&event_type=ai.output.failed&cursor={}",
            cursor.id
        ))
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[reqwest::header::CONTENT_TYPE],
        "text/event-stream"
    );
    assert_eq!(
        response.headers()[reqwest::header::CACHE_CONTROL],
        "private, no-cache, no-store"
    );
    assert_eq!(response.headers()["x-accel-buffering"], "no");

    let mut stream = response.bytes_stream();
    let event = tokio::time::timeout(Duration::from_secs(3), async {
        let mut body = String::new();
        while let Some(chunk) = stream.next().await {
            body.push_str(std::str::from_utf8(&chunk.unwrap()).unwrap());
            if body.contains("\n\n") {
                return body;
            }
        }
        panic!("event stream closed before yielding an event")
    })
    .await
    .expect("event stream timed out");
    assert!(event.contains("event: domain_event"), "{event}");
    assert!(event.contains(&format!("id: {}", expected.id)), "{event}");
    assert!(event.contains("ai.output.completed"), "{event}");
    assert!(event.contains(r#""marker":"owner""#), "{event}");
    assert!(!event.contains("stranger"), "{event}");
}
