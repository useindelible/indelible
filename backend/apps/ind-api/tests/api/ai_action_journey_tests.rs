use ind_domain::{
    DocumentId, ExtractEntitiesDocumentJob, GenericJobEnvelope, JobOutboxId,
    SuggestTagsDocumentJob, SummarizeDocumentJob, job_types,
};
use reqwest::StatusCode;
use serde_json::{Value, json};
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::common::{
    SaveScenario, assert_json_response, build_worker_context, document_id_from_response,
};

fn completion(content: Value) -> Value {
    json!({
        "id": "completion_surgical",
        "model": "surgical-model",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": content.to_string()},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 21, "completion_tokens": 8, "total_tokens": 29}
    })
}

async fn mount_completion(server: &MockServer, schema: &str, content: Value) {
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(json!({
            "response_format": {"json_schema": {"name": schema}}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion(content)))
        .expect(1)
        .mount(server)
        .await;
}

async fn dispatch(
    context: &ind_worker::context::WorkerContext,
    job_type: &str,
    payload: Value,
) -> Result<Option<()>, ind_application::AppError> {
    ind_worker::jobs::ai::dispatch_generic_job(
        &context.ai_search_jobs(),
        GenericJobEnvelope {
            outbox_id: JobOutboxId::new(),
            job_type: job_type.to_string(),
            payload,
            dedupe_key: None,
        },
    )
    .await
}

#[tokio::test]
async fn ai_actions_cross_http_worker_storage_and_persistence_boundaries() {
    let provider = MockServer::start().await;
    mount_completion(
        &provider,
        "summary",
        json!({"summary": "A concise systems article."}),
    )
    .await;
    mount_completion(
        &provider,
        "tags",
        json!({"tags": [" Rust ", "systems", "rust"]}),
    )
    .await;
    mount_completion(
        &provider,
        "entities",
        json!({"entities": [{
            "name": "Rust Foundation",
            "entity_type": "organization",
            "description": "Stewards Rust",
            "mention_count": 2,
            "aliases": ["RF"]
        }]}),
    )
    .await;

    let scenario = SaveScenario::new().await;
    let saved = scenario
        .extension_reader_save("https://example.com/ai-action-boundary")
        .await;
    let document_id: DocumentId = document_id_from_response(&saved).parse().unwrap();
    let client = scenario.web_client();
    let configured = assert_json_response(
        client
            .post_json(
                "/api/v1/mila/config",
                &json!({
                    "chat_api_base": format!("{}/v1", provider.uri()),
                    "chat_model": "surgical-model",
                    "embedding_api_base": format!("{}/v1", provider.uri()),
                    "embedding_model": "surgical-embedding",
                    "embedding_dim": 768,
                    "model_context_window": 16000,
                    "chat_context_pct": 70,
                    "top_k": 5,
                    "cross_item_top_k": 10,
                    "cross_item_max_per_item": 3,
                    "enabled": true,
                    "byo_enabled": true,
                    "supports_structured_output": true,
                    "supports_reasoning_effort": true
                }),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(configured["byo_enabled"], true);

    let context = build_worker_context(&scenario.app);
    assert_eq!(
        dispatch(
            &context,
            job_types::DOCUMENT_AI_SUMMARIZE,
            serde_json::to_value(SummarizeDocumentJob { document_id }).unwrap(),
        )
        .await
        .unwrap(),
        Some(())
    );
    dispatch(
        &context,
        job_types::DOCUMENT_AI_TAGS,
        serde_json::to_value(SuggestTagsDocumentJob { document_id }).unwrap(),
    )
    .await
    .unwrap();
    dispatch(
        &context,
        job_types::DOCUMENT_AI_ENTITIES,
        serde_json::to_value(ExtractEntitiesDocumentJob { document_id }).unwrap(),
    )
    .await
    .unwrap();

    let outputs: Vec<(String, Value)> = sqlx::query_as(
        "SELECT output_type, content FROM ai_outputs WHERE document_id = $1 ORDER BY output_type",
    )
    .bind(document_id.into_uuid())
    .fetch_all(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(outputs.len(), 3);
    assert_eq!(
        outputs[0],
        (
            "entities".into(),
            json!([{
                "name": "Rust Foundation",
                "entity_type": "organization",
                "description": "Stewards Rust",
                "mention_count": 2,
                "aliases": ["RF"]
            }])
        )
    );
    assert_eq!(
        outputs[1],
        ("summary".into(), json!("A concise systems article."))
    );
    assert_eq!(outputs[2], ("tags".into(), json!(["rust", "systems"])));

    let completed_runs: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM ai_runs WHERE document_id = $1 AND status = 'completed' \
         AND input_tokens = 21 AND output_tokens = 8",
    )
    .bind(document_id.into_uuid())
    .fetch_one(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(completed_runs, 3);
    let completed_events: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM domain_events WHERE aggregate_id = $1 AND event_type = 'ai.output.completed'",
    )
    .bind(document_id.into_uuid())
    .fetch_one(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(completed_events, 3);
    let entity: (String, i32) = sqlx::query_as(
        "SELECT e.name, m.mention_count FROM entities e \
         JOIN entity_mentions m ON m.entity_id = e.id \
         WHERE m.document_id = $1",
    )
    .bind(document_id.into_uuid())
    .fetch_one(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(entity, ("Rust Foundation".into(), 2));

    provider.reset().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(503).set_body_json(json!({
            "error": {"message": "provider unavailable"}
        })))
        .expect(1)
        .mount(&provider)
        .await;
    let failure = dispatch(
        &context,
        job_types::DOCUMENT_AI_SUMMARIZE,
        serde_json::to_value(SummarizeDocumentJob { document_id }).unwrap(),
    )
    .await;
    assert!(failure.is_err());
    let failed: (i64, i64) = sqlx::query_as(
        "SELECT \
           count(*) FILTER (WHERE status = 'failed'), \
           count(*) FILTER (WHERE status = 'failed' AND error_message LIKE '%provider unavailable%') \
         FROM ai_runs WHERE document_id = $1",
    )
    .bind(document_id.into_uuid())
    .fetch_one(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(failed, (1, 1));
    let failed_events: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM domain_events WHERE aggregate_id = $1 AND event_type = 'ai.output.failed'",
    )
    .bind(document_id.into_uuid())
    .fetch_one(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(failed_events, 1);
}
