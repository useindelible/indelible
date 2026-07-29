use chrono::{DateTime, Utc};
use uuid::Uuid;

use ind_domain::{Tag, TagId, UserId};

#[derive(sqlx::FromRow)]
pub(super) struct TagRow {
    pub(super) id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) name: String,
    pub(super) color: Option<String>,
    pub(super) parent_id: Option<Uuid>,
    pub(super) created_at: DateTime<Utc>,
}

impl From<TagRow> for Tag {
    fn from(row: TagRow) -> Self {
        Tag {
            id: TagId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            name: row.name,
            color: row.color,
            parent_id: row.parent_id.map(TagId::from_uuid),
            created_at: row.created_at,
        }
    }
}
