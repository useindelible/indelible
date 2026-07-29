use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{ApiError, FieldError};
use crate::extract::Validate;
use ind_application::repos::entity::EntityDocument;
use ind_domain::{EntityCoOccurrence, EntityDetail, EntityId, EntitySummary, EntityType};

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListEntitiesParams {
    pub r#type: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListEntityDocumentsParams {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEntityBody {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[schema(value_type = Option<String>, nullable)]
    pub description: Option<Option<String>>,
}

impl Validate for UpdateEntityBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if let Some(ref name) = self.name
            && name.trim().is_empty()
        {
            errors.push(FieldError {
                field: "name".into(),
                message: "must not be empty".into(),
            });
        }
        if self.name.is_none() && self.description.is_none() {
            errors.push(FieldError {
                field: "_".into(),
                message: "at least one field must be provided".into(),
            });
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MergeEntityBody {
    pub target_id: String,
}

impl Validate for MergeEntityBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        if parse_entity_id(&self.target_id).is_err() {
            Err(vec![FieldError {
                field: "target_id".into(),
                message: "invalid entity ID".into(),
            }])
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EntitySummaryResponse {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    pub entity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub total_mentions: i64,
    pub item_count: i64,
    #[schema(value_type = String, format = DateTime)]
    pub first_seen_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub last_seen_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl EntitySummaryResponse {
    pub fn from_domain(summary: EntitySummary) -> Self {
        Self {
            id: summary.entity.id.to_string(),
            object: "entity",
            name: summary.entity.name,
            entity_type: entity_type_to_api(summary.entity.entity_type).into(),
            description: summary.entity.description,
            total_mentions: summary.total_mentions,
            item_count: summary.item_count,
            first_seen_at: summary.first_seen_at,
            last_seen_at: summary.last_seen_at,
            created_at: summary.entity.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EntityDocumentResponse {
    pub id: String,
    pub object: &'static str,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub saved_at: DateTime<Utc>,
}

impl EntityDocumentResponse {
    pub fn from_domain(document: EntityDocument) -> Self {
        Self {
            id: document.document_id.to_string(),
            object: "document",
            title: document.title,
            author: document.author,
            excerpt: document.excerpt,
            domain: document.domain,
            saved_at: document.saved_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EntityCoOccurrenceResponse {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    pub entity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub shared_item_count: i64,
    pub total_mentions: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl EntityCoOccurrenceResponse {
    pub fn from_domain(co_occurrence: EntityCoOccurrence) -> Self {
        Self {
            id: co_occurrence.entity.id.to_string(),
            object: "entity",
            name: co_occurrence.entity.name,
            entity_type: entity_type_to_api(co_occurrence.entity.entity_type).into(),
            description: co_occurrence.entity.description,
            shared_item_count: co_occurrence.shared_item_count,
            total_mentions: co_occurrence.total_mentions,
            created_at: co_occurrence.entity.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EntityDetailResponse {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    pub entity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub total_mentions: i64,
    pub item_count: i64,
    #[schema(value_type = String, format = DateTime)]
    pub first_seen_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub last_seen_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub co_occurring: Vec<EntityCoOccurrenceResponse>,
}

impl EntityDetailResponse {
    pub fn from_domain(detail: EntityDetail) -> Self {
        Self {
            id: detail.entity.id.to_string(),
            object: "entity",
            name: detail.entity.name,
            entity_type: entity_type_to_api(detail.entity.entity_type).into(),
            description: detail.entity.description,
            total_mentions: detail.total_mentions,
            item_count: detail.item_count,
            first_seen_at: detail.first_seen_at,
            last_seen_at: detail.last_seen_at,
            created_at: detail.entity.created_at,
            co_occurring: detail
                .co_occurring
                .into_iter()
                .map(EntityCoOccurrenceResponse::from_domain)
                .collect(),
        }
    }
}

pub(crate) fn parse_entity_id(s: &str) -> Result<EntityId, ApiError> {
    s.parse().map_err(|_| ApiError::NotFound {
        entity: "Entity",
        id: s.to_string(),
    })
}

pub(crate) fn parse_entity_type_param(value: &str) -> Result<EntityType, FieldError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "person" => Ok(EntityType::Person),
        "organization" => Ok(EntityType::Organization),
        "location" => Ok(EntityType::Location),
        "event" => Ok(EntityType::Event),
        "topic" | "work" => Ok(EntityType::Work),
        _ => Err(FieldError {
            field: "type".into(),
            message: "must be one of: person, organization, location, event, topic".into(),
        }),
    }
}

fn entity_type_to_api(value: EntityType) -> &'static str {
    match value {
        EntityType::Person => "person",
        EntityType::Organization => "organization",
        EntityType::Location => "location",
        EntityType::Event => "event",
        EntityType::Work => "topic",
    }
}

fn deserialize_optional_nullable<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<Option<String>> = Option::deserialize(deserializer)?;
    Ok(value)
}
