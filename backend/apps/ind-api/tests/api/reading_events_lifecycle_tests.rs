use ind_domain::DocumentType;
use ind_test_support::{AuthedClient, DocumentFactory, spawn_app};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::{assert_json_response, assert_status};

const CLIENT_A: &str = "cli_018f5b1e-0000-7000-8000-00000000000a";

fn rev(n: u32) -> String {
    format!("rev_{}", uuid::Uuid::from_u128(u128::from(n)))
}

fn event(id: &str, seq: i64, percent: i32, page: i32, recorded_at: &str) -> Value {
    json!({
        "id": id, "origin_seq": seq, "kind": "progress", "progress_basis_points": percent * 100,
        "position": {"anchor": {"type": "page", "page": page}, "fraction": 0.25},
        "active_ms": 12_000, "recorded_at": recorded_at
    })
}

async fn append(
    client: &AuthedClient<'_>,
    doc: &str,
    device: &str,
    events: Vec<Value>,
) -> reqwest::Response {
    client
        .post_json(
            &format!("/api/v1/documents/{doc}/reading-events"),
            &json!({"client_id": device, "events": events}),
        )
        .await
}

async fn reader(client: &AuthedClient<'_>, doc: &str) -> Value {
    assert_json_response(
        client.get(&format!("/api/v1/documents/{doc}")).await,
        StatusCode::OK,
    )
    .await
}

/// Lifecycle outranks arithmetic: a reader who stops at 90% because the rest is appendices has
/// finished the book, and must not have to write a false 100 into the log to say so.
#[tokio::test]
async fn finishing_below_full_progress_is_accepted_and_latches() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let doc = DocumentFactory::new(session.user.id)
        .with_document_type(DocumentType::Book)
        .insert(app.pool())
        .await
        .id
        .to_string();

    assert_status(
        append(
            &client,
            &doc,
            CLIENT_A,
            vec![json!({
                "id": rev(50), "origin_seq": 1, "kind": "finished",
                "progress_basis_points": 9000, "recorded_at": "2026-08-29T10:00:00Z"
            })],
        )
        .await,
        StatusCode::ACCEPTED,
    )
    .await;

    let view = reader(&client, &doc).await;
    assert_eq!(view["progress_percent"], 90, "the real figure is preserved");
    assert!(
        !view["finished_at"].is_null(),
        "the finished latch follows the event kind, not the number"
    );
}

/// The converse of the same rule: reaching the end is not a declaration of having finished, so
/// `progress` at 100% is legal and must not latch on the reader's behalf.
#[tokio::test]
async fn full_progress_without_a_finished_event_does_not_latch() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let doc = DocumentFactory::new(session.user.id)
        .with_document_type(DocumentType::Book)
        .insert(app.pool())
        .await
        .id
        .to_string();

    assert_status(
        append(
            &client,
            &doc,
            CLIENT_A,
            vec![event(&rev(70), 1, 100, 1, "2026-08-29T10:00:00Z")],
        )
        .await,
        StatusCode::ACCEPTED,
    )
    .await;

    let view = reader(&client, &doc).await;
    assert_eq!(view["progress_percent"], 100);
    assert!(
        view["finished_at"].is_null(),
        "only a finished event latches"
    );
}

/// A reread is a deliberate act, not a regression to discard: a higher attempt wins outright,
/// and the previous pass's ceiling and finish do not carry into it.
#[tokio::test]
async fn a_new_attempt_outranks_the_previous_pass_and_starts_clean() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let doc = DocumentFactory::new(session.user.id)
        .with_document_type(DocumentType::Book)
        .insert(app.pool())
        .await
        .id
        .to_string();

    assert_status(
        append(
            &client,
            &doc,
            CLIENT_A,
            vec![json!({
                "id": rev(60), "origin_seq": 1, "kind": "finished",
                "progress_basis_points": 10000, "recorded_at": "2026-08-29T10:00:00Z"
            })],
        )
        .await,
        StatusCode::ACCEPTED,
    )
    .await;
    let done = reader(&client, &doc).await;
    assert_eq!(done["max_progress_percent"], 100);
    assert!(!done["finished_at"].is_null());

    assert_status(
        append(
            &client,
            &doc,
            CLIENT_A,
            vec![json!({
                "id": rev(61), "origin_seq": 2, "kind": "progress", "attempt": 2,
                "progress_basis_points": 300, "recorded_at": "2026-08-29T11:00:00Z"
            })],
        )
        .await,
        StatusCode::ACCEPTED,
    )
    .await;

    let reread = reader(&client, &doc).await;
    assert_eq!(
        reread["progress_percent"], 3,
        "the reread position takes hold"
    );
    assert_eq!(
        reread["max_progress_percent"], 3,
        "the previous attempt's ceiling must not carry into this one"
    );
    assert!(
        reread["finished_at"].is_null(),
        "a new attempt is not already finished"
    );
}
