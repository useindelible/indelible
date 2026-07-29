use std::collections::HashMap;

use bytes::Bytes;
use futures::StreamExt;
use ind_application::AppError;
use ind_application::ports::{FileUploadProcessor, UploadFileProcessRequest};
use ind_application::repos::document_reprocess::CompleteUploadReprocess;
use ind_application::repos::document_upload::StagedDocumentAsset;
use ind_domain::{
    ArchiveAssetKind, ArchiveAssetStatus, DomainError, NewDocumentAsset, PrepareDocumentJob,
    ReprocessDocumentJob, YoutubeIngestDocumentJob,
};
use ind_ingest::DocumentFileUploadProcessor;

use crate::context::CaptureJobDeps;

pub async fn handle_document_reprocess(
    ctx: &CaptureJobDeps,
    job: ReprocessDocumentJob,
) -> Result<(), AppError> {
    let document = ctx
        .document_repo
        .find_by_id(job.user_id, job.document_id)
        .await?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "Document",
                id: job.document_id.to_string(),
            })
        })?;

    if let Some(url) = document
        .canonical_url
        .clone()
        .or_else(|| document.original_url.clone())
    {
        if ind_application::dispatch::is_youtube_url(&url) {
            return crate::jobs::youtube::handle_youtube_ingest_document(
                ctx,
                YoutubeIngestDocumentJob {
                    document_id: job.document_id,
                    user_id: job.user_id,
                    url,
                },
            )
            .await;
        }
        return crate::jobs::feed::handle_prepare_document(
            &ctx.feed,
            PrepareDocumentJob {
                document_id: job.document_id,
                user_id: job.user_id,
                url,
            },
        )
        .await;
    }

    reprocess_upload(ctx, job, document.title).await
}

async fn reprocess_upload(
    ctx: &CaptureJobDeps,
    job: ReprocessDocumentJob,
    title: String,
) -> Result<(), AppError> {
    let original = ctx
        .document_asset_repo
        .find_by_document_and_kind(job.document_id, ArchiveAssetKind::OriginalUpload)
        .await?
        .filter(|asset| asset.status == ArchiveAssetStatus::Completed)
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "OriginalUpload",
                id: job.document_id.to_string(),
            })
        })?;
    let filename = match original.content_type.as_str() {
        "application/pdf" => "original_upload.pdf",
        "application/epub+zip" => "original_upload.epub",
        _ => {
            return Err(AppError::Domain(DomainError::Validation {
                field: "document_id".into(),
                message: "document original upload is not a PDF or EPUB".into(),
            }));
        }
    };
    let storage = ctx
        .object_storage
        .as_ref()
        .ok_or_else(|| AppError::ExternalService {
            service: "storage".into(),
            message: "object storage is not configured for document reprocessing".into(),
        })?;
    let object = storage.get_object(&original.s3_key).await?;
    let mut stream = object.body;
    let mut data = Vec::new();
    while let Some(chunk) = stream.next().await {
        data.extend_from_slice(&chunk.map_err(|err| AppError::Repository(Box::new(err)))?);
    }

    let processor = DocumentFileUploadProcessor;
    let processed = match processor
        .process_upload(UploadFileProcessRequest {
            filename: filename.into(),
            content_type: original.content_type.clone(),
            data: Bytes::from(data),
            title_override: Some(title),
            max_bytes: usize::MAX,
        })
        .await
    {
        Ok(processed) => processed,
        Err(error) => {
            mark_upload_failure(ctx, &job, &original.content_type, &error).await?;
            return Err(error);
        }
    };

    let prefix = original
        .s3_key
        .rsplit_once('/')
        .map(|(prefix, _)| format!("{prefix}/"))
        .unwrap_or_default();
    let mut uploaded_by_filename = HashMap::from([(
        filename.to_string(),
        (
            original.s3_key.clone(),
            original.s3_bucket.clone(),
            original.size_bytes,
        ),
    )]);
    let mut staged_assets = Vec::new();
    for asset in processed.assets {
        if asset.asset_kind == Some(ArchiveAssetKind::OriginalUpload) {
            continue;
        }
        let (s3_key, s3_bucket, size_bytes) =
            if asset.bytes.is_empty() && asset.status != ArchiveAssetStatus::Completed {
                (String::new(), String::new(), 0)
            } else if let Some(existing) = uploaded_by_filename.get(&asset.filename) {
                existing.clone()
            } else {
                let key = format!("{prefix}{}", asset.filename);
                let upload = storage
                    .upload(&key, &asset.content_type, asset.bytes)
                    .await?;
                let record = (upload.key, upload.bucket, upload.size_bytes);
                uploaded_by_filename.insert(asset.filename.clone(), record.clone());
                record
            };
        if let Some(asset_kind) = asset.asset_kind {
            staged_assets.push(StagedDocumentAsset {
                asset_kind,
                s3_key,
                s3_bucket,
                content_type: asset.content_type,
                size_bytes,
                status: asset.status,
                failed_reason: asset.failed_reason,
            });
        }
    }

    ctx.document_reprocess_repo
        .complete_upload(CompleteUploadReprocess {
            document_id: job.document_id,
            user_id: job.user_id,
            assets: staged_assets,
            word_count: processed.word_count,
            reading_time_minutes: processed.reading_time_minutes,
        })
        .await
}

async fn mark_upload_failure(
    ctx: &CaptureJobDeps,
    job: &ReprocessDocumentJob,
    content_type: &str,
    error: &AppError,
) -> Result<(), AppError> {
    let asset_kind = if content_type == "application/epub+zip" {
        ArchiveAssetKind::Epub
    } else {
        ArchiveAssetKind::ExtractedText
    };
    ctx.document_asset_repo
        .upsert_document_asset(NewDocumentAsset {
            document_id: job.document_id,
            asset_kind,
            s3_key: String::new(),
            s3_bucket: String::new(),
            content_type: if asset_kind == ArchiveAssetKind::Epub {
                "application/json".into()
            } else {
                "text/plain".into()
            },
            size_bytes: 0,
            status: ArchiveAssetStatus::Failed,
            failed_reason: Some(format!("document reprocess failed: {error}")),
        })
        .await?;
    Ok(())
}
