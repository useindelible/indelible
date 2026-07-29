use chrono::{DateTime, Utc};
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::entity::EntityDocument;
use ind_domain::{
    DocumentId, DomainError, Entity, EntityCoOccurrence, EntityId, EntitySummary, EntityType,
    UserId,
};

#[derive(sqlx::FromRow)]
pub(super) struct EntityRow {
    pub(super) id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) name: String,
    pub(super) entity_type: String,
    pub(super) description: Option<String>,
    pub(super) created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub(super) struct EntitySummaryRow {
    pub(super) id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) name: String,
    pub(super) entity_type: String,
    pub(super) description: Option<String>,
    pub(super) created_at: DateTime<Utc>,
    pub(super) total_mentions: i64,
    pub(super) item_count: i64,
    pub(super) first_seen_at: DateTime<Utc>,
    pub(super) last_seen_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub(super) struct EntityDocumentRow {
    pub(super) document_id: Uuid,
    pub(super) title: String,
    pub(super) author: Option<String>,
    pub(super) excerpt: Option<String>,
    pub(super) domain: Option<String>,
    pub(super) saved_at: DateTime<Utc>,
}

impl From<EntityDocumentRow> for EntityDocument {
    fn from(row: EntityDocumentRow) -> Self {
        Self {
            document_id: DocumentId::from_uuid(row.document_id),
            title: row.title,
            author: row.author,
            excerpt: row.excerpt,
            domain: row.domain,
            saved_at: row.saved_at,
        }
    }
}

#[derive(sqlx::FromRow)]
pub(super) struct EntityCoOccurrenceRow {
    pub(super) id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) name: String,
    pub(super) entity_type: String,
    pub(super) description: Option<String>,
    pub(super) created_at: DateTime<Utc>,
    pub(super) shared_item_count: i64,
    pub(super) total_mentions: i64,
}

impl TryFrom<EntityRow> for Entity {
    type Error = AppError;

    fn try_from(row: EntityRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: EntityId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            name: row.name,
            entity_type: parse_entity_type(&row.entity_type)?,
            description: row.description,
            created_at: row.created_at,
        })
    }
}

impl TryFrom<EntitySummaryRow> for EntitySummary {
    type Error = AppError;

    fn try_from(row: EntitySummaryRow) -> Result<Self, Self::Error> {
        Ok(Self {
            entity: Entity {
                id: EntityId::from_uuid(row.id),
                user_id: UserId::from_uuid(row.user_id),
                name: row.name,
                entity_type: parse_entity_type(&row.entity_type)?,
                description: row.description,
                created_at: row.created_at,
            },
            total_mentions: row.total_mentions,
            item_count: row.item_count,
            first_seen_at: row.first_seen_at,
            last_seen_at: row.last_seen_at,
        })
    }
}

impl TryFrom<EntityCoOccurrenceRow> for EntityCoOccurrence {
    type Error = AppError;

    fn try_from(row: EntityCoOccurrenceRow) -> Result<Self, Self::Error> {
        Ok(Self {
            entity: Entity {
                id: EntityId::from_uuid(row.id),
                user_id: UserId::from_uuid(row.user_id),
                name: row.name,
                entity_type: parse_entity_type(&row.entity_type)?,
                description: row.description,
                created_at: row.created_at,
            },
            shared_item_count: row.shared_item_count,
            total_mentions: row.total_mentions,
        })
    }
}

pub(super) fn format_entity_type(value: EntityType) -> &'static str {
    match value {
        EntityType::Person => "person",
        EntityType::Organization => "organization",
        EntityType::Location => "location",
        EntityType::Event => "event",
        EntityType::Work => "work",
    }
}

fn parse_entity_type(value: &str) -> Result<EntityType, AppError> {
    match value {
        "person" => Ok(EntityType::Person),
        "organization" => Ok(EntityType::Organization),
        "location" => Ok(EntityType::Location),
        "event" => Ok(EntityType::Event),
        "work" => Ok(EntityType::Work),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("unknown entity type: {other}"),
        })),
    }
}
