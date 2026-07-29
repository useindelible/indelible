use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ApiError;

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchEmbeddedSenderResponse {
    pub id: String,
    pub canonical_addr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_id: Option<String>,
    pub blocked: bool,
}

impl SearchEmbeddedSenderResponse {
    pub fn from_domain(sender: ind_domain::EmailSender) -> Self {
        Self {
            id: sender.id.to_string(),
            canonical_addr: sender.canonical_addr,
            display_name: sender.display_name,
            list_id: sender.list_id,
            blocked: sender.blocked_at.is_some(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct SearchParams {
    pub q: String,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct SearchSuggestionsParams {
    pub q: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct SearchRecentParams {
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchSectionResponse {
    pub kind: String,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchEntityChipResponse {
    pub entity_id: String,
    pub name: String,
    pub entity_type: String,
    pub mention_count: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchEntityCardResponse {
    pub entity_id: String,
    pub name: String,
    pub entity_type: String,
    pub mention_count: i64,
    #[schema(value_type = String, format = DateTime)]
    pub first_seen_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResultResponse {
    pub result_kind: String,
    /// Set for `document` results (durable, prepared/saved content).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    /// Set for `feed_preview` results: the unprepared delivery; the client opens/prepares the
    /// canonical reader keyed by this id (no document is materialized by searching).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_id: Option<String>,
    /// Set for `feed_preview` results: the source entry behind the delivery (provenance).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_entry_id: Option<String>,
    pub title: String,
    pub snippet: String,
    pub score: f64,
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub saved_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<SearchSectionResponse>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entity_chips: Vec<SearchEntityChipResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<SearchEmbeddedSenderResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResultsResponse {
    pub query: String,
    pub results: Vec<SearchResultResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_card: Option<SearchEntityCardResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchSuggestionResponse {
    pub kind: String,
    pub label: String,
    pub insert_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchSuggestionsResponse {
    pub query: String,
    pub suggestions: Vec<SearchSuggestionResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RecentSearchResponse {
    pub id: String,
    pub query: String,
    pub normalized_query: String,
    #[schema(value_type = String, format = DateTime)]
    pub last_searched_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RecentSearchListResponse {
    pub items: Vec<RecentSearchResponse>,
}

impl SearchResultsResponse {
    pub fn from_domain(page: ind_domain::SearchPage) -> Self {
        Self {
            query: page.query,
            results: page
                .results
                .into_iter()
                .map(SearchResultResponse::from_domain)
                .collect(),
            next_cursor: page.next_cursor,
            has_more: page.has_more,
            entity_card: page.entity_card.map(SearchEntityCardResponse::from_domain),
        }
    }
}

impl SearchResultResponse {
    pub fn from_domain(hit: ind_domain::SearchHit) -> Self {
        Self {
            result_kind: search_result_kind_to_str(hit.result_kind).to_string(),
            document_id: hit.document_id.map(|id| id.to_string()),
            delivery_id: hit.delivery_id.map(|id| id.to_string()),
            source_entry_id: hit.source_entry_id.map(|id| id.to_string()),
            title: hit.title,
            snippet: hit.snippet,
            score: hit.score,
            content_type: item_type_to_str(hit.content_type).to_string(),
            url: hit.url,
            saved_at: hit.saved_at,
            updated_at: hit.updated_at,
            section: hit.section.map(|section| SearchSectionResponse {
                kind: search_section_kind_to_str(section.kind).to_string(),
                key: section.key,
                title: section.title,
            }),
            entity_chips: hit
                .entity_chips
                .into_iter()
                .map(SearchEntityChipResponse::from_domain)
                .collect(),
            sender_id: hit.sender_id.map(|id| id.to_string()),
            sender: None,
        }
    }

    pub fn attach_sender(&mut self, sender: ind_domain::EmailSender) {
        self.sender = Some(SearchEmbeddedSenderResponse::from_domain(sender));
    }
}

impl SearchEntityChipResponse {
    pub fn from_domain(chip: ind_domain::SearchEntityChip) -> Self {
        Self {
            entity_id: chip.entity_id.to_string(),
            name: chip.name,
            entity_type: entity_type_to_str(chip.entity_type).to_string(),
            mention_count: chip.mention_count,
        }
    }
}

impl SearchEntityCardResponse {
    pub fn from_domain(card: ind_domain::SearchEntityCard) -> Self {
        Self {
            entity_id: card.entity_id.to_string(),
            name: card.name,
            entity_type: entity_type_to_str(card.entity_type).to_string(),
            mention_count: card.mention_count,
            first_seen_at: card.first_seen_at,
            last_seen_at: card.last_seen_at,
        }
    }
}

impl SearchSuggestionsResponse {
    pub fn from_domain(query: String, suggestions: Vec<ind_domain::SearchSuggestion>) -> Self {
        Self {
            query,
            suggestions: suggestions
                .into_iter()
                .map(SearchSuggestionResponse::from_domain)
                .collect(),
        }
    }
}

impl SearchSuggestionResponse {
    pub fn from_domain(suggestion: ind_domain::SearchSuggestion) -> Self {
        Self {
            kind: search_suggestion_kind_to_str(suggestion.kind).to_string(),
            label: suggestion.label,
            insert_text: suggestion.insert_text,
            description: suggestion.description,
        }
    }
}

impl RecentSearchListResponse {
    pub fn from_domain(items: Vec<ind_domain::RecentSearch>) -> Self {
        Self {
            items: items
                .into_iter()
                .map(RecentSearchResponse::from_domain)
                .collect(),
        }
    }
}

impl RecentSearchResponse {
    pub fn from_domain(search: ind_domain::RecentSearch) -> Self {
        Self {
            id: search.id.to_string(),
            query: search.raw_query,
            normalized_query: search.normalized_query,
            last_searched_at: search.last_searched_at,
            created_at: search.created_at,
            updated_at: search.updated_at,
        }
    }
}

pub(crate) fn parse_recent_search_id(s: &str) -> Result<ind_domain::RecentSearchId, ApiError> {
    s.parse().map_err(|_| ApiError::NotFound {
        entity: "RecentSearch",
        id: s.to_string(),
    })
}

fn item_type_to_str(value: ind_domain::ItemType) -> &'static str {
    value.as_str()
}

fn search_section_kind_to_str(value: ind_domain::SearchSectionKind) -> &'static str {
    match value {
        ind_domain::SearchSectionKind::Item => "item",
        ind_domain::SearchSectionKind::EpubChapter => "epub_chapter",
    }
}

fn search_result_kind_to_str(value: ind_domain::SearchResultKind) -> &'static str {
    match value {
        ind_domain::SearchResultKind::Document => "document",
        ind_domain::SearchResultKind::FeedPreview => "feed_preview",
    }
}

fn search_suggestion_kind_to_str(value: ind_domain::SearchSuggestionKind) -> &'static str {
    match value {
        ind_domain::SearchSuggestionKind::Filter => "filter",
        ind_domain::SearchSuggestionKind::Tag => "tag",
        ind_domain::SearchSuggestionKind::Collection => "collection",
        ind_domain::SearchSuggestionKind::Recent => "recent",
        ind_domain::SearchSuggestionKind::Entity => "entity",
        ind_domain::SearchSuggestionKind::Sender => "sender",
        ind_domain::SearchSuggestionKind::Author => "author",
        ind_domain::SearchSuggestionKind::List => "list",
    }
}

fn entity_type_to_str(value: ind_domain::EntityType) -> &'static str {
    match value {
        ind_domain::EntityType::Person => "person",
        ind_domain::EntityType::Organization => "organization",
        ind_domain::EntityType::Location => "location",
        ind_domain::EntityType::Event => "event",
        ind_domain::EntityType::Work => "work",
    }
}
