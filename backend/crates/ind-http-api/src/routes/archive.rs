use axum::Router;

use crate::state::AppState;

pub fn archive_routes() -> Router<AppState> {
    Router::new()
}
