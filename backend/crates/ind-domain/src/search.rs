use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    ContentVectorId, DocumentId, EntityId, FeedDeliveryId, FeedSourceEntryId, HighlightId,
    ItemType, RecentSearchId, SearchDocumentId, UserId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Event,
    Work,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchDocumentKind {
    Item,
    EpubChapter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchSectionKind {
    Item,
    EpubChapter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchSuggestionKind {
    Filter,
    Tag,
    Collection,
    Recent,
    Entity,
    Sender,
    Author,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchStatusFilter {
    Read,
    Unread,
    Archived,
    Favorited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchHasFilter {
    Highlights,
    Notes,
    Unsubscribe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchSourceFilter {
    Feed,
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchResultKind {
    /// Durable, document-keyed result (prepared/saved/engaged content).
    Document,
    /// Unprepared feed delivery surfaced from the discovery query (feed_deliveries +
    /// feed_source_entries); has no document and is never materialized by search.
    FeedPreview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SearchFilter {
    Tag {
        value: String,
        negated: bool,
    },
    Collection {
        value: String,
        negated: bool,
    },
    ContentType {
        value: String,
        negated: bool,
    },
    Author {
        value: String,
        negated: bool,
    },
    Before {
        value: NaiveDate,
    },
    After {
        value: NaiveDate,
    },
    Status {
        value: SearchStatusFilter,
        negated: bool,
    },
    Has {
        value: SearchHasFilter,
        negated: bool,
    },
    Url {
        value: String,
        negated: bool,
    },
    Entity {
        value: String,
        negated: bool,
    },
    Pinned {
        value: bool,
        negated: bool,
    },
    Sender {
        value: String,
        negated: bool,
    },
    SenderDomain {
        value: String,
        negated: bool,
    },
    ListId {
        value: String,
        negated: bool,
    },
    Subject {
        value: String,
        negated: bool,
    },
    SenderBlocked {
        negated: bool,
    },
    Source {
        value: SearchSourceFilter,
        negated: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    pub id: SearchDocumentId,
    pub source: SearchDocumentSource,
    pub user_id: UserId,
    pub document_kind: SearchDocumentKind,
    pub section_key: String,
    pub section_title: Option<String>,
    pub title: String,
    pub body_text: String,
    pub highlight_text: String,
    pub metadata_text: String,
    pub search_config: String,
    pub saved_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SearchDocumentSource {
    Document { document_id: DocumentId },
}

impl SearchDocument {
    pub fn document_id(&self) -> DocumentId {
        match self.source {
            SearchDocumentSource::Document { document_id } => document_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentVector {
    pub id: ContentVectorId,
    pub document_id: DocumentId,
    pub user_id: UserId,
    pub embedding_model: String,
    pub embedding_dim: i32,
    pub section_kind: SearchSectionKind,
    pub section_key: String,
    pub chunk_index: i32,
    pub content: String,
    pub token_count: i32,
    pub search_config: String,
    pub embedding: Vec<f32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSectionRef {
    pub kind: SearchSectionKind,
    pub key: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub source_chunk_id: Option<ContentVectorId>,
    pub result_kind: SearchResultKind,
    /// Set for `Document` hits (durable document-keyed results).
    pub document_id: Option<DocumentId>,
    /// Set for `FeedPreview` hits (the discovery delivery row); the open/select action keys
    /// preparation off this delivery.
    pub delivery_id: Option<FeedDeliveryId>,
    /// Set for `FeedPreview` hits (the underlying source entry).
    pub source_entry_id: Option<FeedSourceEntryId>,
    pub title: String,
    pub snippet: String,
    pub score: f64,
    pub content_type: ItemType,
    pub url: Option<String>,
    pub saved_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub section: Option<SearchSectionRef>,
    pub entity_chips: Vec<SearchEntityChip>,
    pub sender_id: Option<crate::EmailSenderId>,
}

impl SearchHit {
    #[expect(
        clippy::expect_used,
        reason = "internal invariant: a Document hit always carries document_id and a FeedPreview hit always carries delivery_id"
    )]
    pub fn result_id_uuid(&self) -> uuid::Uuid {
        match self.result_kind {
            SearchResultKind::Document => self
                .document_id
                .map(|id| id.into_uuid())
                .expect("document hit must carry document_id"),
            SearchResultKind::FeedPreview => self
                .delivery_id
                .map(|id| id.into_uuid())
                .expect("feed_preview hit must carry delivery_id"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchCursor {
    pub score: f64,
    #[serde(default = "default_score_reference_at")]
    pub score_reference_at: DateTime<Utc>,
    pub saved_at: DateTime<Utc>,
    pub result_id: uuid::Uuid,
    pub section_key: String,
}

fn default_score_reference_at() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPage {
    pub query: String,
    pub results: Vec<SearchHit>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub entity_card: Option<SearchEntityCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestion {
    pub kind: SearchSuggestionKind,
    pub label: String,
    pub insert_text: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSearchQuery {
    pub raw_query: String,
    pub text_query: Option<String>,
    pub filters: Vec<SearchFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentSearch {
    pub id: RecentSearchId,
    pub user_id: UserId,
    pub raw_query: String,
    pub normalized_query: String,
    pub last_searched_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndexedHighlight {
    pub highlight_id: HighlightId,
    pub text: String,
    pub note: Option<String>,
    pub section_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRateLimitStatus {
    pub allowed: bool,
    pub quota_name: String,
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
    pub retry_after_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntityChip {
    pub entity_id: EntityId,
    pub name: String,
    pub entity_type: EntityType,
    pub mention_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntityCard {
    pub entity_id: EntityId,
    pub name: String,
    pub entity_type: EntityType,
    pub mention_count: i64,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub user_id: UserId,
    pub name: String,
    pub entity_type: EntityType,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySummary {
    pub entity: Entity,
    pub total_mentions: i64,
    pub item_count: i64,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCoOccurrence {
    pub entity: Entity,
    pub shared_item_count: i64,
    pub total_mentions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDetail {
    pub entity: Entity,
    pub total_mentions: i64,
    pub item_count: i64,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub co_occurring: Vec<EntityCoOccurrence>,
}

#[cfg(test)]
mod tests;
