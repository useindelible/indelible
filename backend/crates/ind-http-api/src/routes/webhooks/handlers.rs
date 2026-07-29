use super::helpers::*;
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/webhooks",
    responses(
        (status = 200, description = "List webhook endpoints", body = WebhookEndpointListResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Email verification required"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Webhooks",
)]
pub async fn list_webhook_endpoints(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
) -> Result<ApiResponse<WebhookEndpointListResponse>, ApiError> {
    let ops = service(&state)?;
    let endpoints = ops
        .list_endpoints(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;

    let mut data = Vec::with_capacity(endpoints.len());
    for endpoint in endpoints {
        let deliveries = ops
            .list_deliveries(auth_user.user_id, endpoint.id, 8)
            .await
            .map_err(ApiError::from)?;
        data.push(endpoint_response(endpoint, &deliveries));
    }

    Ok(ApiResponse::new(WebhookEndpointListResponse { data }))
}

#[utoipa::path(
    post,
    path = "/api/v1/webhooks",
    request_body = CreateWebhookEndpointRequest,
    responses(
        (status = 201, description = "Webhook endpoint created", body = WebhookEndpointSecretResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Email verification required"),
        (status = 422, description = "Validation error"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Webhooks",
)]
pub async fn create_webhook_endpoint(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Json(body): Json<CreateWebhookEndpointRequest>,
) -> Result<(http::StatusCode, Json<WebhookEndpointSecretResponse>), ApiError> {
    let name = validate_name(body.name.as_deref(), &body.url)?;
    let url = validate_webhook_url(&body.url, state.config.allow_private_webhook_targets)?;
    let events = validate_events(body.events)?;
    let ops = service(&state)?;
    let (endpoint, raw_secret) = ops
        .create_endpoint(auth_user.user_id, name, url, events, body.is_active)
        .await
        .map_err(ApiError::from)?;

    Ok((
        http::StatusCode::CREATED,
        Json(WebhookEndpointSecretResponse {
            endpoint: endpoint_response(endpoint, &[]),
            raw_secret,
        }),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/webhooks/{webhook_id}",
    request_body = UpdateWebhookEndpointRequest,
    params(("webhook_id" = String, Path, description = "Webhook endpoint ID with whk_ prefix")),
    responses(
        (status = 200, description = "Webhook endpoint updated", body = WebhookEndpointResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Email verification required"),
        (status = 404, description = "Webhook endpoint not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Webhooks",
)]
pub async fn update_webhook_endpoint(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(webhook_id): Path<String>,
    Json(body): Json<UpdateWebhookEndpointRequest>,
) -> Result<ApiResponse<WebhookEndpointResponse>, ApiError> {
    let endpoint_id = parse_webhook_id(&webhook_id)?;
    let name = match body.name.as_deref() {
        Some(name) => Some(validate_name(
            Some(name),
            body.url.as_deref().unwrap_or(""),
        )?),
        None => None,
    };
    let url = match body.url.as_deref() {
        Some(url) => Some(validate_webhook_url(
            url,
            state.config.allow_private_webhook_targets,
        )?),
        None => None,
    };
    let events = match body.events {
        Some(events) => Some(validate_events(events)?),
        None => None,
    };
    let ops = service(&state)?;
    let endpoint = ops
        .update_endpoint(
            auth_user.user_id,
            endpoint_id,
            name,
            url,
            events,
            body.is_active,
        )
        .await
        .map_err(ApiError::from)?;
    let deliveries = ops
        .list_deliveries(auth_user.user_id, endpoint.id, 8)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(endpoint_response(endpoint, &deliveries)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/webhooks/{webhook_id}",
    params(("webhook_id" = String, Path, description = "Webhook endpoint ID with whk_ prefix")),
    responses(
        (status = 204, description = "Webhook endpoint deleted"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Email verification required"),
        (status = 404, description = "Webhook endpoint not found"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Webhooks",
)]
pub async fn delete_webhook_endpoint(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(webhook_id): Path<String>,
) -> Result<EmptyResponse, ApiError> {
    let endpoint_id = parse_webhook_id(&webhook_id)?;
    service(&state)?
        .delete_endpoint(auth_user.user_id, endpoint_id)
        .await
        .map_err(ApiError::from)?;
    Ok(EmptyResponse)
}

#[utoipa::path(
    post,
    path = "/api/v1/webhooks/{webhook_id}/rotate-secret",
    params(("webhook_id" = String, Path, description = "Webhook endpoint ID with whk_ prefix")),
    responses(
        (status = 200, description = "Webhook secret rotated", body = WebhookEndpointSecretResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Email verification required"),
        (status = 404, description = "Webhook endpoint not found"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Webhooks",
)]
pub async fn rotate_webhook_secret(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(webhook_id): Path<String>,
) -> Result<ApiResponse<WebhookEndpointSecretResponse>, ApiError> {
    let endpoint_id = parse_webhook_id(&webhook_id)?;
    let ops = service(&state)?;
    let (endpoint, raw_secret) = ops
        .rotate_secret(auth_user.user_id, endpoint_id)
        .await
        .map_err(ApiError::from)?;
    let deliveries = ops
        .list_deliveries(auth_user.user_id, endpoint.id, 8)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(WebhookEndpointSecretResponse {
        endpoint: endpoint_response(endpoint, &deliveries),
        raw_secret,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/webhooks/{webhook_id}/test",
    request_body = TestWebhookEndpointRequest,
    params(("webhook_id" = String, Path, description = "Webhook endpoint ID with whk_ prefix")),
    responses(
        (status = 200, description = "Webhook test sent", body = WebhookDeliveryResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Email verification required"),
        (status = 404, description = "Webhook endpoint not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Webhooks",
)]
pub async fn test_webhook_endpoint(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(webhook_id): Path<String>,
    Json(body): Json<TestWebhookEndpointRequest>,
) -> Result<ApiResponse<WebhookDeliveryResponse>, ApiError> {
    let endpoint_id = parse_webhook_id(&webhook_id)?;
    let event = validate_events(vec![body.event])?
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::ValidationError {
            errors: vec![crate::error::FieldError {
                field: "event".into(),
                message: "must include at least one event".into(),
            }],
        })?;
    let delivery = service(&state)?
        .test_endpoint(auth_user.user_id, endpoint_id, event)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(delivery_response(
        delivery,
        "webhook endpoint",
    )))
}

#[utoipa::path(
    get,
    path = "/api/v1/webhooks/{webhook_id}/deliveries",
    params(("webhook_id" = String, Path, description = "Webhook endpoint ID with whk_ prefix")),
    responses(
        (status = 200, description = "Webhook delivery log", body = WebhookDeliveryListResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Email verification required"),
        (status = 404, description = "Webhook endpoint not found"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Webhooks",
)]
pub async fn list_webhook_deliveries(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(webhook_id): Path<String>,
) -> Result<ApiResponse<WebhookDeliveryListResponse>, ApiError> {
    let endpoint_id = parse_webhook_id(&webhook_id)?;
    let webhooks = service(&state)?;
    let target = webhooks
        .list_endpoints(auth_user.user_id)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .find(|endpoint| endpoint.id == endpoint_id)
        .map(|endpoint| endpoint.url)
        .ok_or_else(|| ApiError::NotFound {
            entity: "webhook",
            id: endpoint_id.to_string(),
        })?;
    let deliveries = webhooks
        .list_deliveries(auth_user.user_id, endpoint_id, 50)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(WebhookDeliveryListResponse {
        data: deliveries
            .into_iter()
            .map(|delivery| delivery_response(delivery, target.clone()))
            .collect(),
    }))
}
