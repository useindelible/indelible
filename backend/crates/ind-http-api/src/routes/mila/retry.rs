use axum::extract::{Path, State};
use ind_application::ports::RetryMilaDocumentAction;

use crate::error::{ApiError, FieldError};
use crate::middleware::RequireAiUseAndLibraryRead;
use crate::response::ApiResponse;
use crate::state::AppState;

use super::dto::RetryMilaActionResponse;
use super::{require_mila_action_retry_ops, validation_error};

fn parse_retry_action(raw: &str) -> Result<RetryMilaDocumentAction, ApiError> {
    match raw {
        "summary" => Ok(RetryMilaDocumentAction::Summary),
        "tags" => Ok(RetryMilaDocumentAction::Tags),
        "entities" => Ok(RetryMilaDocumentAction::Entities),
        _ => Err(validation_error(vec![FieldError {
            field: "action".into(),
            message: "must be one of: summary, tags, entities".into(),
        }])),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/mila/documents/{document_id}/actions/{action}/retry",
    params(
        ("document_id" = String, Path, description = "Document id with doc_ prefix"),
        ("action" = String, Path, description = "Retryable Mila action: summary, tags, or entities"),
    ),
    responses(
        (status = 200, description = "Mila action retry queued", body = RetryMilaActionResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Document not found"),
        (status = 422, description = "Unsupported Mila action"),
        (status = 503, description = "Mila service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["ai:use", "library:read"]))),
    tag = "Mila",
)]
pub async fn retry_mila_document_action(
    RequireAiUseAndLibraryRead {
        principal: auth_user,
        ..
    }: RequireAiUseAndLibraryRead,
    State(state): State<AppState>,
    Path((document_id, raw_action)): Path<(String, String)>,
) -> Result<ApiResponse<RetryMilaActionResponse>, ApiError> {
    let document_id = document_id.parse().map_err(|_| ApiError::NotFound {
        entity: "Document",
        id: document_id,
    })?;
    let action = parse_retry_action(&raw_action)?;
    require_mila_action_retry_ops(&state)?
        .retry_document_action(auth_user.user_id, document_id, action)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(RetryMilaActionResponse {
        queued: true,
        action: raw_action,
    }))
}
