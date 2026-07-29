use chrono::{DateTime, Utc};
use ind_domain::{DomainEvent, WebhookDeliveryId};
use serde::Serialize;

pub const WEBHOOK_EVENT_TYPES: &[&str] = &[
    "library_entry.saved",
    "library_entry.triaged",
    "library_entry.archived",
    "library_entry.favorited",
    "library_entry.trashed",
    "library_entry.restored",
    "library_entry.permanently_deleted",
    "library_entry.tagged",
    "library_entry.untagged",
    "document.highlighted",
    "highlight.updated",
    "highlight.deleted",
    "highlight.noted",
    "feed.subscribed",
    "feed.unsubscribed",
    "feed.new_item",
    "feed.poll_failed",
    "collection.created",
    "collection.updated",
    "collection.deleted",
    "collection.item_added",
    "collection.item_removed",
    "tag.created",
    "tag.merged",
    "integration.sync_completed",
    "integration.sync_failed",
    "account.created",
    "account.email_verified",
    "account.deleted",
    "review.completed",
    "review.streak",
];

#[derive(Debug, Clone, Serialize)]
pub struct WebhookPayload {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub created_at: DateTime<Utc>,
    pub data: serde_json::Value,
}

pub fn is_known_webhook_event(event: &str) -> bool {
    WEBHOOK_EVENT_TYPES.contains(&event)
}

pub fn payload_for_event(event: &DomainEvent) -> WebhookPayload {
    WebhookPayload {
        id: event.id.to_string(),
        event_type: event.event_type.clone(),
        created_at: event.created_at,
        data: event.payload.clone(),
    }
}

pub fn delivery_idempotency_key(delivery_id: WebhookDeliveryId) -> String {
    delivery_id.to_string()
}
