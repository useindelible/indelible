use chrono::Utc;
use ind_application::repos::entity::EntityRepository;
use ind_application::repos::search::SearchRepository;
use ind_domain::{
    DocumentId, EntityType, SearchDocument, SearchDocumentId, SearchDocumentKind,
    SearchDocumentSource, UserId,
};
use ind_persistence::repos::{PgEntityRepository, PgSearchRepository};
use ind_test_support::{DocumentFactory, LibraryEntryFactory, TestApp, spawn_app};
use reqwest::StatusCode;

use super::common::assert_json_response;

async fn indexed_document(app: &TestApp, user_id: UserId, title: &str, body: &str) -> DocumentId {
    let document = DocumentFactory::new(user_id)
        .with_title(title)
        .insert(app.pool())
        .await;
    LibraryEntryFactory::new(user_id, document.id)
        .insert(app.pool())
        .await;
    let now = Utc::now();
    PgSearchRepository::new(app.pool().clone())
        .upsert_search_document(&SearchDocument {
            id: SearchDocumentId::new(),
            source: SearchDocumentSource::Document {
                document_id: document.id,
            },
            user_id,
            document_kind: SearchDocumentKind::Item,
            section_key: String::new(),
            section_title: None,
            title: title.into(),
            body_text: body.into(),
            highlight_text: String::new(),
            metadata_text: String::new(),
            search_config: "simple".into(),
            saved_at: now,
            updated_at: now,
        })
        .await
        .unwrap();
    document.id
}

#[tokio::test]
async fn search_api_crosses_engine_cursor_suggestions_recent_and_tenant_boundaries() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let owner_client = app.authed_client(&owner);
    let first = indexed_document(
        &app,
        owner.user.id,
        "Rust ownership model",
        "Rust prevents data races through ownership.",
    )
    .await;
    let second = indexed_document(
        &app,
        owner.user.id,
        "Rust async model",
        "Rust futures make concurrent systems explicit.",
    )
    .await;

    let entities = PgEntityRepository::new(app.pool().clone());
    let foundation = entities
        .insert_canonical(
            owner.user.id,
            "Rust Foundation",
            EntityType::Organization,
            None,
        )
        .await
        .unwrap();
    entities
        .set_document_mentions(owner.user.id, first, &[(foundation.id, 4)])
        .await
        .unwrap();

    let page_one_response = owner_client.get("/api/v1/search?q=rust&limit=1").await;
    assert_eq!(page_one_response.headers()["x-ratelimit-limit"], "60");
    let page_one = assert_json_response(page_one_response, StatusCode::OK).await;
    assert_eq!(page_one["results"].as_array().unwrap().len(), 1);
    assert_eq!(page_one["has_more"], true);
    let cursor = page_one["next_cursor"].as_str().unwrap();
    let page_one_id = page_one["results"][0]["document_id"].as_str().unwrap();

    let page_two = assert_json_response(
        owner_client
            .get(&format!("/api/v1/search?q=rust&limit=1&cursor={cursor}"))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(page_two["results"].as_array().unwrap().len(), 1);
    let page_two_id = page_two["results"][0]["document_id"].as_str().unwrap();
    assert_ne!(page_two_id, page_one_id);
    assert!([first.to_string(), second.to_string()].contains(&page_one_id.to_string()));
    assert!([first.to_string(), second.to_string()].contains(&page_two_id.to_string()));

    let entity_page = assert_json_response(
        owner_client
            .get("/api/v1/search?q=entity:%22Rust%20Foundation%22")
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(entity_page["results"].as_array().unwrap().len(), 1);
    assert_eq!(entity_page["results"][0]["document_id"], first.to_string());
    assert_eq!(
        entity_page["results"][0]["entity_chips"][0]["name"],
        "Rust Foundation"
    );
    assert_eq!(entity_page["entity_card"]["mention_count"], 4);

    let dynamic_suggestions = assert_json_response(
        owner_client
            .get("/api/v1/search/suggestions?q=entity:Rust")
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(dynamic_suggestions["suggestions"][0]["kind"], "entity");
    assert_eq!(
        dynamic_suggestions["suggestions"][0]["insert_text"],
        "entity:\"Rust Foundation\""
    );
    let static_suggestions = assert_json_response(
        owner_client
            .get("/api/v1/search/suggestions?q=type:a")
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        static_suggestions["suggestions"][0]["insert_text"],
        "type:article"
    );
    for query in [
        "tag:",
        "collection:",
        "sender_domain:",
        "sender:",
        "list:",
        "author:",
        "is:",
        "has:",
        "pinned:",
    ] {
        let response = assert_json_response(
            owner_client
                .get(&format!("/api/v1/search/suggestions?q={query}"))
                .await,
            StatusCode::OK,
        )
        .await;
        assert!(response["suggestions"].is_array(), "{query}");
    }

    assert_eq!(
        owner_client
            .get("/api/v1/search?q=rust&cursor=not-a-cursor")
            .await
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        owner_client.get("/api/v1/search?q=%20").await.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let recent = assert_json_response(
        owner_client.get("/api/v1/search/recent").await,
        StatusCode::OK,
    )
    .await;
    assert!(recent["items"].as_array().unwrap().len() >= 2);
    let recent_id = recent["items"][0]["id"].as_str().unwrap();

    let stranger = app.create_web_session().await;
    let stranger_client = app.authed_client(&stranger);
    let isolated = assert_json_response(
        stranger_client.get("/api/v1/search?q=rust").await,
        StatusCode::OK,
    )
    .await;
    assert!(isolated["results"].as_array().unwrap().is_empty());
    assert_eq!(
        stranger_client
            .delete(&format!("/api/v1/search/recent/{recent_id}"))
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    let owner_recent = assert_json_response(
        owner_client.get("/api/v1/search/recent").await,
        StatusCode::OK,
    )
    .await;
    assert!(
        owner_recent["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == recent_id)
    );
    assert_eq!(
        owner_client.delete("/api/v1/search/recent").await.status(),
        StatusCode::NO_CONTENT
    );
    let cleared = assert_json_response(
        owner_client.get("/api/v1/search/recent").await,
        StatusCode::OK,
    )
    .await;
    assert!(cleared["items"].as_array().unwrap().is_empty());
}
