use super::helpers::default_webhook_active;
use super::*;

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryHistoryTick {
    S2xx,
    S4xx,
    S5xx,
    Pending,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEndpointStatus {
    Healthy,
    Failing,
    Paused,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookEndpointResponse {
    pub id: String,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub secret_preview: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
    pub last_status: WebhookEndpointStatus,
    pub delivery_history: Vec<DeliveryHistoryTick>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookEndpointSecretResponse {
    #[serde(flatten)]
    pub endpoint: WebhookEndpointResponse,
    pub raw_secret: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookEndpointListResponse {
    pub data: Vec<WebhookEndpointResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookDeliveryResponse {
    pub id: String,
    pub endpoint_id: String,
    pub event: String,
    pub target: String,
    pub status_code: Option<i32>,
    #[schema(value_type = String, format = DateTime)]
    pub delivered_at: DateTime<Utc>,
    pub latency_ms: Option<i32>,
    pub attempt: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookDeliveryListResponse {
    pub data: Vec<WebhookDeliveryResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWebhookEndpointRequest {
    pub name: Option<String>,
    pub url: String,
    pub events: Vec<String>,
    #[serde(default = "default_webhook_active")]
    pub is_active: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateWebhookEndpointRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TestWebhookEndpointRequest {
    pub event: String,
}
