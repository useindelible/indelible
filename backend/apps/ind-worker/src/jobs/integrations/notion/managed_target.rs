use ind_integrations::notion::{
    NotionClient, NotionError, NotionManagedTarget, write_managed_target_to_config,
};

use crate::context::NotionJobDeps;

pub(super) fn has_cached_managed_target(config: &serde_json::Value) -> bool {
    config
        .get("database_id")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.trim().is_empty())
        && config
            .get("data_source_id")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.trim().is_empty())
}

pub(super) async fn resolve_managed_target(
    connection: &ind_domain::IntegrationConnection,
    client: &NotionClient,
    deps: &NotionJobDeps,
) -> Result<NotionManagedTarget, NotionError> {
    if let (Some(database_id), Some(data_source_id)) = (
        connection
            .config
            .get("database_id")
            .and_then(|v| v.as_str()),
        connection
            .config
            .get("data_source_id")
            .and_then(|v| v.as_str()),
    ) {
        match client
            .validate_managed_target(database_id, data_source_id)
            .await
        {
            Ok(target) => {
                let mut new_config = connection.config.clone();
                write_managed_target_to_config(&mut new_config, &target);
                if new_config != connection.config {
                    deps.connection_repo
                        .update_config(connection.id, connection.user_id, new_config)
                        .await
                        .map_err(|e| NotionError::State(e.to_string()))?;
                }
                return Ok(target);
            }
            Err(NotionError::Api { status: 404, .. }) => {
                return mark_managed_target_missing(connection.id, connection.user_id, deps).await;
            }
            Err(e) => return Err(e),
        }
    }

    resolve_managed_target_with_lock(connection, client, deps).await
}

async fn resolve_managed_target_with_lock(
    connection: &ind_domain::IntegrationConnection,
    client: &NotionClient,
    deps: &NotionJobDeps,
) -> Result<NotionManagedTarget, NotionError> {
    let _lock = deps
        .connection_repo
        .acquire_notion_managed_target_lock(connection.id)
        .await
        .map_err(|e| NotionError::State(e.to_string()))?;

    let latest = deps
        .connection_repo
        .find_by_id(connection.user_id, connection.id)
        .await
        .map_err(|e| NotionError::State(e.to_string()))?
        .ok_or_else(|| NotionError::State(format!("connection {} not found", connection.id)))?;

    if let (Some(database_id), Some(data_source_id)) = (
        latest.config.get("database_id").and_then(|v| v.as_str()),
        latest.config.get("data_source_id").and_then(|v| v.as_str()),
    ) {
        match client
            .validate_managed_target(database_id, data_source_id)
            .await
        {
            Ok(target) => {
                let mut new_config = latest.config.clone();
                write_managed_target_to_config(&mut new_config, &target);
                if new_config != latest.config {
                    deps.connection_repo
                        .update_config(latest.id, latest.user_id, new_config)
                        .await
                        .map_err(|e| NotionError::State(e.to_string()))?;
                }
                return Ok(target);
            }
            Err(NotionError::Api { status: 404, .. }) => {
                return mark_managed_target_missing(latest.id, latest.user_id, deps).await;
            }
            Err(e) => return Err(e),
        }
    }

    let target = client.find_or_create_database().await?;
    let mut new_config = latest.config.clone();
    write_managed_target_to_config(&mut new_config, &target);
    deps.connection_repo
        .update_config(latest.id, latest.user_id, new_config)
        .await
        .map_err(|e| NotionError::State(e.to_string()))?;
    Ok(target)
}

async fn mark_managed_target_missing<T>(
    connection_id: ind_domain::IntegrationConnectionId,
    user_id: ind_domain::UserId,
    deps: &NotionJobDeps,
) -> Result<T, NotionError> {
    let last_error =
        "Managed Notion database no longer exists; please reconnect to recreate".to_string();
    deps.connection_repo
        .set_last_error(connection_id, user_id, Some(last_error.clone()))
        .await
        .map_err(|e| NotionError::State(e.to_string()))?;
    if let Err(e) = deps
        .connection_repo
        .set_status(connection_id, user_id, "needs_attention")
        .await
    {
        tracing::warn!(error = %e, "failed to mark Notion connection needs_attention");
    }
    Err(NotionError::State(last_error))
}
