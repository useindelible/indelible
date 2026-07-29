use axum::extract::State;
use axum::response::IntoResponse;
use http::StatusCode;
use ind_http_api::AppState;
use serde_json::json;

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.acquire().await {
        Ok(_) => (StatusCode::OK, axum::Json(json!({"status": "healthy"}))),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(json!({"status": "unhealthy"})),
        ),
    }
}
