use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json};

use crate::browser::{BrowserManager, PageGuard};
use crate::render::{self, CaptureError};
use crate::storage::S3Storage;
use crate::types::{
    HealthResponse, RenderErrorResponse, RenderMonolithRequest, RenderResponse, RenderUrlRequest,
};

pub struct AppState {
    pub browser: Arc<BrowserManager>,
    pub capture: crate::config::CaptureSettings,
    pub storage: Arc<S3Storage>,
    pub egress_policy: ind_egress::EgressPolicy,
}

type RouteResponse = (StatusCode, Json<serde_json::Value>);

pub async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        browser_running: state.browser.is_browser_running(),
    })
}

pub async fn render_url(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RenderUrlRequest>,
) -> RouteResponse {
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

    run_capture(&state, async |page_guard| {
        render::render_url(
            page_guard,
            &state.storage,
            &req.url,
            &req.user_id,
            &req.item_id,
            &req.outputs,
            &state.capture,
        )
        .await
    })
    .await
}

pub async fn render_monolith(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RenderMonolithRequest>,
) -> RouteResponse {
    run_capture(&state, async |page_guard| {
        render::render_monolith(
            page_guard,
            &state.storage,
            &req.monolith_s3_key,
            &req.user_id,
            &req.item_id,
            &req.outputs,
        )
        .await
    })
    .await
}

/// Runs one capture on a freshly acquired page under the configured deadline, then returns the
/// page (and, when the capture left the browser in doubt, the browser itself) before answering.
///
/// The deadline is what keeps a single wedged capture from holding a capture slot forever: CDP
/// calls have no reliable timeout of their own once Chromium goes silent, so the route is the
/// boundary that guarantees every slot is eventually released.
async fn run_capture(
    state: &AppState,
    capture: impl AsyncFnOnce(&PageGuard) -> Result<RenderResponse, CaptureError>,
) -> RouteResponse {
    let mut page_guard = match state.browser.acquire_page().await {
        Ok(page_guard) => page_guard,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(render_error_json(format!(
                    "failed to acquire browser page: {error}"
                ))),
            );
        }
    };

    let response = tokio::time::timeout(state.capture.deadline(), capture(&page_guard))
        .await
        .unwrap_or_else(|_elapsed| {
            Err(CaptureError::DeadlineExceeded {
                budget_secs: state.capture.deadline_secs,
            })
        });

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
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(render_error_json(error.to_string())),
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
fn render_success_json(response: RenderResponse) -> serde_json::Value {
    serde_json::to_value(response).expect("render response should serialize")
}
