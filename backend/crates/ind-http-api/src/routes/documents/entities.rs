use ind_application::ports::EntityOperations;

use super::*;
use crate::routes::entities::EntitySummaryResponse;

fn require_entity_ops(state: &AppState) -> Result<&dyn EntityOperations, ApiError> {
    state
        .entity_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "entity service not configured".into(),
        })
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/entities",
    params(("document_id" = String, Path, description = "Document id with doc_ prefix")),
    responses(
        (status = 200, description = "Entities mentioned in the document", body = Vec<EntitySummaryResponse>),
        (status = 401, description = "Authentication required"),
        (status = 503, description = "Entity service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Documents",
)]
pub async fn list_document_entities(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
) -> Result<crate::extract::Json<Vec<EntitySummaryResponse>>, ApiError> {
    let ops = require_entity_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let summaries = ops
        .list_entities_for_document(auth_user.user_id, document_id)
        .await
        .map_err(ApiError::from)?;
    let items = summaries
        .into_iter()
        .map(EntitySummaryResponse::from_domain)
        .collect();
    Ok(crate::extract::Json(items))
}
