use ind_domain::{DocumentId, DocumentType, UserId};
use ind_test_support::{
    AuthedClient, DocumentFactory, TestApp, TestPersonalAccessToken, spawn_app,
};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::assert_json_response;

fn rev(n: u32) -> String {
    format!("rev_{}", uuid::Uuid::from_u128(u128::from(n)))
}

/// No `origin_seq`: a caller without a device counter has the server assign one.
fn progress_event(id: &str) -> Value {
    json!({
        "id": id, "kind": "progress", "progress_basis_points": 1000,
        "recorded_at": "2026-08-29T10:00:00Z"
    })
}

fn device_progress_event(id: &str, seq: i64) -> Value {
    json!({
        "id": id, "origin_seq": seq, "kind": "progress", "progress_basis_points": 1000,
        "recorded_at": "2026-08-29T10:00:00Z"
    })
}

async fn append(client: &AuthedClient<'_>, doc: DocumentId, body: &Value) -> reqwest::Response {
    client
        .post_json(&format!("/api/v1/documents/{doc}/reading-events"), body)
        .await
}

async fn new_document(app: &TestApp, user_id: UserId) -> DocumentId {
    DocumentFactory::new(user_id)
        .with_document_type(DocumentType::Pdf)
        .insert(app.pool())
        .await
        .id
}

/// Reads back what `origin_from` actually stored, since the HTTP response never exposes it.
async fn stored_origins(pool: &sqlx::PgPool, doc: DocumentId) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT origin FROM reading_events WHERE document_id = $1 ORDER BY received_at",
    )
    .bind(doc.into_uuid())
    .fetch_all(pool)
    .await
    .expect("query stored reading event origins")
}

#[tokio::test]
async fn batch_without_client_id_falls_back_to_web_surface() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let doc = new_document(&app, session.user.id).await;

    assert_json_response(
        append(&client, doc, &json!({"events": [progress_event(&rev(1))]})).await,
        StatusCode::ACCEPTED,
    )
    .await;

    assert_eq!(
        stored_origins(app.pool(), doc).await,
        vec!["surface:web".to_string()]
    );
}

#[tokio::test]
async fn batch_without_client_id_falls_back_to_personal_access_token() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let doc = new_document(&app, session.user.id).await;

    let created = assert_json_response(
        app.authed_client(&session)
            .post_json(
                "/api/v1/tokens",
                &json!({"name": "reader", "permissions": ["library:write"]}),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let token_id = created["id"].as_str().expect("token id").to_string();
    let token = TestPersonalAccessToken::new(
        created["raw_token"]
            .as_str()
            .expect("raw token")
            .to_string(),
    );
    let client = app.authed_client(&token);

    assert_json_response(
        append(&client, doc, &json!({"events": [progress_event(&rev(2))]})).await,
        StatusCode::ACCEPTED,
    )
    .await;

    assert_eq!(stored_origins(app.pool(), doc).await, vec![token_id]);
}

#[tokio::test]
async fn progress_patch_omits_client_id_and_falls_back_to_caller_surface() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let doc = new_document(&app, session.user.id).await;

    let resp = client
        .patch_json(
            &format!("/api/v1/documents/{doc}/progress"),
            &json!({"progress_percent": 42}),
        )
        .await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    assert_eq!(
        stored_origins(app.pool(), doc).await,
        vec!["surface:web".to_string()]
    );
}

#[tokio::test]
async fn batch_with_client_id_wins_over_the_caller_credential() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let doc = new_document(&app, session.user.id).await;
    let device = "cli_018f5b1e-0000-7000-8000-0000000000ff";

    assert_json_response(
        append(
            &client,
            doc,
            &json!({"client_id": device, "events": [device_progress_event(&rev(3), 1)]}),
        )
        .await,
        StatusCode::ACCEPTED,
    )
    .await;

    assert_eq!(
        stored_origins(app.pool(), doc).await,
        vec![device.to_string()]
    );
}

/// The sequence authority follows the origin: a device supplies its own counter, and a caller
/// without one must not supply a sequence the server would have to reconcile with `PATCH`'s.
#[tokio::test]
async fn origin_seq_is_required_for_devices_and_refused_for_everyone_else() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let doc = new_document(&app, session.user.id).await;
    let device = "cli_018f5b1e-0000-7000-8000-00000000000d";

    let missing = json!({
        "client_id": device,
        "events": [progress_event(&rev(10))]
    });
    assert_eq!(
        append(&client, doc, &missing).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "a device batch must carry its own sequence"
    );

    let unwanted = json!({"events": [device_progress_event(&rev(11), 1)]});
    assert_eq!(
        append(&client, doc, &unwanted).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "a surface batch must leave the sequence to the server"
    );

    assert_json_response(
        append(&client, doc, &json!({"events": [progress_event(&rev(12))]})).await,
        StatusCode::ACCEPTED,
    )
    .await;
    assert_json_response(
        append(
            &client,
            doc,
            &json!({"client_id": device, "events": [device_progress_event(&rev(13), 7)]}),
        )
        .await,
        StatusCode::ACCEPTED,
    )
    .await;
}
