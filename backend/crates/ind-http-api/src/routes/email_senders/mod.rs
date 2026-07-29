pub(crate) mod dto;

use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use ind_application::ports::EmailSenderOperations;

use crate::error::ApiError;
use crate::extract::{Json, ValidatedJson};
use crate::middleware::{RequireFeedsRead, RequireFeedsWrite};
use crate::state::AppState;

pub use dto::{
    DestinationDto, EmailSenderResponse, ListEmailSendersParams, ListEmailSendersResponse,
    RenderDefaultDto, UnsubscribeEmailSenderResponse, UpdateEmailSenderRequest,
};

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 200;

fn service(state: &AppState) -> Result<&Arc<dyn EmailSenderOperations>, ApiError> {
    state
        .email_sender_ops
        .as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable {
            message: "email sender operations are not configured".into(),
        })
}

fn clamp_pagination(params: ListEmailSendersParams) -> (i64, i64) {
    let offset = params.offset.unwrap_or(0).max(0);
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    (offset, limit)
}

#[utoipa::path(
    get,
    path = "/api/v1/email-senders",
    params(ListEmailSendersParams),
    responses(
        (status = 200, description = "Paginated list of email senders", body = ListEmailSendersResponse),
        (status = 401, description = "Authentication required"),
        (status = 503, description = "Email sender operations not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["feeds:read"]))),
    tag = "Email Senders",
)]
pub async fn list_email_senders(
    RequireFeedsRead {
        principal: auth_user,
        ..
    }: RequireFeedsRead,
    State(state): State<AppState>,
    Query(params): Query<ListEmailSendersParams>,
) -> Result<Json<ListEmailSendersResponse>, ApiError> {
    let ops = service(&state)?;
    let (offset, limit) = clamp_pagination(params);

    let (senders, total) = ops
        .list(auth_user.user_id, offset, limit)
        .await
        .map_err(ApiError::from)?;

    let data = senders
        .into_iter()
        .map(EmailSenderResponse::from_domain)
        .collect();

    Ok(Json(ListEmailSendersResponse {
        data,
        total,
        offset,
        limit,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/email-senders/{id}",
    params(("id" = String, Path, description = "Email sender ID with snd_ prefix")),
    request_body = UpdateEmailSenderRequest,
    responses(
        (status = 200, description = "Email sender updated", body = EmailSenderResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Email sender not found"),
        (status = 422, description = "Validation error"),
        (status = 503, description = "Email sender operations not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["feeds:write"]))),
    tag = "Email Senders",
)]
pub async fn update_email_sender(
    RequireFeedsWrite {
        principal: auth_user,
        ..
    }: RequireFeedsWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateEmailSenderRequest>,
) -> Result<Json<EmailSenderResponse>, ApiError> {
    let ops = service(&state)?;
    let sender_id = dto::parse_sender_id(&id)?;

    let mut current: Option<ind_domain::EmailSender> = None;

    if let Some(blocked) = body.blocked {
        let sender = if blocked {
            ops.block(auth_user.user_id, sender_id).await
        } else {
            ops.unblock(auth_user.user_id, sender_id).await
        }
        .map_err(ApiError::from)?;
        current = Some(sender);
    }

    if let Some(render_default) = body.render_default {
        let sender = ops
            .set_render_default(auth_user.user_id, sender_id, render_default.into())
            .await
            .map_err(ApiError::from)?;
        current = Some(sender);
    }

    if let Some(routing_default) = body.routing_default {
        let sender = ops
            .set_routing_default(
                auth_user.user_id,
                sender_id,
                routing_default.map(Into::into),
            )
            .await
            .map_err(ApiError::from)?;
        current = Some(sender);
    }

    let sender = match current {
        Some(sender) => sender,
        None => ops
            .get(auth_user.user_id, sender_id)
            .await
            .map_err(ApiError::from)?,
    };

    Ok(Json(EmailSenderResponse::from_domain(sender)))
}

#[utoipa::path(
    post,
    path = "/api/v1/email-senders/{id}/unsubscribe",
    params(("id" = String, Path, description = "Email sender ID with snd_ prefix")),
    responses(
        (status = 200, description = "Local block applied + unsubscribe job enqueued", body = UnsubscribeEmailSenderResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Email sender not found"),
        (status = 503, description = "Email sender operations not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["feeds:write"]))),
    tag = "Email Senders",
)]
pub async fn unsubscribe_email_sender(
    RequireFeedsWrite {
        principal: auth_user,
        ..
    }: RequireFeedsWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<UnsubscribeEmailSenderResponse>, ApiError> {
    let ops = service(&state)?;
    let sender_id = dto::parse_sender_id(&id)?;
    let outcome = ops
        .unsubscribe(auth_user.user_id, sender_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(UnsubscribeEmailSenderResponse::from_outcome(outcome)?))
}

pub fn email_sender_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/email-senders", get(list_email_senders))
        .route(
            "/api/v1/email-senders/{id}",
            axum::routing::patch(update_email_sender),
        )
        .route(
            "/api/v1/email-senders/{id}/unsubscribe",
            post(unsubscribe_email_sender),
        )
}
