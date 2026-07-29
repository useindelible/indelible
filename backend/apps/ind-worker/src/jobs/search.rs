use chrono::Utc;

use ind_application::AppError;
use ind_application::repos::search_reindex::SearchReindexCursor;
use ind_domain::{DocumentId, GenericJobEnvelope, SearchReindexAllJob, SearchReindexDocumentJob};

use crate::context::{AiSearchJobDeps, IndexQueueContext};

pub async fn dispatch_generic_job(
    ctx: &AiSearchJobDeps,
    envelope: GenericJobEnvelope,
) -> Result<Option<()>, AppError> {
    match envelope.job_type.as_str() {
        "search.reindex_document" => {
            let job: SearchReindexDocumentJob = serde_json::from_value(envelope.payload)
                .map_err(|err| AppError::Repository(Box::new(err)))?;
            handle_search_reindex_document(ctx, job).await?;
            Ok(Some(()))
        }
        "search.reindex_all" => {
            let job: SearchReindexAllJob = serde_json::from_value(envelope.payload)
                .map_err(|err| AppError::Repository(Box::new(err)))?;
            handle_search_reindex_all(ctx, job).await?;
            Ok(Some(()))
        }
        _ => Ok(None),
    }
}

pub async fn enqueue_search_reindex_document(
    ctx: &impl IndexQueueContext,
    document_id: DocumentId,
) -> Result<(), AppError> {
    let payload = serde_json::to_value(SearchReindexDocumentJob { document_id })
        .map_err(|err| AppError::Repository(Box::new(err)))?;
    ctx.outbox_repo()
        .enqueue(
            "search.reindex_document",
            payload,
            Some(format!("search.reindex_document:{document_id}")),
            Utc::now(),
        )
        .await?;
    Ok(())
}

pub async fn handle_search_reindex_document(
    ctx: &AiSearchJobDeps,
    job: SearchReindexDocumentJob,
) -> Result<(), AppError> {
    ctx.search_indexer.reindex_document(job.document_id).await
}

pub async fn handle_search_reindex_all(
    ctx: &AiSearchJobDeps,
    job: SearchReindexAllJob,
) -> Result<(), AppError> {
    let page_size = i64::from(job.page_size.unwrap_or(250).clamp(1, 1000));

    walk_documents(ctx, page_size, job.target_version).await?;
    if let Some(version) = job.target_version {
        ctx.search_reindex_repo.complete_version(version).await?;
    }

    Ok(())
}

async fn walk_documents(
    ctx: &AiSearchJobDeps,
    page_size: i64,
    target_version: Option<i32>,
) -> Result<(), AppError> {
    let mut cursor = match target_version {
        Some(version) => ctx.search_reindex_repo.load_version_cursor(version).await?,
        None => None,
    };
    loop {
        let rows = ctx
            .document_repo
            .list_ids_for_reindex(
                cursor.map(|value| value.created_at),
                cursor.map(|value| value.document_id),
                page_size,
            )
            .await?;
        if rows.is_empty() {
            break;
        }
        for (document_id, _) in &rows {
            if let Err(error) = ctx.search_indexer.reindex_document(*document_id).await {
                tracing::warn!(
                    document_id = %document_id,
                    error = %error,
                    "full search reindex deferred a failed document to its retry queue"
                );
                enqueue_search_reindex_document(ctx, *document_id).await?;
            }
        }
        match rows.last() {
            Some((document_id, created_at)) => {
                let next = SearchReindexCursor {
                    created_at: *created_at,
                    document_id: document_id.into_uuid(),
                };
                if let Some(version) = target_version {
                    ctx.search_reindex_repo
                        .checkpoint_version_cursor(version, next)
                        .await?;
                }
                cursor = Some(next);
            }
            None => break,
        }
        if rows.len() < page_size as usize {
            break;
        }
    }
    Ok(())
}
