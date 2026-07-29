use chrono::Utc;

use ind_application::AppError;
use ind_domain::{
    DocumentId, EmbedDocumentJob, ExtractEntitiesDocumentJob, GenericJobEnvelope,
    SuggestTagsDocumentJob, SummarizeDocumentJob, job_types,
};

use crate::context::{AiSearchJobDeps, IndexQueueContext};

pub async fn dispatch_generic_job(
    ctx: &AiSearchJobDeps,
    envelope: GenericJobEnvelope,
) -> Result<Option<()>, AppError> {
    match envelope.job_type.as_str() {
        job_types::DOCUMENT_AI_EMBED => {
            let job: EmbedDocumentJob = decode(envelope.payload)?;
            handle_embed_document(ctx, job.document_id).await?;
            Ok(Some(()))
        }
        job_types::DOCUMENT_AI_SUMMARIZE => {
            let job: SummarizeDocumentJob = decode(envelope.payload)?;
            ctx.ai_action_runner
                .summarize_document(job.document_id)
                .await?;
            Ok(Some(()))
        }
        job_types::DOCUMENT_AI_TAGS => {
            let job: SuggestTagsDocumentJob = decode(envelope.payload)?;
            ctx.ai_action_runner
                .suggest_tags_for_document(job.document_id)
                .await?;
            Ok(Some(()))
        }
        job_types::DOCUMENT_AI_ENTITIES => {
            let job: ExtractEntitiesDocumentJob = decode(envelope.payload)?;
            ctx.ai_action_runner
                .extract_entities_for_document(job.document_id)
                .await?;
            Ok(Some(()))
        }
        _ => Ok(None),
    }
}

async fn handle_embed_document(
    ctx: &AiSearchJobDeps,
    document_id: DocumentId,
) -> Result<(), AppError> {
    ctx.embedding_indexer.embed_document(document_id).await?;

    Ok(())
}

pub async fn enqueue_document_embed(
    ctx: &impl IndexQueueContext,
    document_id: DocumentId,
) -> Result<(), AppError> {
    enqueue_job(
        ctx,
        job_types::DOCUMENT_AI_EMBED,
        EmbedDocumentJob { document_id },
        format!("{}:{document_id}", job_types::DOCUMENT_AI_EMBED),
    )
    .await
}

pub async fn enqueue_document_embed_if_engaged(
    ctx: &impl IndexQueueContext,
    user_id: ind_domain::UserId,
    document_id: DocumentId,
) -> Result<(), AppError> {
    let Some(provenance) = ctx
        .document_repo()
        .load_provenance(user_id, document_id)
        .await?
    else {
        return Ok(());
    };
    if provenance.is_engaged_for_ai() {
        enqueue_document_ai_processing(ctx, document_id).await?;
    }
    Ok(())
}

pub async fn enqueue_document_ai_processing(
    ctx: &impl IndexQueueContext,
    document_id: DocumentId,
) -> Result<(), AppError> {
    enqueue_document_embed(ctx, document_id).await?;
    enqueue_document_summarize(ctx, document_id).await?;
    enqueue_document_tags(ctx, document_id).await?;
    enqueue_document_entities(ctx, document_id).await?;
    Ok(())
}

async fn enqueue_document_summarize(
    ctx: &impl IndexQueueContext,
    document_id: DocumentId,
) -> Result<(), AppError> {
    enqueue_job(
        ctx,
        job_types::DOCUMENT_AI_SUMMARIZE,
        SummarizeDocumentJob { document_id },
        format!("{}:{document_id}", job_types::DOCUMENT_AI_SUMMARIZE),
    )
    .await
}

async fn enqueue_document_tags(
    ctx: &impl IndexQueueContext,
    document_id: DocumentId,
) -> Result<(), AppError> {
    enqueue_job(
        ctx,
        job_types::DOCUMENT_AI_TAGS,
        SuggestTagsDocumentJob { document_id },
        format!("{}:{document_id}", job_types::DOCUMENT_AI_TAGS),
    )
    .await
}

async fn enqueue_document_entities(
    ctx: &impl IndexQueueContext,
    document_id: DocumentId,
) -> Result<(), AppError> {
    enqueue_job(
        ctx,
        job_types::DOCUMENT_AI_ENTITIES,
        ExtractEntitiesDocumentJob { document_id },
        format!("{}:{document_id}", job_types::DOCUMENT_AI_ENTITIES),
    )
    .await
}

fn decode<T: serde::de::DeserializeOwned>(payload: serde_json::Value) -> Result<T, AppError> {
    serde_json::from_value(payload).map_err(|err| AppError::Repository(Box::new(err)))
}

async fn enqueue_job<T: serde::Serialize>(
    ctx: &impl IndexQueueContext,
    job_type: &str,
    payload: T,
    dedupe_key: String,
) -> Result<(), AppError> {
    let payload =
        serde_json::to_value(payload).map_err(|err| AppError::Repository(Box::new(err)))?;
    ctx.outbox_repo()
        .enqueue(job_type, payload, Some(dedupe_key), Utc::now())
        .await?;
    Ok(())
}
