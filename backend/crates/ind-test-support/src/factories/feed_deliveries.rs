use super::prelude::*;

pub struct FeedDeliveryFactory(FeedDelivery);

impl FeedDeliveryFactory {
    pub fn new(
        user_id: UserId,
        subscription_id: FeedSubscriptionId,
        source_id: FeedSourceId,
        source_entry_id: FeedSourceEntryId,
    ) -> Self {
        let timestamp = Utc::now();
        Self(FeedDelivery {
            id: FeedDeliveryId::new(),
            user_id,
            subscription_id,
            source_id,
            source_entry_id,
            document_id: None,
            delivered_at: timestamp,
            seen_at: None,
            dismissed_at: None,
            hidden_at: None,
            created_at: timestamp,
            updated_at: timestamp,
        })
    }

    pub async fn insert(self, pool: &sqlx::PgPool) -> FeedDelivery {
        PgFeedDeliveryRepository::new(pool.clone())
            .upsert_delivery(self.0)
            .await
            .map(|result| result.delivery)
            .expect("FeedDeliveryFactory::insert failed")
    }
}
