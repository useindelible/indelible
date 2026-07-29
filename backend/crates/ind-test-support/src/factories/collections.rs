use super::prelude::*;

pub struct CollectionFactory {
    user_id: UserId,
}

impl CollectionFactory {
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }

    pub async fn insert(self, pool: &sqlx::PgPool) -> Collection {
        let timestamp = Utc::now();
        PgCollectionRepository::new(pool.clone())
            .create(Collection {
                id: CollectionId::new(),
                user_id: self.user_id,
                parent_id: None,
                name: format!("collection-{}", short_unique_suffix()),
                description: None,
                icon: None,
                color: None,
                sort_order: 0,
                is_pinned: false,
                rss_token: None,
                created_at: timestamp,
                updated_at: timestamp,
            })
            .await
            .expect("CollectionFactory::insert failed")
    }
}
