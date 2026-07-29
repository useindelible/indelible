use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/feeds/deliveries",
    params(ListFeedDeliveriesParams),
    responses(
        (status = 200, description = "Paginated feed deliveries", body = PaginatedResponse<FeedDeliveryResponse>),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["feeds:read"]))),
    tag = "Feed",
)]
pub async fn list_feed_deliveries(
    RequireFeedsRead {
        principal: auth_user,
        ..
    }: RequireFeedsRead,
    State(state): State<AppState>,
    Query(params): Query<ListFeedDeliveriesParams>,
) -> Result<PaginatedResponse<FeedDeliveryResponse>, ApiError> {
    let ops = require_feed_delivery_ops(&state)?;
    let delivery_state = params
        .parse_state()
        .map_err(|errors| ApiError::ValidationError { errors })?;
    let subscription_id = parse_subscription_id(params.subscription_id.as_deref())?;
    let cursor = params.cursor.map(Cursor);
    let limit = params.limit.unwrap_or(state.config.default_page_size);

    let page = ops
        .list(
            auth_user.user_id,
            delivery_state,
            subscription_id,
            cursor,
            limit,
        )
        .await
        .map_err(ApiError::from)?;

    Ok(PaginatedResponse::from(Page {
        items: page
            .items
            .into_iter()
            .map(FeedDeliveryResponse::from_display)
            .collect(),
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/feeds/deliveries/stats",
    responses(
        (status = 200, description = "Unseen feed delivery count", body = FeedDeliveryCountResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["feeds:read"]))),
    tag = "Feed",
)]
pub async fn get_feed_delivery_stats(
    RequireFeedsRead {
        principal: auth_user,
        ..
    }: RequireFeedsRead,
    State(state): State<AppState>,
) -> Result<ApiResponse<FeedDeliveryCountResponse>, ApiError> {
    let ops = require_feed_delivery_ops(&state)?;
    let unseen_count = ops
        .count_unseen(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(FeedDeliveryCountResponse { unseen_count }))
}

#[utoipa::path(
    get,
    path = "/api/v1/feeds/deliveries/{delivery_id}",
    params(("delivery_id" = String, Path, description = "Feed delivery id")),
    responses(
        (status = 200, description = "Feed delivery detail", body = FeedDeliveryResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Feed delivery not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["feeds:read"]))),
    tag = "Feed",
)]
pub async fn get_feed_delivery(
    RequireFeedsRead {
        principal: auth_user,
        ..
    }: RequireFeedsRead,
    State(state): State<AppState>,
    Path(delivery_id): Path<String>,
) -> Result<ApiResponse<FeedDeliveryResponse>, ApiError> {
    let ops = require_feed_delivery_ops(&state)?;
    let id = parse_delivery_id(&delivery_id)?;
    let display = ops
        .get(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound {
            entity: "FeedDelivery",
            id: delivery_id,
        })?;
    Ok(ApiResponse::new(FeedDeliveryResponse::from_display(
        display,
    )))
}
