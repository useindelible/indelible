use ind_domain::job_types;
use ind_test_support::{DocumentFactory, spawn_app};
use reqwest::StatusCode;

use super::common::assert_json_response;
use super::resource_route_permission_support::{RouteCase, RoutePermissionFixture};

fn enabled_mila_config() -> serde_json::Value {
    serde_json::json!({
        "chat_api_base": "https://api.openai.com/v1",
        "chat_model": "gpt-4.1-mini",
        "embedding_api_base": "https://api.openai.com/v1",
        "embedding_model": "text-embedding-3-small",
        "embedding_dim": 768,
        "model_context_window": 16000,
        "chat_context_pct": 70,
        "top_k": 5,
        "cross_item_top_k": 10,
        "cross_item_max_per_item": 3,
        "enabled": true
    })
}

#[tokio::test]
async fn owner_can_retry_each_supported_mila_document_action() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let document = DocumentFactory::new(session.user.id)
        .with_title("Retryable Mila output")
        .insert(app.pool())
        .await;
    let client = app.authed_client(&session);
    assert_eq!(
        client
            .post_json("/api/v1/mila/config", &enabled_mila_config())
            .await
            .status(),
        StatusCode::OK
    );

    for (action, job_type) in [
        ("summary", job_types::DOCUMENT_AI_SUMMARIZE),
        ("tags", job_types::DOCUMENT_AI_TAGS),
        ("entities", job_types::DOCUMENT_AI_ENTITIES),
    ] {
        let response = assert_json_response(
            client
                .post_json(
                    &format!(
                        "/api/v1/mila/documents/{}/actions/{action}/retry",
                        document.id
                    ),
                    &serde_json::json!({}),
                )
                .await,
            StatusCode::OK,
        )
        .await;
        assert_eq!(response["queued"], true);
        assert_eq!(response["action"], action);

        let queued: (
            String,
            serde_json::Value,
            Option<chrono::DateTime<chrono::Utc>>,
        ) = sqlx::query_as(
            "SELECT job_type, payload, dispatched_at FROM job_outbox WHERE job_type = $1",
        )
        .bind(job_type)
        .fetch_one(app.pool())
        .await
        .unwrap();
        assert_eq!(queued.0, job_type);
        assert_eq!(queued.1["document_id"], document.id.to_string());
        assert!(queued.2.is_none());

        if action == "summary" {
            sqlx::query(
				"UPDATE job_outbox SET dispatched_at = now(), available_at = now() + INTERVAL '1 hour' WHERE job_type = $1",
			)
			.bind(job_type)
			.execute(app.pool())
			.await
			.unwrap();
            let retry_started = chrono::Utc::now();
            assert_eq!(
                client
                    .post_json(
                        &format!(
                            "/api/v1/mila/documents/{}/actions/{action}/retry",
                            document.id
                        ),
                        &serde_json::json!({}),
                    )
                    .await
                    .status(),
                StatusCode::OK
            );
            let replayed: (
                chrono::DateTime<chrono::Utc>,
                Option<chrono::DateTime<chrono::Utc>>,
            ) = sqlx::query_as(
                "SELECT available_at, dispatched_at FROM job_outbox WHERE job_type = $1",
            )
            .bind(job_type)
            .fetch_one(app.pool())
            .await
            .unwrap();
            assert!(replayed.0 >= retry_started - chrono::Duration::seconds(1));
            assert!(replayed.1.is_none());
        }
    }
}

#[tokio::test]
async fn retry_rejects_disabled_mila_without_enqueuing() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let document = DocumentFactory::new(session.user.id)
        .with_title("Mila disabled")
        .insert(app.pool())
        .await;

    assert_eq!(
        app.authed_client(&session)
            .post_json(
                &format!(
                    "/api/v1/mila/documents/{}/actions/summary/retry",
                    document.id
                ),
                &serde_json::json!({}),
            )
            .await
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let queued: i64 = sqlx::query_scalar("SELECT count(*) FROM job_outbox WHERE job_type = $1")
        .bind(job_types::DOCUMENT_AI_SUMMARIZE)
        .fetch_one(app.pool())
        .await
        .unwrap();
    assert_eq!(queued, 0);
}

#[tokio::test]
async fn retry_rejects_foreign_documents_and_unsupported_actions() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let caller = app.create_web_session().await;
    let document = DocumentFactory::new(owner.user.id)
        .with_title("Another user's document")
        .insert(app.pool())
        .await;
    let client = app.authed_client(&caller);

    assert_eq!(
        client
            .post_json(
                &format!(
                    "/api/v1/mila/documents/{}/actions/summary/retry",
                    document.id
                ),
                &serde_json::json!({}),
            )
            .await
            .status(),
        StatusCode::NOT_FOUND
    );

    for action in ["chat", "custom"] {
        assert_eq!(
            client
                .post_json(
                    &format!(
                        "/api/v1/mila/documents/{}/actions/{action}/retry",
                        document.id
                    ),
                    &serde_json::json!({}),
                )
                .await
                .status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    let queued: i64 =
        sqlx::query_scalar("SELECT count(*) FROM job_outbox WHERE job_type IN ($1, $2, $3)")
            .bind(job_types::DOCUMENT_AI_SUMMARIZE)
            .bind(job_types::DOCUMENT_AI_TAGS)
            .bind(job_types::DOCUMENT_AI_ENTITIES)
            .fetch_one(app.pool())
            .await
            .unwrap();
    assert_eq!(queued, 0);
}

#[tokio::test]
async fn retry_requires_ai_use_and_library_read_permissions() {
    let fixture = RoutePermissionFixture::new().await;
    fixture
        .assert_pat_composite_matrix(
            &["ai:use", "library:read"],
            &[&["ai:use"], &["library:read"]],
            &[RouteCase::post(
                "/api/v1/mila/documents/bad/actions/summary/retry",
            )],
        )
        .await;
}
