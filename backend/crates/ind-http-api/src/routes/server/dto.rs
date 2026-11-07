use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ServerCapabilitiesResponse {
    /// Optional behaviours this server supports; clients treat an absent entry as unsupported.
    pub capabilities: Vec<String>,
}
