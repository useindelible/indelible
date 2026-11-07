use ind_domain::DocumentType;
use ind_test_support::{AuthedClient, DocumentFactory, spawn_app};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::{assert_json_response, assert_status};

const CLIENT_A: &str = "cli_018f5b1e-0000-7000-8000-00000000000a";

fn rev(n: u32) -> String {
    format!("rev_{}", uuid::Uuid::from_u128(u128::from(n)))
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
async fn every_document_type_records_and_returns_its_own_position() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    for (n, (document_type, position)) in [
        (
            DocumentType::Article,
            json!({"anchor": {"type": "section", "section": "sec-3"}, "offset": 4210, "fraction": 0.44}),
        ),
        (
            DocumentType::Book,
            json!({"anchor": {"type": "spine", "chapter": "OEBPS/ch09.xhtml"}, "offset": 1180, "fraction": 0.62}),
        ),
        (
            DocumentType::Pdf,
            json!({"anchor": {"type": "page", "page": 12}, "fraction": 0.31}),
        ),
        (
            DocumentType::Video,
            json!({"anchor": {"type": "cue", "cue": "0142"}, "fraction": 0.63, "seconds": 757.4}),
        ),
        (
            DocumentType::Podcast,
            json!({"fraction": 0.18, "seconds": 1284.0}),
        ),
        (
            DocumentType::Email,
            json!({"anchor": {"type": "section", "section": "quote-2"}, "offset": 96}),
        ),
        (DocumentType::Tweet, json!({"fraction": 1.0})),
    ]
    .into_iter()
    .enumerate()
    {
        let doc = DocumentFactory::new(session.user.id)
            .with_document_type(document_type)
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
                    "id": rev(100 + n as u32), "origin_seq": 1, "kind": "progress",
                    "progress_basis_points": 4200, "position": position.clone(),
                    "recorded_at": "2026-08-29T10:00:00Z"
                })],
            )
            .await,
            StatusCode::ACCEPTED,
        )
        .await;
        assert_eq!(
            reader(&client, &doc).await["position"],
            position,
            "{document_type} lost its position"
        );
    }
}

#[tokio::test]
async fn an_unrecognised_anchor_kind_is_refused() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let doc = DocumentFactory::new(session.user.id)
        .with_document_type(DocumentType::Video)
        .insert(app.pool())
        .await
        .id
        .to_string();

    for (n, anchor) in [
        json!({"type": "frame", "frame": "94"}),
        json!({"type": "other", "kind": "frame", "value": "94"}),
        json!({"page": 12}),
    ]
    .into_iter()
    .enumerate()
    {
        assert_status(
            append(
                &client,
                &doc,
                CLIENT_A,
                vec![json!({
                    "id": rev(30 + n as u32), "origin_seq": 1, "kind": "progress",
                    "progress_basis_points": 500, "position": {"anchor": anchor},
                    "recorded_at": "2026-08-29T10:00:00Z"
                })],
            )
            .await,
            StatusCode::BAD_REQUEST,
        )
        .await;
    }

    let cue = json!({"anchor": {"type": "cue", "cue": "0142"}, "seconds": 12.5});
    assert_status(
        append(
            &client,
            &doc,
            CLIENT_A,
            vec![json!({
                "id": rev(40), "origin_seq": 1, "kind": "progress", "progress_basis_points": 500,
                "position": cue.clone(), "recorded_at": "2026-08-29T10:00:00Z"
            })],
        )
        .await,
        StatusCode::ACCEPTED,
    )
    .await;
    let view = reader(&client, &doc).await;
    assert_eq!(view["position"], cue);
    assert_eq!(view["chapter_locator"], "cue:0142");
}
