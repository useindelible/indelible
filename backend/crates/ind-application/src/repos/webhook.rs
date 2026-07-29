use chrono::{DateTime, Utc};
use ind_domain::{
    DomainEvent, DomainEventId, UserId, WebhookDelivery, WebhookDeliveryId, WebhookDispatch,
    WebhookDispatchId, WebhookEndpoint, WebhookEndpointId,
};

use crate::AppError;

#[derive(Debug, Clone)]
pub struct UpdateWebhookEndpointInput {
    pub name: Option<String>,
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct WebhookDeliveryInput {
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
}

#[derive(Debug, Clone)]
pub struct WebhookDispatchContext {
    pub dispatch: WebhookDispatch,
    pub endpoint: WebhookEndpoint,
    pub event: DomainEvent,
}

#[derive(Debug, Clone)]
pub struct WebhookProjectionEvent {
    pub event: DomainEvent,
    pub dispatch_ids: Vec<WebhookDispatchId>,
}

#[async_trait::async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn create_endpoint(&self, endpoint: WebhookEndpoint)
    -> Result<WebhookEndpoint, AppError>;

    async fn list_endpoints(&self, user_id: UserId) -> Result<Vec<WebhookEndpoint>, AppError>;

    async fn find_endpoint(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
    ) -> Result<Option<WebhookEndpoint>, AppError>;

    async fn update_endpoint(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
        input: UpdateWebhookEndpointInput,
    ) -> Result<WebhookEndpoint, AppError>;

    async fn update_endpoint_secret(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
        secret_hash: String,
        secret_ciphertext: Vec<u8>,
        secret_preview: String,
    ) -> Result<WebhookEndpoint, AppError>;

    async fn delete_endpoint(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
    ) -> Result<(), AppError>;

    async fn list_deliveries(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
        limit: i64,
    ) -> Result<Vec<WebhookDelivery>, AppError>;

    async fn create_test_dispatch(
        &self,
        endpoint: &WebhookEndpoint,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<WebhookDispatchContext, AppError>;

    async fn get_dispatch_context(
        &self,
        dispatch_id: WebhookDispatchId,
    ) -> Result<Option<WebhookDispatchContext>, AppError>;

    async fn record_delivery(
        &self,
        input: WebhookDeliveryInput,
    ) -> Result<WebhookDelivery, AppError>;

    async fn mark_dispatch_delivered(
        &self,
        dispatch_id: WebhookDispatchId,
        delivered_at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn mark_dispatch_exhausted(
        &self,
        dispatch_id: WebhookDispatchId,
        exhausted_at: DateTime<Utc>,
        last_error: String,
    ) -> Result<(), AppError>;

    async fn project_next_events(
        &self,
        batch_size: i64,
    ) -> Result<Vec<WebhookProjectionEvent>, AppError>;
}
