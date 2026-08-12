#![allow(clippy::unwrap_used)]

use chrono::{DateTime, TimeZone, Utc};
use ind_application::repos::{feed::FeedRepository, feed_delivery::FeedDeliveryRepository};
use ind_domain::{
    FeedDelivery, FeedDeliveryId, FeedDeliveryState, FeedSourceEntry, FeedSourceEntryId,
    FeedSourceId, FeedSubscriptionId, UserId,
};
use ind_persistence::repos::{PgFeedDeliveryRepository, PgFeedRepository};
use ind_test_support::{FeedSourceFactory, FeedSubscriptionFactory, TestDb, UserFactory};

fn timestamp(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 12, hour, minute, 0).unwrap()
}

struct DeliverySeed<'a> {
    ordinal: u128,
    title: &'a str,
    published_at: Option<DateTime<Utc>>,
    delivered_at: DateTime<Utc>,
}

async fn insert_delivery(
    feeds: &PgFeedRepository,
    deliveries: &PgFeedDeliveryRepository,
    user_id: UserId,
    subscription_id: FeedSubscriptionId,
    source_id: FeedSourceId,
    seed: DeliverySeed<'_>,
) -> FeedDeliveryId {
    let DeliverySeed {
        ordinal,
        title,
        published_at,
        delivered_at,
    } = seed;
    let entry = feeds
        .create_source_entry(FeedSourceEntry {
            id: FeedSourceEntryId::new(),
            source_id,
            guid: format!("entry-{ordinal}"),
            title: title.into(),
            url: Some(format!("https://example.com/{ordinal}")),
            canonical_url: Some(format!("https://example.com/{ordinal}")),
            author: None,
            excerpt: None,
            content_html: None,
            language: None,
            lead_image_url: None,
            published_at,
            discovered_at: delivered_at,
        })
        .await
        .unwrap();
    let id = FeedDeliveryId::from_uuid(uuid::Uuid::from_u128(ordinal));

    deliveries
        .upsert_delivery(FeedDelivery {
            id,
            user_id,
            subscription_id,
            source_id,
            source_entry_id: entry.id,
            document_id: None,
            delivered_at,
            seen_at: None,
            dismissed_at: None,
            hidden_at: None,
            created_at: delivered_at,
            updated_at: delivered_at,
        })
        .await
        .unwrap();

    id
}

async fn setup() -> (
    TestDb,
    UserId,
    FeedSourceId,
    FeedSubscriptionId,
    PgFeedRepository,
    PgFeedDeliveryRepository,
) {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let source = FeedSourceFactory.insert(&pool).await;
    let subscription = FeedSubscriptionFactory::new(user.id)
        .with_source(source.clone())
        .insert(&pool)
        .await;

    (
        db,
        user.id,
        source.id,
        subscription.id,
        PgFeedRepository::new(pool.clone()),
        PgFeedDeliveryRepository::new(pool),
    )
}

#[tokio::test]
async fn unseen_cursor_pages_follow_entry_publication_order_without_duplicates() {
    let (_db, user_id, source_id, subscription_id, feeds, deliveries) = setup().await;
    let delivered_at = timestamp(13, 0);
    let newest = insert_delivery(
        &feeds,
        &deliveries,
        user_id,
        subscription_id,
        source_id,
        DeliverySeed {
            ordinal: 1,
            title: "Newest",
            published_at: Some(timestamp(12, 0)),
            delivered_at,
        },
    )
    .await;
    let middle = insert_delivery(
        &feeds,
        &deliveries,
        user_id,
        subscription_id,
        source_id,
        DeliverySeed {
            ordinal: 2,
            title: "Middle",
            published_at: Some(timestamp(11, 0)),
            delivered_at,
        },
    )
    .await;
    let oldest = insert_delivery(
        &feeds,
        &deliveries,
        user_id,
        subscription_id,
        source_id,
        DeliverySeed {
            ordinal: 3,
            title: "Oldest",
            published_at: Some(timestamp(10, 0)),
            delivered_at,
        },
    )
    .await;

    let first = deliveries
        .list_deliveries(user_id, FeedDeliveryState::Unseen, None, None, 2)
        .await
        .unwrap();
    assert_eq!(
        first
            .items
            .iter()
            .map(|item| item.delivery.id)
            .collect::<Vec<_>>(),
        vec![newest, middle]
    );

    let second = deliveries
        .list_deliveries(
            user_id,
            FeedDeliveryState::Unseen,
            None,
            first.next_cursor,
            2,
        )
        .await
        .unwrap();
    assert_eq!(
        second
            .items
            .iter()
            .map(|item| item.delivery.id)
            .collect::<Vec<_>>(),
        vec![oldest]
    );
    assert!(second.next_cursor.is_none());
}

#[tokio::test]
async fn unseen_cursor_pages_break_equal_timestamps_by_delivery_id_without_overlap() {
    let (_db, user_id, source_id, subscription_id, feeds, deliveries) = setup().await;
    let published_at = timestamp(12, 0);
    let delivered_at = timestamp(13, 0);
    let first_id = insert_delivery(
        &feeds,
        &deliveries,
        user_id,
        subscription_id,
        source_id,
        DeliverySeed {
            ordinal: 30,
            title: "Lowest ID",
            published_at: Some(published_at),
            delivered_at,
        },
    )
    .await;
    let second_id = insert_delivery(
        &feeds,
        &deliveries,
        user_id,
        subscription_id,
        source_id,
        DeliverySeed {
            ordinal: 31,
            title: "Middle ID",
            published_at: Some(published_at),
            delivered_at,
        },
    )
    .await;
    let third_id = insert_delivery(
        &feeds,
        &deliveries,
        user_id,
        subscription_id,
        source_id,
        DeliverySeed {
            ordinal: 32,
            title: "Highest ID",
            published_at: Some(published_at),
            delivered_at,
        },
    )
    .await;

    let first = deliveries
        .list_deliveries(user_id, FeedDeliveryState::Unseen, None, None, 2)
        .await
        .unwrap();
    assert_eq!(
        first
            .items
            .iter()
            .map(|item| item.delivery.id)
            .collect::<Vec<_>>(),
        vec![third_id, second_id]
    );

    let second = deliveries
        .list_deliveries(
            user_id,
            FeedDeliveryState::Unseen,
            None,
            first.next_cursor,
            2,
        )
        .await
        .unwrap();
    assert_eq!(
        second
            .items
            .iter()
            .map(|item| item.delivery.id)
            .collect::<Vec<_>>(),
        vec![first_id]
    );
    assert!(first.items.iter().all(|first_item| {
        second
            .items
            .iter()
            .all(|second_item| second_item.delivery.id != first_item.delivery.id)
    }));
    assert!(second.next_cursor.is_none());
}

#[tokio::test]
async fn unseen_order_falls_back_to_delivery_time_without_a_publication_time() {
    let (_db, user_id, source_id, subscription_id, feeds, deliveries) = setup().await;
    let published = insert_delivery(
        &feeds,
        &deliveries,
        user_id,
        subscription_id,
        source_id,
        DeliverySeed {
            ordinal: 10,
            title: "Published earlier",
            published_at: Some(timestamp(10, 0)),
            delivered_at: timestamp(13, 0),
        },
    )
    .await;
    let unpublished = insert_delivery(
        &feeds,
        &deliveries,
        user_id,
        subscription_id,
        source_id,
        DeliverySeed {
            ordinal: 11,
            title: "No publication date",
            published_at: None,
            delivered_at: timestamp(12, 0),
        },
    )
    .await;

    let page = deliveries
        .list_deliveries(user_id, FeedDeliveryState::Unseen, None, None, 10)
        .await
        .unwrap();

    assert_eq!(
        page.items
            .iter()
            .map(|item| item.delivery.id)
            .collect::<Vec<_>>(),
        vec![unpublished, published]
    );
}

#[tokio::test]
async fn prefetch_candidates_follow_effective_publication_order() {
    let (_db, user_id, source_id, subscription_id, feeds, deliveries) = setup().await;
    let newest = insert_delivery(
        &feeds,
        &deliveries,
        user_id,
        subscription_id,
        source_id,
        DeliverySeed {
            ordinal: 20,
            title: "Newest publication",
            published_at: Some(timestamp(12, 0)),
            delivered_at: timestamp(9, 0),
        },
    )
    .await;
    let unpublished = insert_delivery(
        &feeds,
        &deliveries,
        user_id,
        subscription_id,
        source_id,
        DeliverySeed {
            ordinal: 21,
            title: "No publication date",
            published_at: None,
            delivered_at: timestamp(11, 30),
        },
    )
    .await;
    let oldest = insert_delivery(
        &feeds,
        &deliveries,
        user_id,
        subscription_id,
        source_id,
        DeliverySeed {
            ordinal: 22,
            title: "Oldest publication",
            published_at: Some(timestamp(11, 0)),
            delivered_at: timestamp(13, 0),
        },
    )
    .await;

    let candidates = deliveries
        .list_prefetch_candidates(user_id, Some(subscription_id), 30, 10)
        .await
        .unwrap();

    assert_eq!(
        candidates
            .iter()
            .map(|delivery| delivery.id)
            .collect::<Vec<_>>(),
        vec![newest, unpublished, oldest]
    );
}
