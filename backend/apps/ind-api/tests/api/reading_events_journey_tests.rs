use ind_domain::DocumentType;
use ind_test_support::{AuthedClient, DocumentFactory, spawn_app};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::{assert_json_response, assert_status};

const CLIENT_A: &str = "cli_018f5b1e-0000-7000-8000-00000000000a";
const CLIENT_B: &str = "cli_018f5b1e-0000-7000-8000-00000000000b";

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

#[tokio::test]
async fn reading_events_replay_order_and_expose_position() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let doc = DocumentFactory::new(session.user.id)
        .with_document_type(DocumentType::Pdf)
        .insert(app.pool())
        .await
        .id
        .to_string();
    let batch = vec![
        event(&rev(1), 1, 10, 1, "2026-08-29T10:00:00Z"),
        event(&rev(2), 2, 20, 2, "2026-08-29T10:05:00Z"),
    ];

    let first = assert_json_response(
        append(&client, &doc, CLIENT_A, batch.clone()).await,
        StatusCode::ACCEPTED,
    )
    .await;
    assert_eq!(first, json!({"accepted": 2, "replayed": 0}));
    let replay = assert_json_response(
        append(&client, &doc, CLIENT_A, batch).await,
        StatusCode::ACCEPTED,
    )
    .await;
    assert_eq!(replay, json!({"accepted": 0, "replayed": 2}));

    assert_status(
        append(
            &client,
            &doc,
            CLIENT_A,
            vec![event(&rev(2), 2, 21, 2, "2026-08-29T10:05:00Z")],
        )
        .await,
        StatusCode::CONFLICT,
    )
    .await;

    // origin_seq 0 is unused so far for CLIENT_A (1 and 2 are taken) and sits behind the
    // high-water mark of 2, so this exercises the not-projected path without tripping the
    // (origin, origin_seq) uniqueness constraint.
    assert_status(
        append(
            &client,
            &doc,
            CLIENT_A,
            vec![event(&rev(3), 0, 5, 1, "2026-08-29T10:06:00Z")],
        )
        .await,
        StatusCode::ACCEPTED,
    )
    .await;
    let view = reader(&client, &doc).await;
    assert_eq!(
        view["progress_percent"], 20,
        "behind own watermark must not project"
    );
    assert_eq!(
        view["position"],
        json!({"anchor": {"type": "page", "page": 2}, "fraction": 0.25})
    );

    assert_status(
        append(
            &client,
            &doc,
            CLIENT_B,
            vec![event(&rev(4), 1, 15, 3, "2026-08-29T09:00:00Z")],
        )
        .await,
        StatusCode::ACCEPTED,
    )
    .await;
    assert_eq!(
        reader(&client, &doc).await["progress_percent"],
        20,
        "older reading time on another device loses"
    );

    assert_status(
        append(
            &client,
            &doc,
            CLIENT_B,
            vec![event(&rev(5), 2, 15, 3, "2026-08-29T11:00:00Z")],
        )
        .await,
        StatusCode::ACCEPTED,
    )
    .await;
    let view = reader(&client, &doc).await;
    assert_eq!(
        view["progress_percent"], 15,
        "later reading time on another device wins"
    );
    assert_eq!(view["chapter_locator"], "page:3");
    assert_eq!(view["max_progress_percent"], 20);
}

#[tokio::test]
async fn reading_events_validate_shape_and_ownership() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let stranger = app.create_web_session().await;
    let doc = DocumentFactory::new(owner.user.id)
        .with_document_type(DocumentType::Book)
        .insert(app.pool())
        .await
        .id
        .to_string();
    let client = app.authed_client(&owner);
    let ok = |id: &str, seq: i64| event(id, seq, 10, 1, "2026-08-29T10:00:00Z");

    assert_status(
        append(
            &app.authed_client(&stranger),
            &doc,
            CLIENT_A,
            vec![ok(&rev(9), 1)],
        )
        .await,
        StatusCode::NOT_FOUND,
    )
    .await;
    // `client_id`/`id` are typed ids: a value that fails their prefixed-string parse never
    // reaches Validate() — it is rejected at JSON deserialization as 400, not 422.
    for (device, events) in [
        ("surface:web", vec![ok(&rev(10), 1)]),
        ("phone", vec![ok(&rev(11), 1)]),
        (
            CLIENT_A,
            vec![
                json!({"id": "018f5b1e-0000-7000-8000-000000000020", "origin_seq": 1, "kind": "progress", "progress_basis_points": 500, "recorded_at": "2026-08-29T10:00:00Z"}),
            ],
        ),
    ] {
        assert_status(
            append(&client, &doc, device, events).await,
            StatusCode::BAD_REQUEST,
        )
        .await;
    }
    for (device, events) in [
        (CLIENT_A, vec![]),
        (CLIENT_A, vec![ok(&rev(12), 2), ok(&rev(13), 2)]),
        (CLIENT_A, vec![ok(&rev(14), -1)]),
        (
            CLIENT_A,
            vec![
                json!({"id": rev(17), "origin_seq": 1, "kind": "opened", "progress_basis_points": 500, "recorded_at": "2026-08-29T10:00:00Z"}),
            ],
        ),
        (
            CLIENT_A,
            vec![
                json!({"id": rev(18), "origin_seq": 1, "kind": "progress", "progress_basis_points": 500, "position": {"offset": -1}, "recorded_at": "2026-08-29T10:00:00Z"}),
            ],
        ),
        (
            CLIENT_A,
            vec![
                json!({"id": rev(19), "origin_seq": 1, "kind": "progress", "progress_basis_points": 500, "position": {"fraction": 1.5}, "recorded_at": "2026-08-29T10:00:00Z"}),
            ],
        ),
        (
            CLIENT_A,
            vec![
                json!({"id": rev(24), "origin_seq": 1, "kind": "progress", "progress_basis_points": 500, "position": {"seconds": -0.5}, "recorded_at": "2026-08-29T10:00:00Z"}),
            ],
        ),
        (
            CLIENT_A,
            vec![
                json!({"id": rev(25), "origin_seq": 1, "kind": "progress", "progress_basis_points": 500, "position": {"anchor": {"type": "page", "page": 0}}, "recorded_at": "2026-08-29T10:00:00Z"}),
            ],
        ),
        (
            CLIENT_A,
            vec![
                json!({"id": rev(26), "origin_seq": 1, "kind": "progress", "progress_basis_points": 500, "position": {"anchor": {"type": "spine", "chapter": ""}}, "recorded_at": "2026-08-29T10:00:00Z"}),
            ],
        ),
    ] {
        assert_status(
            append(&client, &doc, device, events).await,
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .await;
    }
    assert_status(
        append(&client, &doc, CLIENT_A, vec![
            json!({"id": rev(21), "origin_seq": 1, "kind": "opened", "recorded_at": "2026-08-29T10:00:00Z"}),
            json!({"id": rev(22), "origin_seq": 2, "kind": "finished", "progress_basis_points": 10000, "recorded_at": "2026-08-29T10:00:00Z"}),
        ]).await,
        StatusCode::ACCEPTED,
    )
    .await;
    assert!(reader(&client, &doc).await["finished_at"].is_string());
}
