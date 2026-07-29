use chrono::{DateTime, Utc};
use uuid::Uuid;

use ind_domain::{Collection, CollectionId, UserId};

#[derive(sqlx::FromRow)]
pub(super) struct CollectionRow {
    pub(super) id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) parent_id: Option<Uuid>,
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) icon: Option<String>,
    pub(super) color: Option<String>,
    pub(super) sort_order: i32,
    pub(super) is_pinned: bool,
    pub(super) rss_token: Option<String>,
    pub(super) created_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
}

impl From<CollectionRow> for Collection {
    fn from(row: CollectionRow) -> Self {
        Collection {
            id: CollectionId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            parent_id: row.parent_id.map(CollectionId::from_uuid),
            name: row.name,
            description: row.description,
            icon: row.icon,
            color: row.color,
            sort_order: row.sort_order,
            is_pinned: row.is_pinned,
            rss_token: row.rss_token,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
pub(super) struct CollectionWithCount {
    pub(super) id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) parent_id: Option<Uuid>,
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) icon: Option<String>,
    pub(super) color: Option<String>,
    pub(super) sort_order: i32,
    pub(super) is_pinned: bool,
    pub(super) rss_token: Option<String>,
    pub(super) created_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
    pub(super) item_count: Option<i64>,
}

impl CollectionWithCount {
    pub(super) fn into_pair(self) -> (Collection, i64) {
        let collection = Collection {
            id: CollectionId::from_uuid(self.id),
            user_id: UserId::from_uuid(self.user_id),
            parent_id: self.parent_id.map(CollectionId::from_uuid),
            name: self.name,
            description: self.description,
            icon: self.icon,
            color: self.color,
            sort_order: self.sort_order,
            is_pinned: self.is_pinned,
            rss_token: self.rss_token,
            created_at: self.created_at,
            updated_at: self.updated_at,
        };
        (collection, self.item_count.unwrap_or(0))
    }
}
