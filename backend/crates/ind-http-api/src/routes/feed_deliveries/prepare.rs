use super::*;
use ind_application::ports::FeedPreparationOperations;

fn require_feed_preparation_ops(
    state: &AppState,
) -> Result<&dyn FeedPreparationOperations, ApiError> {
    state
        .feed_preparation_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "feed preparation service not configured".into(),
        })
}

#[utoipa::path(
    post,
    path = "/api/v1/feeds/deliveries/read-ahead",
    request_body = ReadAheadBody,
    responses(
        (status = 200, description = "Read-ahead preparation triggered", body = ReadAheadResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
        (status = 503, description = "Feed preparation service not configured"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Feed",
)]
pub async fn prepare_feed_read_ahead(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<ReadAheadBody>,
) -> Result<ApiResponse<ReadAheadResponse>, ApiError> {
    let ops = require_feed_preparation_ops(&state)?;
    let subscription_id = parse_subscription_id(body.subscription_id.as_deref())?;
    let outcome = ops
        .prepare_read_ahead(auth_user.user_id, subscription_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(ReadAheadResponse {
        prepared: outcome.prepared,
        document_ids: outcome
            .document_ids
            .into_iter()
            .map(|id| id.to_string())
            .collect(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/feeds/deliveries/{delivery_id}/prepare",
    params(("delivery_id" = String, Path, description = "Feed delivery id")),
    responses(
        (status = 200, description = "Delivery prepared", body = PrepareDeliveryResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Feed delivery not found"),
        (status = 422, description = "Delivery has no canonical URL to prepare"),
        (status = 503, description = "Feed preparation service not configured"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Feed",
)]
pub async fn prepare_feed_delivery(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(delivery_id): Path<String>,
) -> Result<ApiResponse<PrepareDeliveryResponse>, ApiError> {
    let ops = require_feed_preparation_ops(&state)?;
    let id = parse_delivery_id(&delivery_id)?;
    let outcome = ops
        .prepare_delivery(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(PrepareDeliveryResponse {
        document_id: outcome.document_id.to_string(),
    }))
}
