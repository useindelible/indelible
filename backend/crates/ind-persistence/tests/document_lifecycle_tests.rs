#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::Utc;
use ind_application::handlers::feed_identity::feed_entry_identity;
use ind_application::repos::document_lifecycle::{
    DocumentLifecycle, DocumentStateInput, MaterializeIdentity, MaterializeRequest,
    MaterializeSideEffects, SaveToLibraryRequest,
};
use ind_application::repos::feed::FeedRepository;
use ind_application::repos::lifecycle_outbox::OutboxEntry;
use ind_domain::{
    ContentSource, DocumentId, DocumentType, FeedSourceEntry, FeedSourceEntryId, NewUrlDocument,
    build_domain_event,
};
use ind_persistence::repos::{PgDocumentLifecycle, PgFeedRepository};
use ind_test_support::{
    FeedDeliveryFactory, FeedSourceFactory, FeedSubscriptionFactory, TestDb, UserFactory,
};

#[tokio::test]
async fn failed_transaction_rolls_back_everything() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let source = FeedSourceFactory.insert(&pool).await;
    let subscription = FeedSubscriptionFactory::new(user.id)
        .with_source(source.clone())
        .insert(&pool)
        .await;
    let canonical = format!("https://example.com/{}", uuid::Uuid::now_v7().simple());
    let source_entry = PgFeedRepository::new(pool.clone())
        .create_source_entry(FeedSourceEntry {
            id: FeedSourceEntryId::new(),
            source_id: source.id,
            guid: uuid::Uuid::now_v7().to_string(),
            title: "Source Entry".into(),
            url: Some(canonical.clone()),
            canonical_url: Some(canonical.clone()),
            author: None,
            excerpt: None,
            content_html: None,
            language: None,
            lead_image_url: None,
            published_at: None,
            discovered_at: Utc::now(),
        })
        .await
        .unwrap();
    let delivery = FeedDeliveryFactory::new(user.id, subscription.id, source.id, source_entry.id)
        .insert(&pool)
        .await;

    let user_id = user.id;
    let document_id = DocumentId::new();
    let request = MaterializeRequest {
        identity: MaterializeIdentity::Url {
            document: NewUrlDocument {
                id: document_id,
                user_id,
                document_type: DocumentType::Article,
                canonical_url: canonical.clone(),
                original_url: None,
                content_hash: None,
                title: "Materialized Title".into(),
                author: None,
                excerpt: None,
                published_at: None,
                language: None,
                domain: None,
                lead_image_url: None,
                thumbnail_url: None,
            },
            origin: None,
        },
        document_state: Some(DocumentStateInput {
            opened_at: Some(Utc::now()),
        }),
        side_effects: Some(Box::new(move |document| {
            let duplicate = build_domain_event(
                "document.materialized",
                "document",
                *document.id.as_uuid(),
                user_id,
                serde_json::json!({}),
            );
            MaterializeSideEffects {
                events: vec![duplicate.clone(), duplicate],
                outbox: vec![OutboxEntry {
                    job_type: "document.test_render".into(),
                    payload: serde_json::json!({}),
                    dedupe_key: None,
                    available_at: Utc::now(),
                }],
            }
        })),
    };

    assert!(
        PgDocumentLifecycle::new(pool.clone())
            .materialize_document(request)
            .await
            .is_err()
    );
    assert_eq!(
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM documents WHERE user_id = $1 AND canonical_url = $2",
            user.id.into_uuid(),
            canonical,
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        Some(0)
    );
    assert_eq!(
        sqlx::query_scalar!(
            "SELECT document_id FROM feed_deliveries WHERE id = $1",
            delivery.id.into_uuid()
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        None
    );
    assert_eq!(
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM domain_events WHERE user_id = $1",
            user.id.into_uuid()
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        Some(0)
    );
    assert_eq!(
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM job_outbox WHERE job_type = 'document.test_render'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        Some(0)
    );
}

#[tokio::test]
async fn origin_backed_feed_entries_adopt_deliveries_during_materialize_and_save() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let source = FeedSourceFactory.insert(&pool).await;
    let subscription = FeedSubscriptionFactory::new(user.id)
        .with_source(source.clone())
        .insert(&pool)
        .await;
    let feeds = PgFeedRepository::new(pool.clone());
    let lifecycle = PgDocumentLifecycle::new(pool.clone());

    let entry = feeds
        .create_source_entry(FeedSourceEntry {
            id: FeedSourceEntryId::new(),
            source_id: source.id,
            guid: uuid::Uuid::now_v7().to_string(),
            title: "Origin-only materialize".into(),
            url: None,
            canonical_url: None,
            author: None,
            excerpt: None,
            content_html: None,
            language: None,
            lead_image_url: None,
            published_at: None,
            discovered_at: Utc::now(),
        })
        .await
        .unwrap();
    let delivery = FeedDeliveryFactory::new(user.id, subscription.id, source.id, entry.id)
        .insert(&pool)
        .await;
    let materialized = lifecycle
        .materialize_document(MaterializeRequest {
            identity: feed_entry_identity(user.id, &entry),
            document_state: None,
            side_effects: None,
        })
        .await
        .unwrap();
    assert_eq!(materialized.backlinked_deliveries, 1);
    assert_eq!(
        sqlx::query_scalar!(
            "SELECT document_id FROM feed_deliveries WHERE id = $1",
            delivery.id.into_uuid()
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        Some(materialized.document.id.into_uuid())
    );

    let saved_entry = feeds
        .create_source_entry(FeedSourceEntry {
            id: FeedSourceEntryId::new(),
            source_id: source.id,
            guid: uuid::Uuid::now_v7().to_string(),
            title: "Origin-only save".into(),
            url: None,
            canonical_url: None,
            author: None,
            excerpt: None,
            content_html: None,
            language: None,
            lead_image_url: None,
            published_at: None,
            discovered_at: Utc::now(),
        })
        .await
        .unwrap();
    let saved_delivery =
        FeedDeliveryFactory::new(user.id, subscription.id, source.id, saved_entry.id)
            .insert(&pool)
            .await;
    let saved = lifecycle
        .save_to_library(SaveToLibraryRequest {
            identity: feed_entry_identity(user.id, &saved_entry),
            source: ContentSource::Feed,
            source_delivery_id: Some(saved_delivery.id),
            hide_deliveries: true,
            enqueue_engaged_ai: false,
            restore_policy: Default::default(),
            side_effects: None,
        })
        .await
        .unwrap();
    assert_eq!(
        (saved.backlinked_deliveries, saved.hidden_deliveries),
        (1, 1)
    );
    assert_eq!(saved.entry.document_id, saved.document.id);
}
