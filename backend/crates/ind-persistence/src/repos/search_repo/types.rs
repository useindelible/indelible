use chrono::{DateTime, Utc};
use uuid::Uuid;

use ind_application::AppError;
use ind_domain::{
    DocumentId, EntityId, EntityType, FeedDeliveryId, FeedSourceEntryId, HighlightId, ItemType,
    RecentSearch, RecentSearchId, SearchDocument, SearchDocumentId, SearchDocumentKind,
    SearchDocumentSource, SearchEntityCard, SearchEntityChip, SearchHit, SearchIndexedHighlight,
    SearchResultKind, SearchSectionKind, SearchSectionRef, UserId,
};

#[derive(sqlx::FromRow)]
pub(super) struct SearchDocumentRow {
    pub(super) id: Uuid,
    pub(super) document_id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) document_kind: String,
    pub(super) section_key: String,
    pub(super) section_title: Option<String>,
    pub(super) title: String,
    pub(super) body_text: String,
    pub(super) highlight_text: String,
    pub(super) metadata_text: String,
    pub(super) search_config: String,
    pub(super) saved_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub(super) struct SearchHitRow {
    pub(super) document_id: Option<Uuid>,
    pub(super) delivery_id: Option<Uuid>,
    pub(super) source_entry_id: Option<Uuid>,
    pub(super) result_kind: String,
    pub(super) item_title: String,
    pub(super) snippet: String,
    pub(super) final_score: f64,
    pub(super) item_type: String,
    pub(super) url: Option<String>,
    pub(super) saved_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
    pub(super) section_kind: Option<String>,
    pub(super) section_key: Option<String>,
    pub(super) section_title: Option<String>,
    pub(super) sender_id: Option<Uuid>,
}

#[derive(sqlx::FromRow)]
pub(super) struct RecentSearchRow {
    pub(super) id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) raw_query: String,
    pub(super) normalized_query: String,
    pub(super) last_searched_at: DateTime<Utc>,
    pub(super) created_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub(super) struct SearchIndexedHighlightRow {
    pub(super) highlight_id: Uuid,
    pub(super) text: String,
    pub(super) note: Option<String>,
    pub(super) section_key: Option<String>,
}

#[derive(sqlx::FromRow)]
pub(super) struct SearchEntityChipRow {
    pub(super) document_id: Uuid,
    pub(super) entity_id: Uuid,
    pub(super) name: String,
    pub(super) entity_type: String,
    pub(super) mention_count: i32,
}

#[derive(sqlx::FromRow)]
pub(super) struct EntitySuggestionRow {
    pub(super) entity_id: Uuid,
    pub(super) name: String,
    pub(super) entity_type: String,
    pub(super) mention_count: i64,
}

#[derive(sqlx::FromRow)]
pub(super) struct SearchEntityCardRow {
    pub(super) entity_id: Uuid,
    pub(super) name: String,
    pub(super) entity_type: String,
    pub(super) mention_count: i64,
    pub(super) first_seen_at: DateTime<Utc>,
    pub(super) last_seen_at: DateTime<Utc>,
}

pub(super) fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::super::map_sqlx_error("search", "search conflict", err)
}

pub(super) fn parse_search_document_kind(value: &str) -> Result<SearchDocumentKind, AppError> {
    match value {
        "item" => Ok(SearchDocumentKind::Item),
        "epub_chapter" => Ok(SearchDocumentKind::EpubChapter),
        other => Err(AppError::Domain(
            ind_domain::DomainError::InvariantViolation {
                message: format!("invalid search document kind: {other}"),
            },
        )),
    }
}

pub(super) fn search_document_kind_to_str(value: SearchDocumentKind) -> &'static str {
    match value {
        SearchDocumentKind::Item => "item",
        SearchDocumentKind::EpubChapter => "epub_chapter",
    }
}

pub(super) fn parse_search_section_kind(value: &str) -> Result<SearchSectionKind, AppError> {
    match value {
        "item" => Ok(SearchSectionKind::Item),
        "epub_chapter" => Ok(SearchSectionKind::EpubChapter),
        other => Err(AppError::Domain(
            ind_domain::DomainError::InvariantViolation {
                message: format!("invalid search section kind: {other}"),
            },
        )),
    }
}

pub(super) fn parse_search_result_kind(value: &str) -> Result<SearchResultKind, AppError> {
    match value {
        "document" => Ok(SearchResultKind::Document),
        "feed_preview" => Ok(SearchResultKind::FeedPreview),
        other => Err(AppError::Domain(
            ind_domain::DomainError::InvariantViolation {
                message: format!("invalid search result kind: {other}"),
            },
        )),
    }
}

pub(super) fn parse_entity_type(value: &str) -> Result<EntityType, AppError> {
    match value {
        "person" => Ok(EntityType::Person),
        "organization" => Ok(EntityType::Organization),
        "location" => Ok(EntityType::Location),
        "event" => Ok(EntityType::Event),
        "work" => Ok(EntityType::Work),
        other => Err(AppError::Domain(
            ind_domain::DomainError::InvariantViolation {
                message: format!("invalid entity type: {other}"),
            },
        )),
    }
}

pub(super) fn parse_item_type(value: &str) -> Result<ItemType, AppError> {
    value.parse::<ItemType>().map_err(|_| {
        AppError::Domain(ind_domain::DomainError::InvariantViolation {
            message: format!("invalid item type: {value}"),
        })
    })
}

impl TryFrom<SearchDocumentRow> for SearchDocument {
    type Error = AppError;

    fn try_from(row: SearchDocumentRow) -> Result<Self, Self::Error> {
        let source = SearchDocumentSource::Document {
            document_id: DocumentId::from_uuid(row.document_id),
        };
        Ok(Self {
            id: SearchDocumentId::from_uuid(row.id),
            source,
            user_id: UserId::from_uuid(row.user_id),
            document_kind: parse_search_document_kind(&row.document_kind)?,
            section_key: row.section_key,
            section_title: row.section_title,
            title: row.title,
            body_text: row.body_text,
            highlight_text: row.highlight_text,
            metadata_text: row.metadata_text,
            search_config: row.search_config,
            saved_at: row.saved_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<SearchHitRow> for SearchHit {
    type Error = AppError;

    fn try_from(row: SearchHitRow) -> Result<Self, Self::Error> {
        let result_kind = parse_search_result_kind(&row.result_kind)?;
        Ok(Self {
            source_chunk_id: None,
            result_kind,
            document_id: row.document_id.map(DocumentId::from_uuid),
            delivery_id: row.delivery_id.map(FeedDeliveryId::from_uuid),
            source_entry_id: row.source_entry_id.map(FeedSourceEntryId::from_uuid),
            title: row.item_title,
            snippet: row.snippet,
            score: row.final_score,
            content_type: parse_item_type(&row.item_type)?,
            url: row.url,
            saved_at: row.saved_at,
            updated_at: row.updated_at,
            section: match (row.section_kind, row.section_key) {
                (Some(kind), Some(key)) => Some(SearchSectionRef {
                    kind: parse_search_section_kind(&kind)?,
                    key,
                    title: row.section_title,
                }),
                _ => None,
            },
            entity_chips: Vec::new(),
            sender_id: row.sender_id.map(ind_domain::EmailSenderId::from_uuid),
        })
    }
}

impl TryFrom<SearchEntityChipRow> for SearchEntityChip {
    type Error = AppError;

    fn try_from(row: SearchEntityChipRow) -> Result<Self, Self::Error> {
        Ok(Self {
            entity_id: EntityId::from_uuid(row.entity_id),
            name: row.name,
            entity_type: parse_entity_type(&row.entity_type)?,
            mention_count: row.mention_count,
        })
    }
}

impl TryFrom<EntitySuggestionRow> for SearchEntityChip {
    type Error = AppError;

    fn try_from(row: EntitySuggestionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            entity_id: EntityId::from_uuid(row.entity_id),
            name: row.name,
            entity_type: parse_entity_type(&row.entity_type)?,
            mention_count: row.mention_count.clamp(0, i64::from(i32::MAX)) as i32,
        })
    }
}

impl TryFrom<SearchEntityCardRow> for SearchEntityCard {
    type Error = AppError;

    fn try_from(row: SearchEntityCardRow) -> Result<Self, Self::Error> {
        Ok(Self {
            entity_id: EntityId::from_uuid(row.entity_id),
            name: row.name,
            entity_type: parse_entity_type(&row.entity_type)?,
            mention_count: row.mention_count,
            first_seen_at: row.first_seen_at,
            last_seen_at: row.last_seen_at,
        })
    }
}

impl From<RecentSearchRow> for RecentSearch {
    fn from(row: RecentSearchRow) -> Self {
        Self {
            id: RecentSearchId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            raw_query: row.raw_query,
            normalized_query: row.normalized_query,
            last_searched_at: row.last_searched_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<SearchIndexedHighlightRow> for SearchIndexedHighlight {
    fn from(row: SearchIndexedHighlightRow) -> Self {
        Self {
            highlight_id: HighlightId::from_uuid(row.highlight_id),
            text: row.text,
            note: row.note,
            section_key: row.section_key,
        }
    }
}
