//! Feed delivery entities for the document/feed/library architecture.
//!
//! Source of truth: docs/document-feed-library-architecture.md (feed_deliveries).
//! A `FeedDelivery` is one user-visible delivery of a feed source entry. `document_id` is nullable
//! by design: most deliveries are never materialized and render from `feed_source_entries`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    DocumentId, DocumentType, FeedDeliveryId, FeedSourceEntryId, FeedSourceId, FeedSubscriptionId,
    UserId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedDelivery {
    pub id: FeedDeliveryId,
    pub user_id: UserId,
    pub subscription_id: FeedSubscriptionId,
    pub source_id: FeedSourceId,
    pub source_entry_id: FeedSourceEntryId,
    pub document_id: Option<DocumentId>,
    pub delivered_at: DateTime<Utc>,
    pub seen_at: Option<DateTime<Utc>>,
    pub dismissed_at: Option<DateTime<Utc>>,
    pub hidden_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Seen/unseen partition for the Feed Unseen and Seen tabs. A delivery is Unseen while
/// `seen_at IS NULL` and Seen once `seen_at` is set. Dismissed/hidden/saved deliveries are
/// excluded from both lists by the query, not represented here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedDeliveryState {
    Unseen,
    Seen,
}

impl FeedDeliveryState {
    pub const NAMES: &'static [&'static str] = &["unseen", "seen"];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unseen => "unseen",
            Self::Seen => "seen",
        }
    }
}

impl std::str::FromStr for FeedDeliveryState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unseen" => Ok(Self::Unseen),
            "seen" => Ok(Self::Seen),
            _ => Err(()),
        }
    }
}

/// Document fields overlaid onto a delivery once it has been materialized (AC #3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOverlay {
    pub document_id: DocumentId,
    pub document_type: DocumentType,
    pub title: String,
    pub canonical_url: Option<String>,
    pub author: Option<String>,
    pub excerpt: Option<String>,
    pub lead_image_url: Option<String>,
    pub thumbnail_url: Option<String>,
}

/// A Feed list row. The `entry_*` fields come from `feed_source_entries` and are the
/// pre-materialization display source of truth (AC #2). `document` is the optional
/// overlay present once the delivery is linked to a document (AC #3). The delivery row
/// is always returned regardless of `document_id`. `saved` is true when the linked
/// document has an active Library entry; the Feed lists exclude saved rows, so it is only
/// meaningful via the single-delivery `find_display_by_id` read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedDeliveryDisplay {
    pub delivery: FeedDelivery,
    pub entry_title: String,
    pub entry_url: Option<String>,
    pub entry_author: Option<String>,
    pub entry_excerpt: Option<String>,
    pub entry_published_at: Option<DateTime<Utc>>,
    /// Lead image from the source entry, used as the display image until a document overlay
    /// supplies its own (AC #2). Drives both `lead_image_url` and `thumbnail_url` for unprepared
    /// deliveries.
    pub entry_lead_image_url: Option<String>,
    pub document: Option<DocumentOverlay>,
    pub saved: bool,
}
