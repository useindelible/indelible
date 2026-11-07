use super::*;
use crate::epub_processing::EpubMetadata;

#[tokio::test]
async fn non_epub_asset_is_version_1() {
    let processed = process_upload(UploadFileProcessRequest {
        filename: "page.html".into(),
        content_type: "text/html".into(),
        data: Bytes::from_static(b"<html><head><title>Page</title></head><p>Text.</p></html>"),
        title_override: None,
        max_bytes: 1024,
    })
    .await
    .unwrap();

    assert!(!processed.assets.is_empty());
    assert!(
        processed
            .assets
            .iter()
            .all(|asset| asset.format_version == 1)
    );
}

#[test]
fn epub_toc_without_chapters_is_version_1() {
    let mut metadata = EpubMetadata {
        title: None,
        author: None,
        publisher: None,
        language: None,
        isbn: None,
        total_chapters: 0,
        total_words: 0,
        estimated_pages: 0,
        chapters: None,
    };
    assert_eq!(epub::epub_toc_format_version(&metadata), 1);

    metadata.chapters = Some(Vec::new());
    assert_eq!(epub::epub_toc_format_version(&metadata), 2);
}
