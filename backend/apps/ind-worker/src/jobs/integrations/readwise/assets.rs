use bytes::Bytes;
use ind_application::AppError;
use ind_application::storage::{ObjectStorage, UploadResult};
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, DocumentId, NewDocumentAsset, UserId};

use super::types::ZipEntry;
use crate::context::IntegrationJobDeps;
use crate::jobs::reading_metrics::{
    apply_reading_metrics, word_count_from_html, word_count_from_text,
};

/// Attach a Readwise ZIP entry as document-keyed asset(s). Returns true when a readable HTML asset
/// was attached, so the caller indexes/embeds the document. PDF is stored as a viewable document
/// asset and EPUB as its original upload; their readable extraction is handled by dedicated
/// document preparation paths.
pub(super) async fn attach_zip_asset_to_document(
    ctx: &IntegrationJobDeps,
    storage: &dyn ObjectStorage,
    user_id: UserId,
    document_id: DocumentId,
    entry: &ZipEntry,
) -> Result<bool, AppError> {
    match entry.extension.as_str() {
        "html" => {
            // Readwise export HTML is untrusted third-party content served inline in the reader;
            // sanitize before it is stored as ReadableHtml. The sanitize fallback keeps the
            // import alive when anchor preparation fails.
            let raw = String::from_utf8_lossy(&entry.bytes);
            let sanitized = ind_html::prepare_reader_html(&raw).unwrap_or_else(|err| {
                tracing::warn!(error = %err, "anchor preparation failed; storing sanitized only");
                ind_html::sanitize_reader_html(&raw)
            });
            put_document_asset(
                ctx,
                storage,
                user_id,
                document_id,
                ArchiveAssetKind::ReadableHtml,
                "readwise.html",
                "text/html",
                sanitized.as_bytes(),
            )
            .await?;
            apply_reading_metrics(
                ctx.document_repo.as_ref(),
                user_id,
                document_id,
                word_count_from_html(&sanitized),
            )
            .await;
            ind_application::handlers::article_toc::apply_article_toc(
                Some(storage),
                ctx.document_asset_repo.as_ref(),
                ctx.document_repo.as_ref(),
                document_id,
            )
            .await;
            Ok(true)
        }
        "pdf" => {
            let upload = put_document_asset(
                ctx,
                storage,
                user_id,
                document_id,
                ArchiveAssetKind::Pdf,
                "readwise.pdf",
                "application/pdf",
                &entry.bytes,
            )
            .await?;
            // The original upload points at the same stored object (no second upload).
            ctx.document_asset_repo
                .upsert_document_asset(NewDocumentAsset {
                    document_id,
                    asset_kind: ArchiveAssetKind::OriginalUpload,
                    s3_key: upload.key,
                    s3_bucket: upload.bucket,
                    content_type: "application/pdf".to_string(),
                    size_bytes: upload.size_bytes,
                    status: ArchiveAssetStatus::Completed,
                    failed_reason: None,
                })
                .await?;
            // PDFs get no readable HTML; reading metrics come from the embedded text layer.
            // Scanned/image-only PDFs yield no text and keep NULL metrics.
            if let Ok(text) = ind_ingest::extract_pdf_text(&entry.bytes) {
                apply_reading_metrics(
                    ctx.document_repo.as_ref(),
                    user_id,
                    document_id,
                    word_count_from_text(&text),
                )
                .await;
            }
            Ok(false)
        }
        "epub" => {
            put_document_asset(
                ctx,
                storage,
                user_id,
                document_id,
                ArchiveAssetKind::OriginalUpload,
                "readwise.epub",
                "application/epub+zip",
                &entry.bytes,
            )
            .await?;
            // Chapter word counts summed by the EPUB processor; a malformed EPUB keeps NULL
            // metrics rather than failing the import.
            if let Ok(processed) = ind_ingest::epub_processing::process_epub(&entry.bytes) {
                apply_reading_metrics(
                    ctx.document_repo.as_ref(),
                    user_id,
                    document_id,
                    processed.metadata.total_words as i32,
                )
                .await;
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

#[allow(clippy::too_many_arguments)]
async fn put_document_asset(
    ctx: &IntegrationJobDeps,
    storage: &dyn ObjectStorage,
    user_id: UserId,
    document_id: DocumentId,
    kind: ArchiveAssetKind,
    filename: &str,
    content_type: &str,
    bytes: &[u8],
) -> Result<UploadResult, AppError> {
    let key = format!("documents/{user_id}/{document_id}/{filename}");
    let upload = storage
        .upload(&key, content_type, Bytes::from(bytes.to_vec()))
        .await?;
    ctx.document_asset_repo
        .upsert_document_asset(NewDocumentAsset {
            document_id,
            asset_kind: kind,
            s3_key: upload.key.clone(),
            s3_bucket: upload.bucket.clone(),
            content_type: content_type.to_string(),
            size_bytes: upload.size_bytes,
            status: ArchiveAssetStatus::Completed,
            failed_reason: None,
        })
        .await?;
    Ok(upload)
}

pub(super) fn asset_count_for_extension(extension: &str) -> u32 {
    match extension {
        "html" => 1,
        "pdf" => 2,
        "epub" => 1,
        _ => 0,
    }
}
