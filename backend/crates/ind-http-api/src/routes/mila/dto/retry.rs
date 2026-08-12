use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct RetryMilaActionResponse {
    pub queued: bool,
    pub action: String,
}
