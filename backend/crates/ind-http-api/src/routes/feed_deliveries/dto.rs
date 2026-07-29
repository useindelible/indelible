use chrono::{DateTime, Utc};
use ind_domain::{FeedDeliveryDisplay, FeedDeliveryState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::FieldError;
use crate::extract::Validate;

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListFeedDeliveriesParams {
    /// Feed tab: `unseen` (default) or `seen`.
    pub state: Option<String>,
    /// Optional subscription filter.
    pub subscription_id: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

impl ListFeedDeliveriesParams {
    /// Parse the `state` query param, defaulting to Unseen when omitted.
    pub fn parse_state(&self) -> Result<FeedDeliveryState, Vec<FieldError>> {
        match self.state.as_deref() {
            None => Ok(FeedDeliveryState::Unseen),
            Some(raw) => raw.parse::<FeedDeliveryState>().map_err(|_| {
                vec![FieldError {
                    field: "state".into(),
                    message: format!("must be one of: {}", FeedDeliveryState::NAMES.join(", ")),
                }]
            }),
        }
    }
}

/// Mark all unseen deliveries seen, optionally scoped to one subscription.
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct MarkAllDeliveriesSeenBody {
    pub subscription_id: Option<String>,
}

impl Validate for MarkAllDeliveriesSeenBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MarkAllDeliveriesSeenResponse {
    pub updated: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FeedDeliveryCountResponse {
    pub unseen_count: i64,
}

/// Trigger read-ahead preparation when the user opens Feed, optionally scoped to one
/// subscription (which also marks that subscription active for this pass).
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct ReadAheadBody {
    pub subscription_id: Option<String>,
}

impl Validate for ReadAheadBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReadAheadResponse {
    /// Number of documents materialized and queued for readable preparation.
    pub prepared: u32,
    /// Ids of the documents queued for preparation.
    pub document_ids: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PrepareDeliveryResponse {
    /// The materialized/adopted document for the delivery. A readable-preparation render is
    /// always queued; load the document and poll its readable asset for readiness.
    pub document_id: String,
}

/// A Feed delivery row. Exposes `delivery_id`, `source_entry_id`, and a nullable
/// `document_id` per docs/document-feed-library-architecture.md (API Shape). Display fields
/// come from the feed source entry, overlaid by the linked document when one exists.
#[derive(Debug, Serialize, ToSchema)]
pub struct FeedDeliveryResponse {
    pub object: &'static str,
    pub delivery_id: String,
    pub source_entry_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    pub subscription_id: String,
    pub source_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    /// Content family from the linked document, when materialized.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub delivered_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub seen_at: Option<DateTime<Utc>>,
    /// True when the linked document has an active Library entry.
    pub saved: bool,
}

impl FeedDeliveryResponse {
    pub(crate) fn from_display(display: FeedDeliveryDisplay) -> Self {
        let FeedDeliveryDisplay {
            delivery,
            entry_title,
            entry_url,
            entry_author,
            entry_excerpt,
            entry_published_at,
            entry_lead_image_url,
            document,
            saved,
        } = display;

        // Title/author/excerpt/images overlay from the document once materialized (AC #2);
        // before that the entry is the display source of truth (AC #1). url and published_at
        // stay source-entry-first by design: the entry URL is the publisher link external-open
        // targets, and the feed's published date is what Feed shows (the DocumentOverlay carries
        // canonical_url, not the original_url/published_at a document COALESCE would need).
        let (title, doc_url, author, excerpt, lead_image_url, thumbnail_url, document_type) =
            match document {
                Some(doc) => {
                    // Use the document's own image once materialized; only when it has no image at
                    // all does the entry image fill the gap. A convergent link to a pre-existing
                    // imageless document (ON CONFLICT DO NOTHING) must not hide the entry's image.
                    let (lead_image_url, thumbnail_url) =
                        match (doc.lead_image_url, doc.thumbnail_url) {
                            (None, None) => {
                                (entry_lead_image_url.clone(), entry_lead_image_url.clone())
                            }
                            pair => pair,
                        };
                    (
                        doc.title,
                        doc.canonical_url,
                        doc.author.or(entry_author),
                        doc.excerpt.or(entry_excerpt),
                        lead_image_url,
                        thumbnail_url,
                        Some(doc.document_type.to_string()),
                    )
                }
                None => (
                    entry_title,
                    None,
                    entry_author,
                    entry_excerpt,
                    entry_lead_image_url.clone(),
                    entry_lead_image_url,
                    None,
                ),
            };

        Self {
            object: "feed_delivery",
            delivery_id: delivery.id.to_string(),
            source_entry_id: delivery.source_entry_id.to_string(),
            document_id: delivery.document_id.map(|id| id.to_string()),
            subscription_id: delivery.subscription_id.to_string(),
            source_id: delivery.source_id.to_string(),
            title,
            url: entry_url.or(doc_url),
            author,
            excerpt,
            published_at: entry_published_at,
            lead_image_url,
            thumbnail_url,
            document_type,
            delivered_at: delivery.delivered_at,
            seen_at: delivery.seen_at,
            saved,
        }
    }
}
