use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{DomainEventId, UserId, WebhookDeliveryId, WebhookDispatchId, WebhookEndpointId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookDispatchStatus {
    Pending,
    Delivered,
    Exhausted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: WebhookEndpointId,
    pub user_id: UserId,
    pub name: String,
    pub url: String,
    pub secret_hash: String,
    pub secret_ciphertext: Option<Vec<u8>>,
    pub secret_preview: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDispatch {
    pub id: WebhookDispatchId,
    pub domain_event_id: DomainEventId,
    pub endpoint_id: WebhookEndpointId,
    pub event_type: String,
    pub status: WebhookDispatchStatus,
    pub first_enqueued_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub exhausted_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: WebhookDeliveryId,
    pub dispatch_id: WebhookDispatchId,
    pub domain_event_id: DomainEventId,
    pub endpoint_id: WebhookEndpointId,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status_code: Option<i32>,
    pub response_body: Option<String>,
    pub attempt_number: i32,
    pub latency_ms: Option<i32>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDeliverJob {
    pub dispatch_id: WebhookDispatchId,
}
