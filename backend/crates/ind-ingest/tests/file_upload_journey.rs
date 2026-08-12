#![allow(clippy::unwrap_used)]

use bytes::Bytes;
use ind_application::AppError;
use ind_application::ports::{FileUploadProcessor, UploadFileProcessRequest};
use ind_domain::{ArchiveAssetKind, DocumentType, DomainError};
use ind_ingest::{DocumentFileUploadProcessor, PdfExtractionError, extract_pdf_text};
use pdf_oxide::writer::{DocumentBuilder, PageSize};

fn request(
    filename: &str,
    content_type: &str,
    data: impl Into<Bytes>,
    max_bytes: usize,
) -> UploadFileProcessRequest {
    UploadFileProcessRequest {
        filename: filename.into(),
        content_type: content_type.into(),
        data: data.into(),
        title_override: None,
        max_bytes,
    }
}

fn encrypted_pdf(user_password: &str) -> Vec<u8> {
    let mut builder = DocumentBuilder::new();
    builder
        .page(PageSize::Letter)
        .at(72.0, 720.0)
        .text("Protected upload")
        .done();
    builder
        .to_bytes_encrypted(user_password, "owner-secret")
        .unwrap()
}

fn pdf_with_unsupported_encryption_version() -> Vec<u8> {
    let mut pdf = encrypted_pdf("open-secret");
    let version = b"/V 5";
    let unsupported = b"/V 9";
    let offset = pdf
        .windows(version.len())
        .position(|window| window == version)
        .unwrap();
    pdf[offset..offset + unsupported.len()].copy_from_slice(unsupported);
    pdf
}

#[tokio::test]
async fn html_upload_validates_type_and_preserves_raw_while_sanitizing_reader_content() {
    let processor = DocumentFileUploadProcessor;
    let html = br#"<!doctype html><html><head><title> Surgical upload </title></head>
        <body><article><h1>Upload</h1><p>Real readable words survive the processor boundary.</p>
        <script>fetch('/secret')</script><img src="x" onerror="alert(1)"></article></body></html>"#;
    let processed = processor
        .process_upload(request(
            "article.htm",
            "application/octet-stream",
            html.as_slice(),
            1024,
        ))
        .await
        .unwrap();
    assert_eq!(processed.document_type, DocumentType::Article);
    assert_eq!(processed.original_extension, "html");
    assert_eq!(processed.title, "Surgical upload");
    assert!(processed.word_count.unwrap() >= 7);
    assert!(processed.reading_time_minutes.unwrap() >= 1);
    assert_eq!(processed.assets.len(), 2);
    let original = processed
        .assets
        .iter()
        .find(|asset| asset.asset_kind == Some(ArchiveAssetKind::OriginalUpload))
        .unwrap();
    let readable = processed
        .assets
        .iter()
        .find(|asset| asset.asset_kind == Some(ArchiveAssetKind::ReadableHtml))
        .unwrap();
    assert!(String::from_utf8_lossy(&original.bytes).contains("<script>"));
    let safe = String::from_utf8_lossy(&readable.bytes);
    assert!(safe.contains("Real readable words"));
    assert!(!safe.contains("<script"));
    assert!(!safe.contains("onerror"));

    for (filename, content_type, data, max_bytes, expected) in [
        (
            "bad.txt",
            "text/plain",
            b"plain".as_slice(),
            100,
            "unsupported content type",
        ),
        (
            "bad.pdf",
            "text/html",
            b"<p>html</p>".as_slice(),
            100,
            "invalid content type",
        ),
        (
            "bad.html",
            "text/html",
            b"\0binary".as_slice(),
            100,
            "file contents do not match",
        ),
        (
            "big.html",
            "text/html",
            b"<p>too large</p>".as_slice(),
            1,
            "payload too large",
        ),
    ] {
        let error = processor
            .process_upload(request(filename, content_type, data, max_bytes))
            .await
            .unwrap_err();
        assert!(error.to_string().contains(expected), "{filename}: {error}");
    }
}

#[tokio::test]
async fn password_protected_pdf_upload_is_rejected_before_admission() {
    let processor = DocumentFileUploadProcessor;

    let error = processor
        .process_upload(request(
            "protected.pdf",
            "application/pdf",
            encrypted_pdf("open-secret"),
            1024 * 1024,
        ))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        AppError::Domain(DomainError::Validation { field, message })
            if field == "file" && message == "Password-protected PDFs are not supported."
    ));
}

#[tokio::test]
async fn permission_only_encrypted_pdf_upload_remains_supported() {
    let processor = DocumentFileUploadProcessor;

    let processed = processor
        .process_upload(request(
            "permission-only.pdf",
            "application/pdf",
            encrypted_pdf(""),
            1024 * 1024,
        ))
        .await
        .unwrap();

    assert_eq!(processed.document_type, DocumentType::Pdf);
    assert!(processed.assets.iter().any(|asset| {
        asset.asset_kind == Some(ArchiveAssetKind::OriginalUpload)
            && !asset.bytes.is_empty()
            && asset.status == ind_domain::ArchiveAssetStatus::Completed
    }));
}

#[test]
fn unsupported_pdf_encryption_is_not_reported_as_password_protection() {
    let error = extract_pdf_text(&pdf_with_unsupported_encryption_version()).unwrap_err();

    assert!(
        matches!(error, PdfExtractionError::Parse(_)),
        "unexpected error: {error:?}"
    );
}
