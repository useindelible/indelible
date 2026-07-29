#![allow(clippy::unwrap_used)]

use bytes::Bytes;
use ind_application::ports::{FileUploadProcessor, UploadFileProcessRequest};
use ind_domain::{ArchiveAssetKind, DocumentType};
use ind_ingest::DocumentFileUploadProcessor;

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
