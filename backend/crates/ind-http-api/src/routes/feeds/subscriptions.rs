use super::{
    FeedSubscriptionResponse, ListSubscriptionsParams, OpmlImportResponse, SubscribeBody,
    SubscribeResponse, UpdateSubscriptionBody, dto, require_feed_ops,
};
use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::AccountAccess;
use crate::response::PaginatedResponse;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use utoipa::ToSchema;

#[utoipa::path(
    post,
    path = "/api/v1/feeds/subscriptions",
    request_body = SubscribeBody,
    responses(
        (status = 201, description = "Subscription created", body = SubscribeResponse),
        (status = 200, description = "Already subscribed", body = SubscribeResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Feeds",
)]
pub async fn subscribe(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<SubscribeBody>,
) -> Result<(http::StatusCode, crate::extract::Json<SubscribeResponse>), ApiError> {
    let feed_ops = require_feed_ops(&state)?;

    let result = feed_ops
        .subscribe(
            auth_user.user_id,
            body.url,
            body.title,
            body.poll_interval_override_minutes,
        )
        .await
        .map_err(ApiError::from)?;

    let status = if result.is_new {
        http::StatusCode::CREATED
    } else {
        http::StatusCode::OK
    };

    Ok((
        status,
        crate::extract::Json(SubscribeResponse {
            subscription: FeedSubscriptionResponse::from_domain(result.subscription),
            is_new: result.is_new,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/feeds/subscriptions",
    params(ListSubscriptionsParams),
    responses(
        (status = 200, description = "List of subscriptions", body = PaginatedResponse<FeedSubscriptionResponse>),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Feeds",
)]
pub async fn list_subscriptions(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Query(params): Query<ListSubscriptionsParams>,
) -> Result<PaginatedResponse<FeedSubscriptionResponse>, ApiError> {
    let feed_ops = require_feed_ops(&state)?;

    let page = feed_ops
        .list_subscriptions(auth_user.user_id, params.cursor, params.limit)
        .await
        .map_err(ApiError::from)?;

    Ok(PaginatedResponse::from(ind_application::repos::Page {
        items: page
            .items
            .into_iter()
            .map(FeedSubscriptionResponse::from_domain)
            .collect(),
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/feeds/subscriptions/{id}",
    params(
        ("id" = String, Path, description = "Subscription ID with fed_ prefix"),
    ),
    request_body = UpdateSubscriptionBody,
    responses(
        (status = 200, description = "Subscription updated", body = FeedSubscriptionResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Subscription not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Feeds",
)]
pub async fn update_subscription(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateSubscriptionBody>,
) -> Result<crate::extract::Json<FeedSubscriptionResponse>, ApiError> {
    let feed_ops = require_feed_ops(&state)?;
    let sub_id = dto::parse_feed_subscription_id(&id)?;

    let title_override = body
        .title
        .map(|t| if t.is_empty() { None } else { Some(t) });

    let status = body.status.as_deref().and_then(dto::parse_feed_status);

    let auto_save_collection_id = body
        .auto_save_collection_id
        .map(|opt| opt.map(|s| dto::parse_collection_id(&s)).transpose())
        .transpose()?;

    let sub = feed_ops
        .update_subscription(
            auth_user.user_id,
            sub_id,
            ind_application::UpdateSubscriptionInput {
                title_override,
                auto_save: body.auto_save,
                auto_save_collection_id,
                poll_interval_override_minutes: body.poll_interval_override_minutes,
                status,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(FeedSubscriptionResponse::from_domain(
        sub,
    )))
}

#[utoipa::path(
    delete,
    path = "/api/v1/feeds/subscriptions/{id}",
    params(
        ("id" = String, Path, description = "Subscription ID with fed_ prefix"),
    ),
    responses(
        (status = 204, description = "Subscription deleted"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Subscription not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Feeds",
)]
pub async fn unsubscribe(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<http::StatusCode, ApiError> {
    let feed_ops = require_feed_ops(&state)?;
    let sub_id = dto::parse_feed_subscription_id(&id)?;

    feed_ops
        .unsubscribe(auth_user.user_id, sub_id)
        .await
        .map_err(ApiError::from)?;

    Ok(http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/feeds/subscriptions/opml",
    request_body(content_type = "multipart/form-data", content = inline(OpmlUploadSchema)),
    responses(
        (status = 200, description = "OPML import results", body = OpmlImportResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Invalid OPML file"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Feeds",
)]
pub async fn import_opml(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    mut multipart: axum::extract::Multipart,
) -> Result<crate::extract::Json<OpmlImportResponse>, ApiError> {
    let feed_ops = require_feed_ops(&state)?;

    let mut opml_xml = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest {
            message: format!("multipart error: {e}"),
        })?
    {
        if field.name() == Some("file") {
            let bytes = field.bytes().await.map_err(|e| ApiError::BadRequest {
                message: format!("failed to read file: {e}"),
            })?;
            opml_xml =
                Some(
                    String::from_utf8(bytes.to_vec()).map_err(|_| ApiError::BadRequest {
                        message: "OPML file must be valid UTF-8".into(),
                    })?,
                );
            break;
        }
    }

    let xml = opml_xml.ok_or(ApiError::BadRequest {
        message: "missing 'file' field in multipart upload".into(),
    })?;

    let result = feed_ops
        .import_opml(auth_user.user_id, xml)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(OpmlImportResponse {
        created: result.created,
        skipped: result.skipped,
        errors: result.errors,
    }))
}

#[derive(ToSchema)]
pub struct OpmlUploadSchema {
    pub file: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/feeds/subscriptions/{id}/retry",
    params(
        ("id" = String, Path, description = "Subscription ID with fed_ prefix"),
    ),
    responses(
        (status = 200, description = "Subscription retried", body = FeedSubscriptionResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Subscription not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Feeds",
)]
pub async fn retry_subscription(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<crate::extract::Json<FeedSubscriptionResponse>, ApiError> {
    let feed_ops = require_feed_ops(&state)?;
    let sub_id = dto::parse_feed_subscription_id(&id)?;

    let sub = feed_ops
        .retry_subscription(auth_user.user_id, sub_id)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(FeedSubscriptionResponse::from_domain(
        sub,
    )))
}
