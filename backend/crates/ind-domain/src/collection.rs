use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{CollectionId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: CollectionId,
    pub user_id: UserId,
    pub parent_id: Option<CollectionId>,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub is_pinned: bool,
    pub rss_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
