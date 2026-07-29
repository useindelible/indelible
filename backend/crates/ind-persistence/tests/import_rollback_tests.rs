#![allow(clippy::unwrap_used, clippy::expect_used)]

use ind_application::repos::document::DocumentRepository;
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::highlight::HighlightRepository;
use ind_application::repos::import_job::ImportJobRepository;
use ind_application::repos::library::LibraryRepository;
use ind_domain::{
    DocumentOriginType, HighlightId, HighlightLocator, ImportItemOutcome, ImportJobStatus,
    ImportMethod, ImportSource, NewHighlight, deterministic_origin_id,
};
use ind_persistence::repos::{
    PgDocumentRepository, PgHighlightRepository, PgImportJobRepository, PgLibraryRepository,
};
use ind_test_support::{DocumentFactory, LibraryEntryFactory, TestDb, UserFactory};

#[tokio::test]
async fn import_rollback_removes_membership_but_preserves_authored_data() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let document = DocumentFactory::new(user.id).insert(&pool).await;
    let documents = PgDocumentRepository::new(pool.clone());
    let imports = PgImportJobRepository::new(pool.clone());
    let highlights = PgHighlightRepository::new(pool.clone());
    let library = PgLibraryRepository::new(pool.clone());
    let external_id = "rollback-authored-data";

    documents
        .record_origin(
            user.id,
            document.id,
            DocumentOriginType::ReadwiseImportItem,
            deterministic_origin_id(
                DocumentOriginType::ReadwiseImportItem,
                user.id,
                &format!("readwise:{external_id}"),
            ),
        )
        .await
        .unwrap();
    LibraryEntryFactory::new(user.id, document.id)
        .insert(&pool)
        .await;
    let highlight_id = HighlightId::new();
    highlights
        .create_for_document(
            &NewHighlight {
                id: highlight_id,
                document_id: document.id,
                user_id: user.id,
                color: "yellow".into(),
                text_content: "kept passage".into(),
                locator: Some(HighlightLocator::Html {
                    start_offset: 0,
                    end_offset: 12,
                }),
                source_locator: None,
            },
            MutationSideEffects::none(),
        )
        .await
        .unwrap();
    let job = imports
        .create(
            user.id,
            ImportSource::ReadwiseImport,
            ImportMethod::Csv,
            None,
        )
        .await
        .unwrap();
    imports
        .append_item_outcome(job.id, external_id, ImportItemOutcome::Imported, None, None)
        .await
        .unwrap();

    imports
        .rollback_imported_library_entries(user.id, job.id)
        .await
        .unwrap();

    assert!(
        library
            .find_active_by_document(user.id, document.id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        documents
            .find_by_id(user.id, document.id)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        highlights
            .get_by_id(highlight_id, user.id)
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        imports
            .find_by_id(user.id, job.id)
            .await
            .unwrap()
            .unwrap()
            .status,
        ImportJobStatus::RolledBack
    );
}
