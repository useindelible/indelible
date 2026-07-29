use chrono::{DateTime, Utc};
use uuid::Uuid;

use ind_application::AppError;
use ind_domain::{
    ContentVector, ContentVectorId, DocumentId, ItemType, SearchHit, SearchResultKind,
    SearchSectionKind, SearchSectionRef, UserId,
};

#[derive(sqlx::FromRow)]
pub(super) struct ContentVectorRow {
    pub(super) id: Uuid,
    pub(super) document_id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) embedding_model: String,
    pub(super) embedding_dim: i32,
    pub(super) section_kind: String,
    pub(super) section_key: String,
    pub(super) chunk_index: i32,
    pub(super) content: String,
    pub(super) token_count: i32,
    pub(super) search_config: String,
    pub(super) embedding: String,
    pub(super) created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub(super) struct SearchHitRow {
    pub(super) chunk_id: Uuid,
    pub(super) document_id: Uuid,
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
}

#[derive(sqlx::FromRow)]
pub(super) struct FtsHitRow {
    pub(super) chunk_id: Uuid,
    pub(super) coarse_fallback: bool,
    pub(super) document_id: Uuid,
    pub(super) item_title: String,
    pub(super) snippet: String,
    pub(super) fts_rank: f64,
    pub(super) item_type: String,
    pub(super) url: Option<String>,
    pub(super) saved_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
    pub(super) section_kind: Option<String>,
    pub(super) section_key: Option<String>,
    pub(super) section_title: Option<String>,
}

pub(super) struct SourceRefRow {
    pub(super) chunk_id: Uuid,
    pub(super) document_id: Uuid,
    pub(super) title: String,
}

fn strip_headline_markers(text: &str) -> String {
    text.replace("<<", "").replace(">>", "")
}

pub(super) fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::super::map_sqlx_error("content_vector", "duplicate content vector chunk", err)
}

pub(super) fn build_vector_literal(values: &[f32]) -> String {
    let joined = values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn parse_vector_literal(value: &str) -> Vec<f32> {
    value
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .filter_map(|segment| {
            let trimmed = segment.trim();
            (!trimmed.is_empty())
                .then(|| trimmed.parse::<f32>().ok())
                .flatten()
        })
        .collect()
}

fn parse_item_type(value: &str) -> Result<ItemType, AppError> {
    value.parse::<ItemType>().map_err(|_| {
        AppError::Domain(ind_domain::DomainError::InvariantViolation {
            message: format!("invalid item type: {value}"),
        })
    })
}

fn parse_search_section_kind(value: &str) -> Result<SearchSectionKind, AppError> {
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

pub(super) fn search_section_kind_to_str(value: SearchSectionKind) -> &'static str {
    match value {
        SearchSectionKind::Item => "item",
        SearchSectionKind::EpubChapter => "epub_chapter",
    }
}

impl TryFrom<ContentVectorRow> for ContentVector {
    type Error = AppError;

    fn try_from(row: ContentVectorRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ContentVectorId::from_uuid(row.id),
            document_id: DocumentId::from_uuid(row.document_id),
            user_id: UserId::from_uuid(row.user_id),
            embedding_model: row.embedding_model,
            embedding_dim: row.embedding_dim,
            section_kind: parse_search_section_kind(&row.section_kind)?,
            section_key: row.section_key,
            chunk_index: row.chunk_index,
            content: row.content,
            token_count: row.token_count,
            search_config: row.search_config,
            embedding: parse_vector_literal(&row.embedding),
            created_at: row.created_at,
        })
    }
}

impl TryFrom<SearchHitRow> for SearchHit {
    type Error = AppError;

    fn try_from(row: SearchHitRow) -> Result<Self, Self::Error> {
        Ok(Self {
            source_chunk_id: Some(ContentVectorId::from_uuid(row.chunk_id)),
            result_kind: SearchResultKind::Document,
            document_id: Some(DocumentId::from_uuid(row.document_id)),
            delivery_id: None,
            source_entry_id: None,
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
            sender_id: None,
        })
    }
}

impl TryFrom<FtsHitRow> for SearchHit {
    type Error = AppError;

    fn try_from(row: FtsHitRow) -> Result<Self, Self::Error> {
        let snippet = if row.coarse_fallback {
            strip_headline_markers(&row.snippet)
        } else {
            row.snippet
        };
        Ok(Self {
            source_chunk_id: (!row.coarse_fallback)
                .then(|| ContentVectorId::from_uuid(row.chunk_id)),
            result_kind: SearchResultKind::Document,
            document_id: Some(DocumentId::from_uuid(row.document_id)),
            delivery_id: None,
            source_entry_id: None,
            title: row.item_title,
            snippet,
            score: row.fts_rank,
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
            sender_id: None,
        })
    }
}
