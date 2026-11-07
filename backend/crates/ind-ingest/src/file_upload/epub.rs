use bytes::Bytes;
use ind_application::AppError;
use ind_application::ports::{ProcessedUpload, ProcessedUploadAsset, UploadFileProcessRequest};
use ind_application::repos::epub_format::EPUB_CHAPTER_LIST_FORMAT_VERSION;
use ind_domain::{ArchiveAssetKind, DomainError, reading_time_minutes_from_words};

use super::{FileTypeInfo, failed_asset, filename_stem, upload_asset};
use crate::epub_processing::{EpubError, EpubMetadata, EpubTocResponse, process_epub};

pub(super) async fn process_epub_upload(
    request: UploadFileProcessRequest,
    file_type: FileTypeInfo,
) -> Result<ProcessedUpload, AppError> {
    let epub_data = request.data.clone();
    let processed = tokio::task::spawn_blocking(move || process_epub(&epub_data))
        .await
        .map_err(|err| AppError::ExternalService {
            service: "epub".into(),
            message: format!("epub processing task failed: {err}"),
        })?
        .map_err(|err| {
            AppError::Domain(DomainError::Validation {
                field: "file".into(),
                message: format!(
                    "This file is not a valid EPUB: {}. Choose another EPUB file and try again.",
                    match err {
                        EpubError::Invalid(reason) => reason,
                        EpubError::Zip(_) | EpubError::Io(_) =>
                            "the archive could not be read".into(),
                    }
                ),
            })
        })?;

    let toc_response = EpubTocResponse {
        metadata: processed.metadata.clone(),
        toc: processed.toc.clone(),
    };
    let toc_json = serde_json::to_vec(&toc_response).map_err(|err| AppError::ExternalService {
        service: "epub".into(),
        message: format!("failed to serialize epub toc: {err}"),
    })?;

    let mut assets = vec![
        upload_asset(
            Some(ArchiveAssetKind::OriginalUpload),
            &format!("original_upload.{}", file_type.extension),
            &request.content_type,
            request.data.clone(),
        ),
        ProcessedUploadAsset {
            format_version: epub_toc_format_version(&toc_response.metadata),
            ..upload_asset(
                Some(ArchiveAssetKind::Epub),
                "epub_toc.json",
                "application/json",
                Bytes::from(toc_json),
            )
        },
    ];
    for chapter in &processed.chapters {
        assets.push(upload_asset(
            None,
            &format!("epub_ch_{}.html", chapter.spine_index),
            "text/html",
            Bytes::from(chapter.html.clone()),
        ));
    }
    let extracted_text = processed
        .chapters
        .iter()
        .map(|chapter| ind_html::html_to_text(&chapter.html))
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    if extracted_text.is_empty() {
        assets.push(failed_asset(
            ArchiveAssetKind::ExtractedText,
            "extracted.txt",
            "text/plain",
            "EPUB text extraction produced no readable chapters",
        ));
    } else {
        assets.push(upload_asset(
            Some(ArchiveAssetKind::ExtractedText),
            "extracted.txt",
            "text/plain",
            Bytes::from(extracted_text.into_bytes()),
        ));
    }
    if let Some(cover) =
        super::cover_image::extract_cover_image_async(&request.content_type, request.data).await
    {
        assets.push(upload_asset(
            Some(ArchiveAssetKind::Thumbnail),
            "thumbnail.png",
            cover.content_type,
            cover.bytes,
        ));
    }

    let word_count = i32::try_from(processed.metadata.total_words)
        .ok()
        .filter(|count| *count > 0);
    Ok(ProcessedUpload {
        document_type: file_type.document_type,
        original_extension: file_type.extension,
        title: request
            .title_override
            .or(processed.metadata.title)
            .unwrap_or_else(|| filename_stem(&request.filename)),
        author: processed.metadata.author,
        word_count,
        reading_time_minutes: word_count.map(reading_time_minutes_from_words),
        assets,
    })
}

pub(super) fn epub_toc_format_version(metadata: &EpubMetadata) -> i32 {
    if metadata.chapters.is_some() {
        EPUB_CHAPTER_LIST_FORMAT_VERSION
    } else {
        1
    }
}
