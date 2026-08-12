use axum::extract::{Path, Query, State};
use ind_domain::{IntegrationConnectionId, LibraryEntryId};

use crate::error::{ApiError, FieldError};
use crate::extract::ValidatedJson;
use crate::middleware::{RequireIntegrationsRead, RequireIntegrationsWrite};
use crate::response::{ApiResponse, EmptyResponse};
use crate::state::AppState;

use super::dto::{
    ListNotionExportItemsQuery, NotionExportItemSelectionDto, NotionExportItemsResponse,
    NotionRefreshItemResponse, NotionSettingsDto, UpdateNotionExportItemsRequest,
    UpdateNotionSettingsRequest,
};

#[utoipa::path(
    get,
    path = "/api/v1/integrations/{id}/notion/settings",
    params(("id" = String, Path, description = "Integration connection ID")),
    responses((status = 200, description = "Notion export settings", body = NotionSettingsDto)),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["integrations:read"]))),
    tag = "Integrations",
)]
pub async fn get_notion_settings(
    RequireIntegrationsRead {
        principal: auth_user,
        ..
    }: RequireIntegrationsRead,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<ApiResponse<NotionSettingsDto>, ApiError> {
    let parsed_id: IntegrationConnectionId = id.parse().map_err(|_| ApiError::NotFound {
        entity: "integration",
        id: id.clone(),
    })?;
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: id.clone(),
    })?;
    let settings = ops
        .get_notion_settings(auth_user.user_id, parsed_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(NotionSettingsDto {
        export_automatically: settings.export_automatically,
        include_highlight_locations: settings.include_highlight_locations,
        compact_layout: settings.compact_layout,
        selection_enabled: settings.selection_enabled,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/integrations/{id}/notion/settings",
    params(("id" = String, Path, description = "Integration connection ID")),
    request_body = UpdateNotionSettingsRequest,
    responses((status = 200, description = "Updated Notion export settings", body = NotionSettingsDto)),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["integrations:write"]))),
    tag = "Integrations",
)]
pub async fn update_notion_settings(
    RequireIntegrationsWrite {
        principal: auth_user,
        ..
    }: RequireIntegrationsWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateNotionSettingsRequest>,
) -> Result<ApiResponse<NotionSettingsDto>, ApiError> {
    let parsed_id: IntegrationConnectionId = id.parse().map_err(|_| ApiError::NotFound {
        entity: "integration",
        id: id.clone(),
    })?;
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: id.clone(),
    })?;
    let current = ops
        .get_notion_settings(auth_user.user_id, parsed_id)
        .await
        .map_err(ApiError::from)?;
    let next = ind_domain::NotionExportSettings {
        export_automatically: body
            .export_automatically
            .unwrap_or(current.export_automatically),
        include_highlight_locations: body
            .include_highlight_locations
            .unwrap_or(current.include_highlight_locations),
        compact_layout: body.compact_layout.unwrap_or(current.compact_layout),
        selection_enabled: body.selection_enabled.unwrap_or(current.selection_enabled),
    };
    let settings = ops
        .update_notion_settings(auth_user.user_id, parsed_id, next)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(NotionSettingsDto {
        export_automatically: settings.export_automatically,
        include_highlight_locations: settings.include_highlight_locations,
        compact_layout: settings.compact_layout,
        selection_enabled: settings.selection_enabled,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/integrations/{id}/notion/export-entries",
    params(("id" = String, Path, description = "Integration connection ID"), ListNotionExportItemsQuery),
    responses((status = 200, description = "Notion export item selection list", body = NotionExportItemsResponse)),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["integrations:read"]))),
    tag = "Integrations",
)]
pub async fn list_notion_export_items(
    RequireIntegrationsRead {
        principal: auth_user,
        ..
    }: RequireIntegrationsRead,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ListNotionExportItemsQuery>,
) -> Result<ApiResponse<NotionExportItemsResponse>, ApiError> {
    let parsed_id: IntegrationConnectionId = id.parse().map_err(|_| ApiError::NotFound {
        entity: "integration",
        id: id.clone(),
    })?;
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: id.clone(),
    })?;
    let page = ops
        .list_notion_export_items(
            auth_user.user_id,
            parsed_id,
            query.q,
            query.limit,
            query.offset,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(NotionExportItemsResponse {
        items: page.items.into_iter().map(Into::into).collect(),
        total_count: page.total_count,
        filtered_count: page.filtered_count,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/integrations/{id}/notion/export-entries",
    params(("id" = String, Path, description = "Integration connection ID")),
    request_body = UpdateNotionExportItemsRequest,
    responses((status = 204, description = "Selection updated")),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["integrations:write"]))),
    tag = "Integrations",
)]
pub async fn update_notion_export_items(
    RequireIntegrationsWrite {
        principal: auth_user,
        ..
    }: RequireIntegrationsWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateNotionExportItemsRequest>,
) -> Result<EmptyResponse, ApiError> {
    let parsed_id: IntegrationConnectionId = id.parse().map_err(|_| ApiError::NotFound {
        entity: "integration",
        id: id.clone(),
    })?;
    validate_notion_export_item_selections(&body.selections)?;
    let selections = body
        .selections
        .into_iter()
        .map(|s| {
            s.library_entry_id
                .parse()
                .map(|library_entry_id| (library_entry_id, s.selected))
                .map_err(|_| ApiError::NotFound {
                    entity: "library_entry",
                    id: s.library_entry_id,
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: id.clone(),
    })?;
    ops.update_notion_export_items(auth_user.user_id, parsed_id, selections)
        .await
        .map_err(ApiError::from)?;
    Ok(EmptyResponse)
}

fn validate_notion_export_item_selections(
    selections: &[NotionExportItemSelectionDto],
) -> Result<(), ApiError> {
    const MAX_NOTION_EXPORT_SELECTIONS: usize = 200;
    if selections.len() > MAX_NOTION_EXPORT_SELECTIONS {
        return Err(ApiError::ValidationError {
            errors: vec![FieldError {
                field: "selections".to_string(),
                message: format!("must contain at most {MAX_NOTION_EXPORT_SELECTIONS} items"),
            }],
        });
    }

    let mut seen = std::collections::HashSet::with_capacity(selections.len());
    for selection in selections {
        if !seen.insert(selection.library_entry_id.as_str()) {
            return Err(ApiError::ValidationError {
                errors: vec![FieldError {
                    field: "selections.library_entry_id".to_string(),
                    message: "library_entry_id values must be unique".to_string(),
                }],
            });
        }
    }

    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/v1/integrations/{id}/notion/export-entries/{library_entry_id}/refresh",
    params(
        ("id" = String, Path, description = "Integration connection ID"),
        ("library_entry_id" = String, Path, description = "Library entry ID to refresh")
    ),
    responses((status = 200, description = "Prior Notion page archived and replacement queued", body = NotionRefreshItemResponse)),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["integrations:write"]))),
    tag = "Integrations",
)]
pub async fn refresh_notion_export_item(
    RequireIntegrationsWrite {
        principal: auth_user,
        ..
    }: RequireIntegrationsWrite,
    State(state): State<AppState>,
    Path((id, library_entry_id)): Path<(String, String)>,
) -> Result<ApiResponse<NotionRefreshItemResponse>, ApiError> {
    let parsed_id: IntegrationConnectionId = id.parse().map_err(|_| ApiError::NotFound {
        entity: "integration",
        id: id.clone(),
    })?;
    let parsed_library_entry_id: LibraryEntryId =
        library_entry_id.parse().map_err(|_| ApiError::NotFound {
            entity: "library_entry",
            id: library_entry_id.clone(),
        })?;
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: id.clone(),
    })?;
    let refreshed = ops
        .refresh_notion_export_item(auth_user.user_id, parsed_id, parsed_library_entry_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(NotionRefreshItemResponse {
        library_entry_id,
        job_id: refreshed.job_id,
        archived_page_url: refreshed.archived_page_url,
    }))
}
