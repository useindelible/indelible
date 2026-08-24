use bytes::Bytes;
use ind_application::AppError;
use ind_application::ports::{
    FileUploadProcessor, ProcessedUpload, ProcessedUploadAsset, UploadFileProcessRequest,
};
use ind_domain::{
    ArchiveAssetKind, ArchiveAssetStatus, DocumentType, DomainError,
    reading_time_minutes_from_words,
};

use crate::epub_processing::{EpubError, EpubTocResponse, process_epub};
use crate::pdf_extraction::{PdfExtractionError, extract_pdf_text};

mod cover_image;

#[derive(Default, Clone, Copy)]
pub struct DocumentFileUploadProcessor;

#[derive(Debug, Clone, Copy)]
struct FileTypeInfo {
    document_type: DocumentType,
    extension: &'static str,
}

#[derive(Default)]
struct ExtractedMetadata {
    title: Option<String>,
    author: Option<String>,
}

#[async_trait::async_trait]
impl FileUploadProcessor for DocumentFileUploadProcessor {
    async fn process_upload(
        &self,
        request: UploadFileProcessRequest,
    ) -> Result<ProcessedUpload, AppError> {
        process_upload(request).await
    }
}

async fn process_upload(
    mut request: UploadFileProcessRequest,
) -> Result<ProcessedUpload, AppError> {
    let file_type = validate_content_type(&request.content_type, &request.filename)?;
    verify_magic_bytes(&request.data, file_type)?;
    request.content_type = canonical_upload_content_type(&request.content_type, file_type);
    if request.data.len() > request.max_bytes {
        return Err(AppError::ExternalService {
            service: "upload".into(),
            message: "payload too large".into(),
        });
    }

    match file_type.document_type {
        DocumentType::Pdf => process_pdf(request, file_type).await,
        DocumentType::Book => process_epub_upload(request, file_type).await,
        DocumentType::Article => process_html(request, file_type).await,
        _ => Err(AppError::Domain(DomainError::Validation {
            field: "content_type".into(),
            message: "unsupported upload document type".into(),
        })),
    }
}

async fn process_pdf(
    request: UploadFileProcessRequest,
    file_type: FileTypeInfo,
) -> Result<ProcessedUpload, AppError> {
    let data = request.data.clone();
    let metadata = tokio::task::spawn_blocking(move || extract_pdf_metadata(&data))
        .await
        .unwrap_or_default();

    let text_data = request.data.clone();
    let extracted_text = tokio::task::spawn_blocking(move || extract_pdf_text(&text_data))
        .await
        .map_err(|err| AppError::ExternalService {
            service: "pdf".into(),
            message: format!("PDF extraction task failed: {err}"),
        })?;
    if matches!(extracted_text, Err(PdfExtractionError::PasswordProtected)) {
        return Err(AppError::Domain(DomainError::Validation {
            field: "file".into(),
            message: "Password-protected PDFs are not supported.".into(),
        }));
    }
    if matches!(extracted_text, Err(PdfExtractionError::Parse(_))) {
        return Err(AppError::Domain(DomainError::Validation {
            field: "file".into(),
            message: "This PDF is incomplete or corrupted. Choose a valid PDF and try again."
                .into(),
        }));
    }
    let word_count = extracted_text
        .as_ref()
        .ok()
        .map(|text| word_count_from_text(text) as i32)
        .filter(|count| *count > 0);

    let mut assets = vec![
        upload_asset(
            Some(ArchiveAssetKind::OriginalUpload),
            &format!("original_upload.{}", file_type.extension),
            &request.content_type,
            request.data.clone(),
        ),
        upload_asset(
            Some(ArchiveAssetKind::Pdf),
            &format!("original_upload.{}", file_type.extension),
            &request.content_type,
            request.data.clone(),
        ),
    ];
    match extracted_text {
        Ok(text) if !text.trim().is_empty() => assets.push(upload_asset(
            Some(ArchiveAssetKind::ExtractedText),
            "extracted.txt",
            "text/plain",
            Bytes::from(text.into_bytes()),
        )),
        Ok(_) => assets.push(failed_asset(
            ArchiveAssetKind::ExtractedText,
            "extracted.txt",
            "text/plain",
            "PDF text extraction produced no text",
        )),
        Err(err) => assets.push(failed_asset(
            ArchiveAssetKind::ExtractedText,
            "extracted.txt",
            "text/plain",
            &format!("PDF text extraction failed: {err}"),
        )),
    }
    if let Some(cover) =
        cover_image::extract_cover_image_async(&request.content_type, request.data).await
    {
        assets.push(upload_asset(
            Some(ArchiveAssetKind::Thumbnail),
            "thumbnail.png",
            cover.content_type,
            cover.bytes,
        ));
    }

    Ok(ProcessedUpload {
        document_type: file_type.document_type,
        original_extension: file_type.extension,
        title: request
            .title_override
            .or(metadata.title)
            .unwrap_or_else(|| filename_stem(&request.filename)),
        author: metadata.author,
        word_count,
        reading_time_minutes: word_count.map(reading_time_minutes_from_words),
        assets,
    })
}

async fn process_epub_upload(
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
        upload_asset(
            Some(ArchiveAssetKind::Epub),
            "epub_toc.json",
            "application/json",
            Bytes::from(toc_json),
        ),
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
        cover_image::extract_cover_image_async(&request.content_type, request.data).await
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

async fn process_html(
    request: UploadFileProcessRequest,
    file_type: FileTypeInfo,
) -> Result<ProcessedUpload, AppError> {
    let raw_html = String::from_utf8_lossy(&request.data);
    let metadata = extract_html_metadata(&request.data);
    let text = ind_html::html_to_text(&raw_html);
    let word_count = Some(word_count_from_text(&text) as i32).filter(|count| *count > 0);
    // The OriginalUpload asset keeps the raw bytes (archive copy, served as a
    // download). The ReadableHtml asset is rendered in-app, so it must be
    // sanitized at storage time — asset_proxy relies on that invariant. The
    // sanitize fallback keeps ingest alive when anchor preparation fails.
    let sanitized_html = ind_html::prepare_reader_html(&raw_html).unwrap_or_else(|err| {
        tracing::warn!(error = %err, "anchor preparation failed; storing sanitized only");
        ind_html::sanitize_reader_html(&raw_html)
    });
    let assets = vec![
        upload_asset(
            Some(ArchiveAssetKind::OriginalUpload),
            &format!("original_upload.{}", file_type.extension),
            &request.content_type,
            request.data.clone(),
        ),
        upload_asset(
            Some(ArchiveAssetKind::ReadableHtml),
            "readable_html.html",
            "text/html",
            Bytes::from(sanitized_html.into_bytes()),
        ),
    ];

    Ok(ProcessedUpload {
        document_type: file_type.document_type,
        original_extension: file_type.extension,
        title: request
            .title_override
            .or(metadata.title)
            .unwrap_or_else(|| filename_stem(&request.filename)),
        author: metadata.author,
        word_count,
        reading_time_minutes: word_count.map(reading_time_minutes_from_words),
        assets,
    })
}

fn upload_asset(
    asset_kind: Option<ArchiveAssetKind>,
    filename: &str,
    content_type: &str,
    bytes: Bytes,
) -> ProcessedUploadAsset {
    ProcessedUploadAsset {
        asset_kind,
        filename: filename.to_string(),
        content_type: content_type.to_string(),
        bytes,
        status: ArchiveAssetStatus::Completed,
        failed_reason: None,
    }
}

fn failed_asset(
    asset_kind: ArchiveAssetKind,
    filename: &str,
    content_type: &str,
    failed_reason: &str,
) -> ProcessedUploadAsset {
    ProcessedUploadAsset {
        asset_kind: Some(asset_kind),
        filename: filename.to_string(),
        content_type: content_type.to_string(),
        bytes: Bytes::new(),
        status: ArchiveAssetStatus::Failed,
        failed_reason: Some(failed_reason.to_string()),
    }
}

fn validate_content_type(content_type: &str, filename: &str) -> Result<FileTypeInfo, AppError> {
    let extension = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    let expected = file_type_from_content_type(content_type).or_else(|| {
        if matches!(content_type, "" | "application/octet-stream") {
            file_type_from_extension(&extension)
        } else {
            None
        }
    });
    let Some(expected) = expected else {
        return Err(AppError::Domain(DomainError::Validation {
            field: "content_type".into(),
            message: format!(
                "unsupported content type '{content_type}'; supported: application/pdf, application/epub+zip, text/html"
            ),
        }));
    };

    if extension != expected.extension && !(expected.extension == "html" && extension == "htm") {
        return Err(AppError::Domain(DomainError::Validation {
            field: "content_type".into(),
            message: format!(
                "invalid content type '{content_type}' for file extension '.{extension}'"
            ),
        }));
    }

    Ok(expected)
}

/// Defense-in-depth against type-confusion / polyglot uploads: the declared
/// content type and extension already agreed in `validate_content_type`, but a
/// client can lie about both. Confirm the bytes actually look like the claimed
/// type (via `infer`'s magic-number database) before we process or store them.
/// HTML has no single magic number, so for `text/html` we require the content to
/// both look like text (no NUL bytes) and not be a format `infer` positively
/// identifies as non-text — i.e. reject binary/container masquerades and obvious
/// binary garbage relabeled as `text/html`.
fn verify_magic_bytes(data: &[u8], file_type: FileTypeInfo) -> Result<(), AppError> {
    let matches = match file_type.document_type {
        DocumentType::Pdf => infer::is_mime(data, "application/pdf"),
        DocumentType::Book => {
            infer::is_mime(data, "application/epub+zip") || infer::is_mime(data, "application/zip")
        }
        DocumentType::Article => {
            looks_like_text(data)
                && infer::get(data)
                    .is_none_or(|kind| kind.matcher_type() == infer::MatcherType::Text)
        }
        _ => true,
    };
    if matches {
        Ok(())
    } else {
        Err(AppError::Domain(DomainError::Validation {
            field: "content_type".into(),
            message: "file contents do not match the declared content type".into(),
        }))
    }
}

/// Cheap text heuristic: a NUL byte in the leading window is a reliable binary
/// indicator that plain text / HTML never contains.
fn looks_like_text(data: &[u8]) -> bool {
    let window = &data[..data.len().min(8192)];
    !window.contains(&0)
}

fn file_type_from_content_type(content_type: &str) -> Option<FileTypeInfo> {
    match content_type {
        "application/pdf" => Some(FileTypeInfo {
            document_type: DocumentType::Pdf,
            extension: "pdf",
        }),
        "application/epub+zip" => Some(FileTypeInfo {
            document_type: DocumentType::Book,
            extension: "epub",
        }),
        "text/html" => Some(FileTypeInfo {
            document_type: DocumentType::Article,
            extension: "html",
        }),
        _ => None,
    }
}

fn file_type_from_extension(extension: &str) -> Option<FileTypeInfo> {
    match extension {
        "pdf" => Some(FileTypeInfo {
            document_type: DocumentType::Pdf,
            extension: "pdf",
        }),
        "epub" => Some(FileTypeInfo {
            document_type: DocumentType::Book,
            extension: "epub",
        }),
        "html" | "htm" => Some(FileTypeInfo {
            document_type: DocumentType::Article,
            extension: "html",
        }),
        _ => None,
    }
}

fn canonical_upload_content_type(content_type: &str, file_type: FileTypeInfo) -> String {
    if matches!(content_type, "" | "application/octet-stream") {
        match file_type.document_type {
            DocumentType::Pdf => "application/pdf",
            DocumentType::Book => "application/epub+zip",
            DocumentType::Article => "text/html",
            _ => content_type,
        }
        .to_string()
    } else {
        content_type.to_string()
    }
}

fn extract_pdf_metadata(data: &[u8]) -> ExtractedMetadata {
    use pdf_oxide::document::PdfDocument;
    use pdf_oxide::extractors::xmp::XmpExtractor;
    use pdf_oxide::object::Object;

    let doc = match PdfDocument::from_bytes(data.to_vec()) {
        Ok(doc) => doc,
        Err(err) => {
            tracing::debug!(error = %err, "failed to parse PDF for metadata extraction");
            return ExtractedMetadata::default();
        }
    };

    let mut meta = ExtractedMetadata::default();
    if let Some(info_ref) = doc
        .trailer()
        .as_dict()
        .and_then(|trailer| trailer.get("Info"))
        .and_then(Object::as_reference)
        && let Ok(info_obj) = doc.load_object(info_ref)
        && let Some(info_dict) = info_obj.as_dict()
    {
        if let Some(title_obj) = info_dict.get("Title") {
            meta.title = pdf_object_to_string(title_obj);
        }
        if let Some(author_obj) = info_dict.get("Author") {
            meta.author = pdf_object_to_string(author_obj);
        }
    }

    if (meta.title.is_none() || meta.author.is_none())
        && let Ok(Some(xmp)) = XmpExtractor::extract(&doc)
    {
        if meta.title.is_none() {
            meta.title = xmp.dc_title.and_then(non_empty_string);
        }
        if meta.author.is_none() {
            meta.author = xmp.dc_creator.into_iter().find_map(non_empty_string);
        }
    }

    meta
}

/// PDF text strings are UTF-16 with a BOM or PDFDocEncoding (ISO 32000-2
/// §7.9.2); `pdf_oxide` hands them back undecoded. Reading them as UTF-8
/// turns a UTF-16BE title into a string interleaved with NULs.
fn pdf_object_to_string(obj: &pdf_oxide::object::Object) -> Option<String> {
    obj.as_string()
        .map(pdf_oxide::optional_content::decode_pdf_text_string)
        .map(|value| value.trim().to_string())
        .and_then(non_empty_string)
}

fn non_empty_string(value: String) -> Option<String> {
    let value = value.trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn extract_html_metadata(data: &[u8]) -> ExtractedMetadata {
    let html = String::from_utf8_lossy(data);
    ExtractedMetadata {
        title: extract_html_title(&html),
        author: None,
    }
}

fn extract_html_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let after = &html[start..];
    let content_start = after.find('>')? + 1;
    let remaining = &after[content_start..];
    let end = remaining.to_ascii_lowercase().find("</title>")?;
    let title = remaining[..end].trim().to_string();
    if title.is_empty() { None } else { Some(title) }
}

fn filename_stem(filename: &str) -> String {
    match filename.rfind('.') {
        Some(pos) if pos > 0 => filename[..pos].to_string(),
        _ => filename.to_string(),
    }
}

fn word_count_from_text(text: &str) -> usize {
    text.split_whitespace()
        .filter(|word| !word.is_empty())
        .count()
}
