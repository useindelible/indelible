#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use chrono::Utc;
use ind_integrations::notion::{
    NOTION_API_VERSION, NotionBlock, NotionClient, NotionError, NotionPageSpec, NotionRateLimiter,
};
use serde_json::{Value, json};
use wiremock::matchers::{body_partial_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn managed_properties() -> Value {
    json!({
        "Title": {"id": "p_title", "type": "title", "title": {}},
        "Author": {"id": "p_author", "type": "rich_text", "rich_text": {}},
        "URL": {"id": "p_url", "type": "url", "url": {}},
        "Canonical URL": {"id": "p_canonical", "type": "url", "url": {}},
        "Source": {"id": "p_source", "type": "select", "select": {}},
        "Saved At": {"id": "p_saved", "type": "date", "date": {}},
        "Tags": {"id": "p_tags", "type": "multi_select", "multi_select": {}},
        "Category": {"id": "p_category", "type": "select", "select": {}},
        "Reading Status": {"id": "p_status", "type": "select", "select": {}},
        "Indelible ID": {"id": "p_indelible", "type": "rich_text", "rich_text": {}},
        "Last Synced At": {"id": "p_synced", "type": "date", "date": {}}
    })
}

fn client(server: &MockServer) -> NotionClient {
    NotionClient::new(
        "secret".into(),
        server.uri(),
        Arc::new(NotionRateLimiter::new(1_000.0)),
    )
}

#[tokio::test]
async fn notion_client_crosses_managed_target_page_block_and_rate_limit_contracts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/search"))
        .and(header("authorization", "Bearer secret"))
        .and(header("notion-version", NOTION_API_VERSION))
        .and(body_partial_json(json!({
            "query": "Indelible",
            "filter": {"value": "data_source"}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{"id": "source_existing", "parent": {"database_id": "database_existing"}}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/data_sources/source_existing"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"properties": managed_properties()})),
        )
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/views"))
        .and(query_param("database_id", "database_existing"))
        .and(query_param("page_size", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{"view": {"name": "Articles"}}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/views"))
        .and(body_partial_json(json!({
            "database_id": "database_existing",
            "data_source_id": "source_existing",
            "type": "table"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(6)
        .mount(&server)
        .await;

    let notion = client(&server);
    let target = notion.find_or_create_database().await.unwrap();
    assert_eq!(target.database_id, "database_existing");
    assert_eq!(target.data_source_id, "source_existing");
    assert_eq!(target.property_ids.indelible_id, "p_indelible");
    assert!(target.property_ids.is_complete());
    assert_eq!(
        notion
            .validate_managed_target("database_existing", "source_existing")
            .await
            .unwrap(),
        target
    );

    Mock::given(method("POST"))
        .and(path("/v1/data_sources/source_existing/query"))
        .and(body_partial_json(json!({
            "filter": {"property": "p_indelible", "rich_text": {"equals": "doc_example"}}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"results": []})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/pages"))
        .and(body_partial_json(json!({
            "parent": {"type": "data_source_id", "data_source_id": "source_existing"}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "page_created"})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v1/pages/page_created"))
        .and(body_partial_json(json!({"properties": {}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v1/blocks/page_created/children"))
        .and(body_partial_json(
            json!({"children": [{"type": "divider"}]}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let spec = NotionPageSpec {
        indelible_id: "doc_example".into(),
        title: "  Notion boundary  ".into(),
        url: Some("https://example.com/article".into()),
        canonical_url: None,
        author: Some("Example Author".into()),
        source: "extension".into(),
        saved_at: Utc::now(),
        tags: vec!["rust".into(), "  ".into()],
        item_type: "article".into(),
        triage_state: "inbox".into(),
        property_ids: target.property_ids.clone(),
    };
    let page_id = notion
        .upsert_page("source_existing", None, &spec)
        .await
        .unwrap();
    assert_eq!(page_id, "page_created");
    assert_eq!(
        notion
            .upsert_page("source_existing", Some(&page_id), &spec)
            .await
            .unwrap(),
        page_id
    );
    notion
        .append_blocks(&page_id, &[NotionBlock::Divider])
        .await
        .unwrap();

    Mock::given(method("PATCH"))
        .and(path("/v1/pages/page_created"))
        .and(body_partial_json(json!({"in_trash": true})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "page_created",
            "in_trash": true,
            "url": "https://www.notion.so/Archived-page_created"
        })))
        .expect(1)
        .mount(&server)
        .await;
    assert_eq!(
        notion.archive_page(&page_id).await.unwrap(),
        "https://www.notion.so/Archived-page_created"
    );

    Mock::given(method("GET"))
        .and(path("/v1/data_sources/rate_limited"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "7"))
        .expect(1)
        .mount(&server)
        .await;
    let error = notion
        .validate_managed_target("database_existing", "rate_limited")
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        NotionError::RateLimited {
            retry_after_secs: 7
        }
    ));
}
