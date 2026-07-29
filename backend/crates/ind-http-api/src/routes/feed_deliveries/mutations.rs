use super::*;

#[utoipa::path(
    post,
    path = "/api/v1/feeds/deliveries/mark-all-seen",
    request_body = MarkAllDeliveriesSeenBody,
    responses(
        (status = 200, description = "Deliveries marked seen", body = MarkAllDeliveriesSeenResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Feed",
)]
pub async fn mark_all_deliveries_seen(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<MarkAllDeliveriesSeenBody>,
) -> Result<ApiResponse<MarkAllDeliveriesSeenResponse>, ApiError> {
    let ops = require_feed_delivery_ops(&state)?;
    let subscription_id = parse_subscription_id(body.subscription_id.as_deref())?;
    let updated = ops
        .mark_all_seen(auth_user.user_id, subscription_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(MarkAllDeliveriesSeenResponse { updated }))
}

#[utoipa::path(
    post,
    path = "/api/v1/feeds/deliveries/{delivery_id}/seen",
    params(("delivery_id" = String, Path, description = "Feed delivery id")),
    responses(
        (status = 200, description = "Delivery marked seen", body = FeedDeliveryResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Feed delivery not found"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Feed",
)]
pub async fn mark_delivery_seen(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(delivery_id): Path<String>,
) -> Result<ApiResponse<FeedDeliveryResponse>, ApiError> {
    let ops = require_feed_delivery_ops(&state)?;
    let id = parse_delivery_id(&delivery_id)?;
    ops.mark_seen(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?;
    fetch_response(ops, auth_user.user_id, id).await
}

#[utoipa::path(
    post,
    path = "/api/v1/feeds/deliveries/{delivery_id}/dismiss",
    params(("delivery_id" = String, Path, description = "Feed delivery id")),
    responses(
        (status = 200, description = "Delivery dismissed", body = FeedDeliveryResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Feed delivery not found"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Feed",
)]
pub async fn dismiss_delivery(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(delivery_id): Path<String>,
) -> Result<ApiResponse<FeedDeliveryResponse>, ApiError> {
    let ops = require_feed_delivery_ops(&state)?;
    let id = parse_delivery_id(&delivery_id)?;
    ops.dismiss(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?;
    fetch_response(ops, auth_user.user_id, id).await
}

/// Re-read the delivery with its document overlay so the mutation response carries the same
/// shape as the list/detail reads.
async fn fetch_response(
    ops: &dyn FeedDeliveryOperations,
    user_id: ind_domain::UserId,
    id: ind_domain::FeedDeliveryId,
) -> Result<ApiResponse<FeedDeliveryResponse>, ApiError> {
    let display =
        ops.get(user_id, id)
            .await
            .map_err(ApiError::from)?
            .ok_or(ApiError::NotFound {
                entity: "FeedDelivery",
                id: id.to_string(),
            })?;
    Ok(ApiResponse::new(FeedDeliveryResponse::from_display(
        display,
    )))
}
