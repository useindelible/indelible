use chrono::Utc;
use ind_application::error::AppError;
use ind_domain::{DomainError, NotionExportDocumentJob};
use ind_integrations::notion::{
    HighlightText, NotionBlock, NotionClient, NotionError, NotionPageSpec,
    build_highlight_blocks_with_options, chunk_blocks_for_request, notion_settings_from_config,
};

use crate::context::NotionJobDeps;

const HIGHLIGHT_EXPORT_BATCH_SIZE: i64 = 500;

use super::auth::load_notion_access_token;
use super::managed_target::resolve_managed_target;
use super::support::{
    content_source_to_str, highlight_location, map_notion_error, requeue_after, triage_state_to_str,
};

pub async fn handle_export_document(
    deps: &NotionJobDeps,
    job: NotionExportDocumentJob,
) -> Result<(), AppError> {
    handle_export_document_with_highlight_batch_size(deps, job, HIGHLIGHT_EXPORT_BATCH_SIZE).await
}

#[cfg(any(test, feature = "test-helpers"))]
#[allow(dead_code)]
// allow: TASK-225 - exported for cross-crate integration tests via the test-helpers feature.
pub async fn handle_export_document_with_test_highlight_batch_size(
    deps: &NotionJobDeps,
    job: NotionExportDocumentJob,
    highlight_batch_size: i64,
) -> Result<(), AppError> {
    handle_export_document_with_highlight_batch_size(deps, job, highlight_batch_size).await
}

async fn handle_export_document_with_highlight_batch_size(
    deps: &NotionJobDeps,
    job: NotionExportDocumentJob,
    highlight_batch_size: i64,
) -> Result<(), AppError> {
    let cursor = deps
        .export_cursor_repo
        .upsert(job.connection_id, job.library_entry_id)
        .await?;

    let Some(connection) = deps
        .connection_repo
        .find_by_id(job.user_id, job.connection_id)
        .await?
    else {
        return Err(AppError::Domain(DomainError::NotFound {
            entity: "IntegrationConnection",
            id: job.connection_id.to_string(),
        }));
    };

    let access_token = match load_notion_access_token(deps, job.user_id).await {
        Ok(token) => token,
        Err(err) => {
            let error_message = err.to_string();
            let last_error = if matches!(err, AppError::Domain(DomainError::NotFound { .. })) {
                "no Notion OAuth token".to_string()
            } else {
                error_message
            };
            deps.export_cursor_repo
                .mark_attempted(
                    job.connection_id,
                    job.library_entry_id,
                    Utc::now(),
                    Some(last_error),
                )
                .await?;
            deps.connection_repo
                .set_last_error(
                    job.connection_id,
                    job.user_id,
                    Some("no Notion OAuth token".into()),
                )
                .await?;
            return Err(err);
        }
    };

    let Some(document) = deps
        .document_repo
        .find_by_id(job.user_id, job.document_id)
        .await?
    else {
        deps.export_cursor_repo
            .mark_attempted(
                job.connection_id,
                job.library_entry_id,
                Utc::now(),
                Some("document not found".into()),
            )
            .await?;
        return Ok(());
    };

    // An active library entry is the saved-content gate (TASK-236 AC#4): if the entry was
    // unsaved/removed since enqueue, mark the cursor synced and skip without exporting.
    let Some(entry) = deps
        .library_repo
        .find_active_by_document(job.user_id, job.document_id)
        .await?
    else {
        deps.export_cursor_repo
            .mark_synced(job.connection_id, job.library_entry_id, Utc::now())
            .await?;
        return Ok(());
    };

    let client = NotionClient::new(
        access_token,
        deps.notion_api_base.clone(),
        deps.rate_limiters.for_connection(job.connection_id),
    );

    let target = match resolve_managed_target(&connection, &client, deps).await {
        Ok(t) => t,
        Err(NotionError::RateLimited { retry_after_secs }) => {
            requeue_after(deps, &job, retry_after_secs).await?;
            return Ok(());
        }
        Err(e) => {
            deps.export_cursor_repo
                .mark_attempted(
                    job.connection_id,
                    job.library_entry_id,
                    Utc::now(),
                    Some(e.to_string()),
                )
                .await?;
            deps.connection_repo
                .set_last_error(job.connection_id, job.user_id, Some(e.to_string()))
                .await?;
            return Err(map_notion_error(e));
        }
    };

    let tags: Vec<String> = deps
        .tag_repo
        .list_by_library_entry(job.user_id, entry.id)
        .await?
        .into_iter()
        .map(|tag| tag.name)
        .collect();

    let page_spec = NotionPageSpec {
        indelible_id: job.document_id.to_string(),
        title: document.title.clone(),
        url: document
            .original_url
            .clone()
            .or_else(|| document.canonical_url.clone()),
        canonical_url: document.canonical_url.clone(),
        author: document.author.clone(),
        source: content_source_to_str(entry.source),
        saved_at: entry.saved_at,
        tags,
        item_type: document.document_type.as_str().to_string(),
        triage_state: triage_state_to_str(entry.triage_state),
        property_ids: target.property_ids.clone(),
    };

    let page_id = match client
        .upsert_page(
            &target.data_source_id,
            if job.replaced_page_id.as_deref() == cursor.remote_page_id.as_deref() {
                None
            } else {
                cursor.remote_page_id.as_deref()
            },
            &page_spec,
        )
        .await
    {
        Ok(id) => id,
        Err(NotionError::RateLimited { retry_after_secs }) => {
            requeue_after(deps, &job, retry_after_secs).await?;
            return Ok(());
        }
        Err(e) => {
            deps.export_cursor_repo
                .mark_attempted(
                    job.connection_id,
                    job.library_entry_id,
                    Utc::now(),
                    Some(e.to_string()),
                )
                .await?;
            deps.connection_repo
                .set_last_error(job.connection_id, job.user_id, Some(e.to_string()))
                .await?;
            return Err(map_notion_error(e));
        }
    };

    deps.export_cursor_repo
        .mark_remote_page_resolved(
            job.connection_id,
            job.library_entry_id,
            &page_id,
            Utc::now(),
        )
        .await?;

    let mut after_created_at = cursor.last_exported_highlight_created_at;
    let mut after_id = cursor.last_exported_highlight_id;
    let mut appended_any_highlights = after_created_at.is_some();

    loop {
        let highlights = deps
            .highlight_repo
            .list_by_document_after_cursor(
                job.document_id,
                job.user_id,
                after_created_at,
                after_id,
                highlight_batch_size,
            )
            .await?;

        if highlights.is_empty() {
            break;
        }

        let batch_len = highlights.len();
        let highlight_ids = highlights.iter().map(|h| h.id).collect::<Vec<_>>();
        let tags_by_highlight = deps
            .highlight_repo
            .list_tags_for_highlights(&highlight_ids, job.user_id)
            .await?;
        let mut highlight_texts = Vec::with_capacity(batch_len);
        for h in highlights {
            let note = deps
                .highlight_repo
                .get_note(h.id, job.user_id)
                .await?
                .map(|note| note.body);
            let tags = tags_by_highlight
                .get(&h.id)
                .map(|tags| tags.iter().map(|tag| tag.name.clone()).collect())
                .unwrap_or_default();
            let location = highlight_location(h.locator.as_ref(), h.source_locator.as_ref());
            highlight_texts.push(HighlightText {
                id: h.id,
                created_at: h.created_at,
                text: h.text_content,
                note,
                tags,
                location,
            });
        }

        let settings = notion_settings_from_config(&connection.config);
        let mut blocks = build_highlight_blocks_with_options(&highlight_texts, &settings);
        if appended_any_highlights && matches!(blocks.first(), Some(NotionBlock::Divider)) {
            blocks.remove(0);
        }

        for chunk in chunk_blocks_for_request(&blocks) {
            let last_highlight = chunk
                .iter()
                .rev()
                .find_map(|block| block.highlight_cursor());
            match client.append_blocks(&page_id, &chunk).await {
                Ok(()) => {
                    if let Some((created_at, highlight_id)) = last_highlight {
                        deps.export_cursor_repo
                            .mark_highlight_chunk_synced(
                                job.connection_id,
                                job.library_entry_id,
                                created_at,
                                highlight_id,
                                Utc::now(),
                            )
                            .await?;
                        after_created_at = Some(created_at);
                        after_id = Some(highlight_id);
                        appended_any_highlights = true;
                    }
                }
                Err(NotionError::RateLimited { retry_after_secs }) => {
                    requeue_after(deps, &job, retry_after_secs).await?;
                    return Ok(());
                }
                Err(e) => {
                    deps.export_cursor_repo
                        .mark_attempted(
                            job.connection_id,
                            job.library_entry_id,
                            Utc::now(),
                            Some(e.to_string()),
                        )
                        .await?;
                    deps.connection_repo
                        .set_last_error(job.connection_id, job.user_id, Some(e.to_string()))
                        .await?;
                    return Err(map_notion_error(e));
                }
            }
        }

        if batch_len < highlight_batch_size as usize {
            break;
        }
    }

    deps.export_cursor_repo
        .mark_synced(job.connection_id, job.library_entry_id, Utc::now())
        .await?;
    deps.connection_repo
        .set_last_error(job.connection_id, job.user_id, None)
        .await?;
    deps.connection_repo
        .set_last_sync_at(job.connection_id, job.user_id, Utc::now())
        .await?;

    Ok(())
}
