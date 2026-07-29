pub(crate) mod dto;

use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;

use ind_application::ports::HomeOperations;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::AccountAccess;
use crate::state::AppState;

pub(crate) use dto::{
    ContinueReadingWidget, DailyReviewWidget, FeedDigestWidget, HomeDashboardParams,
    HomeDashboardResponse, HomeFeedItemResponse, HomeItemResponse, HomeSettingsResponse,
    HomeWidgetConfig, PinnedCollectionEntry, PinnedCollectionsWidget, QuickReadsWidget,
    ReadingStatsWidget, RecentlyAddedWidget, UpdateHomeSettingsBody,
};

fn require_home_ops(state: &AppState) -> Result<&dyn HomeOperations, ApiError> {
    state
        .home_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "home service not configured".into(),
        })
}

#[utoipa::path(
    get,
    path = "/api/v1/home",
    params(
        ("widgets" = Option<String>, Query, description = "Comma-separated widget names to include (omit for all)"),
    ),
    responses(
        (status = 200, description = "Home dashboard data", body = HomeDashboardResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Home",
)]
pub async fn get_home(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Query(params): Query<HomeDashboardParams>,
) -> Result<crate::extract::Json<HomeDashboardResponse>, ApiError> {
    let home_ops = require_home_ops(&state)?;

    let widgets = params.widgets.map(|w| {
        w.split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                serde_json::from_str::<ind_domain::HomeWidgetKind>(&format!("\"{trimmed}\"")).ok()
            })
            .collect::<Vec<_>>()
    });

    let data = home_ops
        .get_dashboard(auth_user.user_id, widgets)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(HomeDashboardResponse::from_dashboard(
        data,
    )))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/home",
    responses(
        (status = 200, description = "Home widget settings", body = HomeSettingsResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Home",
)]
pub async fn get_home_settings(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
) -> Result<crate::extract::Json<HomeSettingsResponse>, ApiError> {
    let home_ops = require_home_ops(&state)?;

    let config = home_ops
        .get_widget_config(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(HomeSettingsResponse {
        widget_config: config,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/settings/home",
    request_body = UpdateHomeSettingsBody,
    responses(
        (status = 200, description = "Home settings updated", body = HomeSettingsResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Home",
)]
pub async fn update_home_settings(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<UpdateHomeSettingsBody>,
) -> Result<crate::extract::Json<HomeSettingsResponse>, ApiError> {
    let home_ops = require_home_ops(&state)?;

    let config = serde_json::to_value(&body).map_err(|e| ApiError::BadRequest {
        message: format!("invalid settings: {e}"),
    })?;

    home_ops
        .set_widget_config(auth_user.user_id, config.clone())
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(HomeSettingsResponse {
        widget_config: Some(config),
    }))
}

pub fn home_routes() -> Router<AppState> {
    Router::new().route("/api/v1/home", get(get_home)).route(
        "/api/v1/settings/home",
        get(get_home_settings).patch(update_home_settings),
    )
}
