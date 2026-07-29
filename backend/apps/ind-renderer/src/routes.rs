use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json};

use crate::browser::BrowserManager;
use crate::render;
use crate::storage::S3Storage;
use crate::types::{HealthResponse, RenderErrorResponse, RenderMonolithRequest, RenderUrlRequest};

pub struct AppState {
    pub browser: Arc<BrowserManager>,
    pub capture: crate::config::CaptureSettings,
    pub storage: Arc<S3Storage>,
    pub egress_policy: ind_egress::EgressPolicy,
}

pub async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        browser_running: state.browser.is_browser_running(),
    })
}

pub async fn render_url(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RenderUrlRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Defense-in-depth SSRF pre-flight: refuse to navigate to a private/internal
    // target before driving Chromium. Returns 422 so the caller treats it as a
    // permanent (non-retryable) failure.
    if let Err(err) = ind_egress::resolve_and_validate(
        &req.url,
        &ind_egress::UrlRules::ingest(),
        &state.egress_policy,
    )
    .await
    {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(render_error_json(err.client_message().to_string())),
        );
    }

    let mut page_guard = match state.browser.acquire_page().await {
        Ok(pg) => pg,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(render_error_json(format!(
                    "failed to acquire browser page: {e}"
                ))),
            );
        }
    };

    let response = render::render_url(
        &page_guard,
        &state.storage,
        &req.url,
        &req.user_id,
        &req.item_id,
        &req.outputs,
        &state.capture,
    )
    .await;

    if let Some(error) = response
        .as_ref()
        .err()
        .filter(|error| error.requires_browser_recovery())
    {
        page_guard
            .close_after_capture_failure(error.stage_label(), error.browser_is_unhealthy())
            .await;
    } else if let Err(error) = page_guard.close().await {
        tracing::warn!(%error, "failed to close capture page cleanly");
    }

    match response {
        Ok(response) => (StatusCode::OK, Json(render_success_json(response))),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(render_error_json(err.to_string())),
        ),
    }
}

#[expect(
    clippy::expect_used,
    reason = "RenderErrorResponse is a plain struct with a single String field; serialization cannot fail"
)]
fn render_error_json(error: String) -> serde_json::Value {
    serde_json::to_value(RenderErrorResponse { error })
        .expect("render error response should serialize")
}

#[expect(
    clippy::expect_used,
    reason = "RenderResponse is a plain struct of strings, numbers, and vecs; serialization cannot fail"
)]
fn render_success_json(response: crate::types::RenderResponse) -> serde_json::Value {
    serde_json::to_value(response).expect("render response should serialize")
}

pub async fn render_monolith(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RenderMonolithRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut page_guard = match state.browser.acquire_page().await {
        Ok(pg) => pg,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(render_error_json(format!(
                    "failed to acquire browser page: {e}"
                ))),
            );
        }
    };

    let response = render::render_monolith(
        &page_guard,
        &state.storage,
        &req.monolith_s3_key,
        &req.user_id,
        &req.item_id,
        &req.outputs,
    )
    .await;

    if let Some(error) = response
        .as_ref()
        .err()
        .filter(|error| error.requires_browser_recovery())
    {
        page_guard
            .close_after_capture_failure(error.stage_label(), error.browser_is_unhealthy())
            .await;
    } else if let Err(error) = page_guard.close().await {
        tracing::warn!(%error, "failed to close capture page cleanly");
    }

    match response {
        Ok(response) => (StatusCode::OK, Json(render_success_json(response))),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(render_error_json(err.to_string())),
        ),
    }
}
