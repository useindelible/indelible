use ind_application::repos::feed::FeedRepository;
use ind_domain::{FeedSourceEntry, FeedSourceEntryId, FeedSourceId, FeedSubscriptionId};
use ind_persistence::repos::PgFeedRepository;
use ind_test_support::{
    AuthedClient, FeedDeliveryFactory, FeedSourceFactory, FeedSubscriptionFactory, TestApp,
    spawn_app,
};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::{assert_json_response, dispatch_pending_jobs};

async fn search(client: &AuthedClient<'_>, text: &str) -> Value {
    assert_json_response(
        client.get(&format!("/api/v1/search?q={text}")).await,
        StatusCode::OK,
    )
    .await
}

async fn seed_subscription(
    app: &TestApp,
    user_id: ind_domain::UserId,
) -> (FeedSourceId, FeedSubscriptionId) {
    let source = FeedSourceFactory.insert(app.pool()).await;
    let subscription = FeedSubscriptionFactory::new(user_id)
        .with_source(source.clone())
        .insert(app.pool())
        .await;
    (source.id, subscription.id)
}

async fn seed_delivery(
    app: &TestApp,
    user_id: ind_domain::UserId,
    source_id: FeedSourceId,
    subscription_id: FeedSubscriptionId,
    title: &str,
) -> ind_domain::FeedDeliveryId {
    let slug = uuid::Uuid::now_v7().simple().to_string();
    let entry = PgFeedRepository::new(app.pool().clone())
        .create_source_entry(FeedSourceEntry {
            id: FeedSourceEntryId::new(),
            source_id,
            guid: slug.clone(),
            title: title.to_string(),
            url: Some(format!("https://example.com/{slug}")),
            canonical_url: Some(format!("https://example.com/{slug}")),
            author: None,
            excerpt: Some("discovery preview".to_string()),
            content_html: None,
            language: None,
            lead_image_url: None,
            published_at: None,
            discovered_at: chrono::Utc::now(),
        })
        .await
        .expect("create source entry");
    FeedDeliveryFactory::new(user_id, subscription_id, source_id, entry.id)
        .insert(app.pool())
        .await
        .id
}

#[tokio::test]
async fn unprepared_delivery_is_preview_searchable_then_document_after_prepare() {
    let app = spawn_app().await;
    let web = app.create_web_session().await;
    let client = app.authed_client(&web);
    let (source_id, subscription_id) = seed_subscription(&app, web.user.id).await;
    let delivery_id = seed_delivery(
        &app,
        web.user.id,
        source_id,
        subscription_id,
        "Running migration",
    )
    .await;

    let preview = search(&client, "run").await;
    assert_eq!(preview["results"][0]["result_kind"], "feed_preview");
    assert_eq!(
        preview["results"][0]["delivery_id"],
        delivery_id.to_string()
    );
    assert!(preview["results"][0].get("document_id").is_none());

    let prepared = assert_json_response(
        client
            .post_json(
                &format!("/api/v1/feeds/deliveries/{delivery_id}/prepare"),
                &json!({}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let document_id = prepared["document_id"].as_str().unwrap();
    let stranger = app.create_web_session().await;
    assert_eq!(
        app.authed_client(&stranger)
            .get(&format!("/api/v1/feeds/deliveries/{delivery_id}"))
            .await
            .status(),
        StatusCode::NOT_FOUND
    );

    assert_eq!(
        dispatch_pending_jobs(&app, "feed.prepare_document").await,
        1
    );
    assert_eq!(
        dispatch_pending_jobs(&app, "search.reindex_document").await,
        1
    );

    let hits = search(&client, "run").await;
    assert_eq!(hits["results"].as_array().unwrap().len(), 1);
    assert_eq!(hits["results"][0]["result_kind"], "document");
    assert_eq!(hits["results"][0]["document_id"], document_id);
    let body_hits = search(&client, "readable").await;
    assert_eq!(body_hits["results"][0]["document_id"], document_id);

    let saved = assert_json_response(
        client
            .post_json(
                "/api/v1/library/from-delivery",
                &json!({"delivery_id": delivery_id.to_string()}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(saved["document_id"], document_id);
    let reindex_jobs = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM job_outbox \
         WHERE job_type = 'search.reindex_document' AND dispatched_at IS NULL",
    )
    .fetch_one(app.pool())
    .await
    .unwrap();
    assert_eq!(reindex_jobs, 1);

    assert_eq!(
        dispatch_pending_jobs(&app, "search.reindex_document").await,
        1
    );
    let hits = search(&client, "run").await;
    assert_eq!(hits["results"].as_array().unwrap().len(), 1);
    assert_eq!(hits["results"][0]["result_kind"], "document");
    assert_eq!(hits["results"][0]["document_id"], document_id);
}

#[tokio::test]
async fn marking_a_feed_delivery_seen_does_not_materialize_reader_state() {
    let app = spawn_app().await;
    let web = app.create_web_session().await;
    let client = app.authed_client(&web);
    let (source_id, subscription_id) = seed_subscription(&app, web.user.id).await;
    let delivery_id = seed_delivery(
        &app,
        web.user.id,
        source_id,
        subscription_id,
        "External reader boundary",
    )
    .await;

    assert_json_response(
        client
            .post_json(
                &format!("/api/v1/feeds/deliveries/{delivery_id}/seen"),
                &json!({}),
            )
            .await,
        StatusCode::OK,
    )
    .await;

    let documents =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM documents WHERE user_id = $1")
            .bind(web.user.id.into_uuid())
            .fetch_one(app.pool())
            .await
            .unwrap();
    let states =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_document_state WHERE user_id = $1")
            .bind(web.user.id.into_uuid())
            .fetch_one(app.pool())
            .await
            .unwrap();
    assert_eq!((documents, states), (0, 0));
}
