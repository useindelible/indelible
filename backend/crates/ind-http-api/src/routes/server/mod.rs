mod dto;

use axum::routing::get;
use axum::{Json, Router};

pub use dto::ServerCapabilitiesResponse;

use crate::state::AppState;

const CAPABILITIES: &[&str] = &["epub_chapter_list"];

/// List the optional capabilities this server supports.
#[utoipa::path(
    get,
    path = "/api/v1/server/capabilities",
    responses(
        (status = 200, description = "Capabilities this server supports", body = ServerCapabilitiesResponse),
    ),
    tag = "Server",
)]
pub async fn get_capabilities() -> Json<ServerCapabilitiesResponse> {
    Json(ServerCapabilitiesResponse {
        capabilities: CAPABILITIES.iter().map(ToString::to_string).collect(),
    })
}

pub fn server_routes() -> Router<AppState> {
    Router::new().route("/api/v1/server/capabilities", get(get_capabilities))
}
