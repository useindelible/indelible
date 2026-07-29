use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_domain::{
    DocumentId, DocumentOverlay, DocumentType, DomainError, FeedDelivery, FeedDeliveryDisplay,
    FeedDeliveryId, FeedSourceEntryId, FeedSourceId, FeedSubscriptionId, UserId,
};
use uuid::Uuid;

pub(super) fn map_delivery_error(err: sqlx::Error) -> AppError {
    super::super::map_sqlx_error("feed_delivery", "feed delivery conflict", err)
}

fn parse_document_type(value: &str) -> Result<DocumentType, AppError> {
    value.parse::<DocumentType>().map_err(|_| {
        AppError::Domain(DomainError::InvariantViolation {
            message: format!("unknown document_type: {value}"),
        })
    })
}

pub(super) struct DeliveryRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subscription_id: Uuid,
    pub source_id: Uuid,
    pub source_entry_id: Uuid,
    pub document_id: Option<Uuid>,
    pub delivered_at: DateTime<Utc>,
    pub seen_at: Option<DateTime<Utc>>,
    pub dismissed_at: Option<DateTime<Utc>>,
    pub hidden_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DeliveryRow> for FeedDelivery {
    fn from(row: DeliveryRow) -> Self {
        FeedDelivery {
            id: FeedDeliveryId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            subscription_id: FeedSubscriptionId::from_uuid(row.subscription_id),
            source_id: FeedSourceId::from_uuid(row.source_id),
            source_entry_id: FeedSourceEntryId::from_uuid(row.source_entry_id),
            document_id: row.document_id.map(DocumentId::from_uuid),
            delivered_at: row.delivered_at,
            seen_at: row.seen_at,
            dismissed_at: row.dismissed_at,
            hidden_at: row.hidden_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

pub(super) struct DeliveryDisplayRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subscription_id: Uuid,
    pub source_id: Uuid,
    pub source_entry_id: Uuid,
    pub document_id: Option<Uuid>,
    pub delivered_at: DateTime<Utc>,
    pub seen_at: Option<DateTime<Utc>>,
    pub dismissed_at: Option<DateTime<Utc>>,
    pub hidden_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub entry_title: String,
    pub entry_url: Option<String>,
    pub entry_author: Option<String>,
    pub entry_excerpt: Option<String>,
    pub entry_published_at: Option<DateTime<Utc>>,
    pub entry_lead_image_url: Option<String>,
    pub doc_document_type: Option<String>,
    pub doc_title: Option<String>,
    pub doc_canonical_url: Option<String>,
    pub doc_author: Option<String>,
    pub doc_excerpt: Option<String>,
    pub doc_lead_image_url: Option<String>,
    pub doc_thumbnail_url: Option<String>,
    pub saved: bool,
}

impl DeliveryDisplayRow {
    pub(super) fn into_display(self) -> Result<FeedDeliveryDisplay, AppError> {
        let document = match self.document_id {
            Some(document_id) => {
                let document_type = self.doc_document_type.as_deref().ok_or_else(missing_doc)?;
                Some(DocumentOverlay {
                    document_id: DocumentId::from_uuid(document_id),
                    document_type: parse_document_type(document_type)?,
                    title: self.doc_title.ok_or_else(missing_doc)?,
                    canonical_url: self.doc_canonical_url,
                    author: self.doc_author,
                    excerpt: self.doc_excerpt,
                    lead_image_url: self.doc_lead_image_url,
                    thumbnail_url: self.doc_thumbnail_url,
                })
            }
            None => None,
        };

        let delivery = FeedDelivery {
            id: FeedDeliveryId::from_uuid(self.id),
            user_id: UserId::from_uuid(self.user_id),
            subscription_id: FeedSubscriptionId::from_uuid(self.subscription_id),
            source_id: FeedSourceId::from_uuid(self.source_id),
            source_entry_id: FeedSourceEntryId::from_uuid(self.source_entry_id),
            document_id: self.document_id.map(DocumentId::from_uuid),
            delivered_at: self.delivered_at,
            seen_at: self.seen_at,
            dismissed_at: self.dismissed_at,
            hidden_at: self.hidden_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        };

        Ok(FeedDeliveryDisplay {
            delivery,
            entry_title: self.entry_title,
            entry_url: self.entry_url,
            entry_author: self.entry_author,
            entry_excerpt: self.entry_excerpt,
            entry_published_at: self.entry_published_at,
            entry_lead_image_url: self.entry_lead_image_url,
            document,
            saved: self.saved,
        })
    }
}

fn missing_doc() -> AppError {
    AppError::Domain(DomainError::InvariantViolation {
        message: "linked feed delivery is missing required document columns".into(),
    })
}
