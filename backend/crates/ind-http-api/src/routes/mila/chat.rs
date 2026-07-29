use std::convert::Infallible;

use axum::extract::{Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::{StreamExt, stream};
use ind_application::ports::MilaStreamOutputStream;

use crate::error::ApiError;
use crate::extract::Validate;
use crate::middleware::RequireAiUseAndLibraryRead;
use crate::state::AppState;

use super::dto::{MilaStreamDeltaResponse, MilaStreamErrorResponse, MilaStreamParams};
use super::{require_mila_chat_ops, validation_error};

fn into_sse_stream(
    chat_stream: MilaStreamOutputStream,
) -> impl futures::Stream<Item = Result<Event, Infallible>> {
    stream::unfold(
        (chat_stream, false),
        |(mut chat_stream, terminal_sent)| async move {
            if terminal_sent {
                return None;
            }

            match chat_stream.next().await {
                Some(Ok(delta)) => {
                    #[expect(
                        clippy::expect_used,
                        reason = "serializing a plain owned struct of String fields to JSON is infallible"
                    )]
                    let payload = serde_json::to_string(&MilaStreamDeltaResponse {
                        delta: delta.delta,
                        retrieval_degraded: delta.retrieval_degraded,
                    })
                    .expect("Mila stream delta should serialize");
                    Some((Ok(Event::default().data(payload)), (chat_stream, false)))
                }
                Some(Err(err)) => {
                    #[expect(
                        clippy::expect_used,
                        reason = "serializing a plain owned struct of String fields to JSON is infallible"
                    )]
                    let payload = serde_json::to_string(&MilaStreamErrorResponse {
                        error: err.to_string(),
                    })
                    .expect("Mila stream error should serialize");
                    Some((
                        Ok(Event::default().event("error").data(payload)),
                        (chat_stream, true),
                    ))
                }
                None => Some((Ok(Event::default().data("[DONE]")), (chat_stream, true))),
            }
        },
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/mila/stream",
    params(MilaStreamParams),
    responses(
        (status = 200, description = "SSE stream of Mila chat deltas", content_type = "text/event-stream", body = String),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Session not found"),
        (status = 422, description = "Validation error"),
        (status = 503, description = "AI provider unavailable (code ai_provider_unavailable, includes Retry-After)"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["ai:use", "library:read"]))),
    tag = "Mila",
)]
pub async fn stream_chat(
    RequireAiUseAndLibraryRead {
        principal: auth_user,
        ..
    }: RequireAiUseAndLibraryRead,
    State(state): State<AppState>,
    Query(params): Query<MilaStreamParams>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    params.validate().map_err(validation_error)?;
    let request = params.into_state_request().map_err(validation_error)?;
    let stream = require_mila_chat_ops(&state)?
        .stream_chat(auth_user.user_id, request)
        .await
        .map_err(ApiError::from)?;

    Ok((
        crate::routes::sse::stream_headers(),
        Sse::new(into_sse_stream(stream)).keep_alive(KeepAlive::default()),
    ))
}
