use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{TagAliasId, TagId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagSource {
    Manual,
    Ai,
    Import,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: TagId,
    pub user_id: UserId,
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<TagId>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagAlias {
    pub id: TagAliasId,
    pub tag_id: TagId,
    pub alias: String,
}
