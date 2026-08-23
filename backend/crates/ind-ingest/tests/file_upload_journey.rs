#![allow(clippy::unwrap_used)]

use bytes::Bytes;
use ind_application::AppError;
use ind_application::ports::{FileUploadProcessor, UploadFileProcessRequest};
use ind_domain::{ArchiveAssetKind, DocumentType, DomainError};
use ind_ingest::{DocumentFileUploadProcessor, PdfExtractionError, extract_pdf_text};
use pdf_oxide::writer::{DocumentBuilder, PageSize};
use std::io::{Cursor, Write};

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

fn generated_pdf(text: Option<&str>) -> Vec<u8> {
    let mut builder = DocumentBuilder::new();
    let page = builder.page(PageSize::Letter);
    if let Some(text) = text {
        page.at(72.0, 720.0).text(text).done();
    } else {
        page.done();
    }
    builder.build().unwrap()
}

fn epub_archive(include_container: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut archive = zip::ZipWriter::new(Cursor::new(&mut bytes));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        archive.start_file("mimetype", options).unwrap();
        archive.write_all(b"application/epub+zip").unwrap();

        if include_container {
            archive
                .start_file("META-INF/container.xml", options)
                .unwrap();
            archive
                .write_all(
                    br#"<?xml version="1.0"?>
<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
                )
                .unwrap();
        }

        archive.start_file("OEBPS/content.opf", options).unwrap();
        archive
            .write_all(
                br#"<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Fixture Book</dc:title>
    <dc:creator>Fixture Author</dc:creator>
  </metadata>
  <manifest>
    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine><itemref idref="chapter"/></spine>
</package>"#,
            )
            .unwrap();

        archive.start_file("OEBPS/chapter.xhtml", options).unwrap();
        archive
            .write_all(b"<html><body><p>A readable EPUB chapter.</p></body></html>")
            .unwrap();
        archive.finish().unwrap();
    }
    bytes
}

fn without_eof(mut pdf: Vec<u8>) -> Vec<u8> {
    let eof = pdf
        .windows(b"%%EOF".len())
        .rposition(|window| window == b"%%EOF")
        .unwrap();
    pdf.truncate(eof);
    pdf
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
fn eof_truncated_pdf_is_classified_as_parse_failure() {
    let error = extract_pdf_text(&without_eof(generated_pdf(Some("Truncated tail")))).unwrap_err();

    assert!(
        matches!(error, PdfExtractionError::Parse(_)),
        "unexpected error: {error:?}"
    );
}

#[tokio::test]
async fn eof_truncated_pdf_upload_is_rejected_as_corrupted() {
    let processor = DocumentFileUploadProcessor;

    let error = processor
        .process_upload(request(
            "truncated.pdf",
            "application/pdf",
            without_eof(generated_pdf(Some("Truncated tail"))),
            1024 * 1024,
        ))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        AppError::Domain(DomainError::Validation { field, message })
            if field == "file"
                && message
                    == "This PDF is incomplete or corrupted. Choose a valid PDF and try again."
    ));
}

#[tokio::test]
async fn blank_pdf_upload_remains_supported_when_text_extraction_has_no_text() {
    let processor = DocumentFileUploadProcessor;

    let processed = processor
        .process_upload(request(
            "blank.pdf",
            "application/pdf",
            generated_pdf(None),
            1024 * 1024,
        ))
        .await
        .unwrap();

    assert_eq!(processed.document_type, DocumentType::Pdf);
    assert!(processed.assets.iter().any(|asset| {
        asset.asset_kind == Some(ArchiveAssetKind::OriginalUpload)
            && asset.status == ind_domain::ArchiveAssetStatus::Completed
    }));
    assert!(processed.assets.iter().any(|asset| {
        asset.asset_kind == Some(ArchiveAssetKind::ExtractedText)
            && asset.status == ind_domain::ArchiveAssetStatus::Failed
    }));
}

#[tokio::test]
async fn epub_without_container_is_rejected_as_invalid_file() {
    let processor = DocumentFileUploadProcessor;

    let error = processor
        .process_upload(request(
            "missing-container.epub",
            "application/epub+zip",
            epub_archive(false),
            1024 * 1024,
        ))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        AppError::Domain(DomainError::Validation { field, message })
            if field == "file"
                && message
                    == "This file is not a valid EPUB: missing META-INF/container.xml. Choose another EPUB file and try again."
    ));
}

#[tokio::test]
async fn valid_epub_upload_remains_supported() {
    let processor = DocumentFileUploadProcessor;

    let processed = processor
        .process_upload(request(
            "fixture.epub",
            "application/epub+zip",
            epub_archive(true),
            1024 * 1024,
        ))
        .await
        .unwrap();

    assert_eq!(processed.document_type, DocumentType::Book);
    assert_eq!(processed.title, "Fixture Book");
    assert!(processed.assets.iter().any(|asset| {
        asset.asset_kind == Some(ArchiveAssetKind::Epub)
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

/// Minimal single-page PDF whose Info dictionary carries `raw_title` verbatim,
/// written as a hex string. `DocumentBuilder::title` re-encodes whatever it is
/// given, so a hand-written file is the only way to control the exact bytes a
/// reader sees for `/Title`.
fn pdf_with_raw_info_title(raw_title: &[u8]) -> Vec<u8> {
    let hex: String = raw_title.iter().map(|byte| format!("{byte:02X}")).collect();
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>".to_string(),
        format!("<< /Title <{hex}> >>"),
    ];

    let mut pdf = Vec::from("%PDF-1.7\n");
    let mut offsets = Vec::with_capacity(objects.len());
    for (index, body) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", index + 1).as_bytes());
    }

    // The xref table indexes byte offsets of every object, so it can only be
    // written once the bodies above are laid out.
    let startxref = pdf.len();
    let size = objects.len() + 1;
    pdf.extend_from_slice(format!("xref\n0 {size}\n0000000000 65535 f \n").as_bytes());
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {size} /Root 1 0 R /Info 4 0 R >>\nstartxref\n{startxref}\n%%EOF\n"
        )
        .as_bytes(),
    );
    pdf
}

fn utf16be(text: &str) -> Vec<u8> {
    let mut out = vec![0xFE_u8, 0xFF];
    for unit in text.encode_utf16() {
        out.extend_from_slice(&unit.to_be_bytes());
    }
    out
}

fn utf16le(text: &str) -> Vec<u8> {
    let mut out = vec![0xFF_u8, 0xFE];
    for unit in text.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
}

async fn title_of(raw_info_title: &[u8]) -> String {
    DocumentFileUploadProcessor
        .process_upload(request(
            "book.pdf",
            "application/pdf",
            pdf_with_raw_info_title(raw_info_title),
            10 * 1024 * 1024,
        ))
        .await
        .unwrap()
        .title
}

#[tokio::test]
async fn pdf_utf16be_title_decodes_without_nul_bytes() {
    let expected = "System Design Interview An Insider\u{2019}s Guide";
    let title = title_of(&utf16be(expected)).await;
    assert!(
        !title.contains('\0'),
        "NUL would be rejected by Postgres: {title:?}"
    );
    assert_eq!(title, expected);
}

#[tokio::test]
async fn pdf_utf16le_title_decodes() {
    let expected = "Designing Data\u{2011}Intensive Applications";
    assert_eq!(title_of(&utf16le(expected)).await, expected);
}

#[tokio::test]
async fn pdf_pdfdocencoding_punctuation_decodes() {
    // 0x84 is EM DASH in PDFDocEncoding. As a lone byte it is invalid UTF-8,
    // so reading the string as UTF-8 turns it into U+FFFD.
    let raw = b"Clean Code \x84 A Handbook";
    assert_eq!(title_of(raw).await, "Clean Code \u{2014} A Handbook");
}

#[tokio::test]
async fn pdf_ascii_title_still_round_trips() {
    let expected = "Cracking the Coding Interview";
    assert_eq!(title_of(expected.as_bytes()).await, expected);
}

#[tokio::test]
async fn pdf_raw_utf8_title_is_accepted_leniently() {
    // Spec-violating but common in the wild: UTF-8 bytes with no BOM. The
    // crate decoder tries UTF-8 before falling back to PDFDocEncoding.
    let expected = "Caf\u{e9} Culture";
    assert_eq!(title_of(expected.as_bytes()).await, expected);
}
