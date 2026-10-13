use ind_application::repos::document_asset::DocumentAssetRepository;
use ind_application::repos::epub_format::{EpubFormatIssue, EpubFormatRepository};
use ind_domain::{ArchiveAssetKind, DocumentAsset, DocumentId};
use ind_persistence::repos::{PgDocumentAssetRepository, PgEpubFormatRepository};
use sqlx::PgPool;

pub async fn document_asset(
    pool: &PgPool,
    document_id: DocumentId,
    kind: ArchiveAssetKind,
) -> DocumentAsset {
    PgDocumentAssetRepository::new(pool.clone())
        .find_by_document_and_kind(document_id, kind)
        .await
        .unwrap()
        .unwrap_or_else(|| panic!("document {document_id} has no {kind:?} asset"))
}

/// The `(format_version, format_issue)` of the document's EPUB TOC asset.
pub async fn epub_format(pool: &PgPool, document_id: DocumentId) -> (i32, Option<EpubFormatIssue>) {
    let format = PgEpubFormatRepository::new(pool.clone())
        .find_epub_format(document_id)
        .await
        .unwrap()
        .unwrap_or_else(|| panic!("document {document_id} has no EPUB TOC asset"));
    (format.version, format.issue)
}

pub async fn set_epub_format(
    pool: &PgPool,
    document_id: DocumentId,
    version: i32,
    issue: Option<EpubFormatIssue>,
) {
    let toc = document_asset(pool, document_id, ArchiveAssetKind::Epub).await;
    let formats = PgEpubFormatRepository::new(pool.clone());
    formats
        .mark_format(toc.id, version, toc.size_bytes)
        .await
        .unwrap();
    if let Some(issue) = issue {
        formats.mark_format_issue(toc.id, issue).await.unwrap();
    }
}
