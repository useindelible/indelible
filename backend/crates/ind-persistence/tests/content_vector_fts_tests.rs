#![allow(clippy::unwrap_used)]

use chrono::Utc;
use ind_application::repos::collection::CollectionRepository;
use ind_application::repos::content_vector::{
    CollectionDocumentFtsQuery, ContentVectorRepository, CrossDocumentFtsQuery,
    SingleDocumentFtsQuery,
};
use ind_application::repos::search::SearchRepository;
use ind_domain::{
    ContentVector, ContentVectorId, DocumentId, SearchDocument, SearchDocumentId,
    SearchDocumentKind, SearchDocumentSource, SearchSectionKind, UserId,
};
use ind_persistence::repos::{
    PgCollectionRepository, PgContentVectorRepository, PgSearchRepository,
};
use ind_test_support::{
    CollectionFactory, DocumentFactory, LibraryEntryFactory, TestDb, UserFactory,
};

fn embedding() -> Vec<f32> {
    let mut embedding = vec![0.0; 768];
    embedding[0] = 1.0;
    embedding
}

fn vector(
    user_id: UserId,
    document_id: DocumentId,
    chunk_index: i32,
    content: &str,
    search_config: &str,
) -> ContentVector {
    ContentVector {
        id: ContentVectorId::new(),
        document_id,
        user_id,
        embedding_model: "test-model".into(),
        embedding_dim: 768,
        section_kind: SearchSectionKind::Item,
        section_key: String::new(),
        chunk_index,
        content: content.into(),
        token_count: content.split_whitespace().count() as i32,
        search_config: search_config.into(),
        embedding: embedding(),
        created_at: Utc::now(),
    }
}

fn search_document(
    user_id: UserId,
    document_id: DocumentId,
    title: &str,
    body: &str,
    search_config: &str,
) -> SearchDocument {
    SearchDocument {
        id: SearchDocumentId::new(),
        source: SearchDocumentSource::Document { document_id },
        user_id,
        document_kind: SearchDocumentKind::Item,
        section_key: String::new(),
        section_title: None,
        title: title.into(),
        body_text: body.into(),
        highlight_text: String::new(),
        metadata_text: String::new(),
        search_config: search_config.into(),
        saved_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn mila_fts_targets_exact_chunks_and_falls_back_per_unembedded_document() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let vectors = PgContentVectorRepository::new(pool.clone());
    let search = PgSearchRepository::new(pool.clone());
    let collections = PgCollectionRepository::new(pool.clone());

    let english = DocumentFactory::new(user.id)
        .with_title("English field notes")
        .insert(&pool)
        .await;
    let english_entry = LibraryEntryFactory::new(user.id, english.id)
        .insert(&pool)
        .await;
    let target = vector(
        user.id,
        english.id,
        1,
        "Tappertit documents the running systems in the eastern archive.",
        "english",
    );
    vectors
        .replace_for_document(
            english.id,
            &[
                vector(
                    user.id,
                    english.id,
                    0,
                    "Tappertit appears alone in an unrelated introductory fragment.",
                    "english",
                ),
                target.clone(),
            ],
        )
        .await
        .unwrap();
    search
        .replace_search_documents_for_document(
            english.id,
            &[search_document(
                user.id,
                english.id,
                "English field notes",
                "Tappertit documents the running systems in the eastern archive.",
                "english",
            )],
        )
        .await
        .unwrap();

    let german = DocumentFactory::new(user.id)
        .with_title("Deutsche Feldnotizen")
        .insert(&pool)
        .await;
    LibraryEntryFactory::new(user.id, german.id)
        .insert(&pool)
        .await;
    sqlx::query("UPDATE documents SET language = 'de' WHERE id = $1")
        .bind(german.id.into_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let miggs = vector(
        user.id,
        german.id,
        0,
        "Miggs will morgen das seltene Manuskript lesen.",
        "simple",
    );
    vectors
        .replace_for_document(german.id, std::slice::from_ref(&miggs))
        .await
        .unwrap();

    let fallback = DocumentFactory::new(user.id)
        .with_title("Unembedded field report")
        .insert(&pool)
        .await;
    LibraryEntryFactory::new(user.id, fallback.id)
        .insert(&pool)
        .await;
    search
        .replace_search_documents_for_document(
            fallback.id,
            &[search_document(
                user.id,
                fallback.id,
                "Unembedded field report",
                "Fallback evidence remains available before embeddings exist.",
                "english",
            )],
        )
        .await
        .unwrap();

    let single = vectors
        .fts_single_document(&SingleDocumentFtsQuery {
            user_id: user.id,
            document_id: english.id,
            text_query: "What is Tappertit running?".into(),
            limit: 10,
        })
        .await
        .unwrap();
    assert_eq!(single[0].source_chunk_id, Some(target.id));
    assert_eq!(single[0].snippet, target.content);
    assert!(single.iter().all(|hit| hit.source_chunk_id.is_some()));

    let simple = vectors
        .fts_single_document(&SingleDocumentFtsQuery {
            user_id: user.id,
            document_id: german.id,
            text_query: "Miggs will".into(),
            limit: 10,
        })
        .await
        .unwrap();
    assert_eq!(simple.len(), 1);
    assert_eq!(simple[0].source_chunk_id, Some(miggs.id));

    let fallback_hits = vectors
        .fts_single_document(&SingleDocumentFtsQuery {
            user_id: user.id,
            document_id: fallback.id,
            text_query: "fallback evidence".into(),
            limit: 10,
        })
        .await
        .unwrap();
    assert_eq!(fallback_hits.len(), 1);
    assert_eq!(fallback_hits[0].source_chunk_id, None);

    let cross = vectors
        .fts_cross_document(&CrossDocumentFtsQuery {
            user_id: user.id,
            text_query: "Tappertit running".into(),
            limit: 10,
        })
        .await
        .unwrap();
    assert!(
        cross
            .iter()
            .any(|hit| hit.source_chunk_id == Some(target.id))
    );
    assert!(
        cross
            .iter()
            .filter(|hit| hit.document_id == Some(english.id))
            .all(|hit| hit.source_chunk_id.is_some())
    );

    let collection = CollectionFactory::new(user.id).insert(&pool).await;
    collections
        .add_library_entry_to_collection(user.id, collection.id, english_entry.id)
        .await
        .unwrap();
    let collection_hits = vectors
        .fts_collection_document(&CollectionDocumentFtsQuery {
            user_id: user.id,
            collection_id: collection.id,
            text_query: "Tappertit running".into(),
            limit: 10,
            include_descendants: false,
        })
        .await
        .unwrap();
    assert_eq!(collection_hits[0].source_chunk_id, Some(target.id));
}

#[tokio::test]
async fn relaxed_queries_morphology_sync_and_both_language_branches_use_gin() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let document = DocumentFactory::new(user.id)
        .with_title("Morphology")
        .insert(&pool)
        .await;
    LibraryEntryFactory::new(user.id, document.id)
        .insert(&pool)
        .await;
    let repo = PgContentVectorRepository::new(pool.clone());
    repo.replace_for_document(
        document.id,
        &[vector(
            user.id,
            document.id,
            0,
            "The operators are running resilient systems.",
            "english",
        )],
    )
    .await
    .unwrap();

    let run_hits = repo
        .fts_single_document(&SingleDocumentFtsQuery {
            user_id: user.id,
            document_id: document.id,
            text_query: "run".into(),
            limit: 10,
        })
        .await
        .unwrap();
    assert_eq!(run_hits.len(), 1);

    let normalized: String = sqlx::query_scalar(
        "SELECT fts_relaxed_query('english'::regconfig, 'What is Tappertit running?')::text",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(normalized, "'tappertit' | 'run'");

    let single_character: String =
        sqlx::query_scalar("SELECT fts_relaxed_query('english'::regconfig, 'C')::text")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(single_character, "'c'");

    sqlx::query("UPDATE documents SET language = 'de' WHERE id = $1")
        .bind(document.id.into_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let synced: String = sqlx::query_scalar(
        "SELECT search_config::text FROM content_vectors WHERE document_id = $1",
    )
    .bind(document.id.into_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(synced, "simple");

    let feed_trigger: String = sqlx::query_scalar(
        "SELECT pg_get_triggerdef(oid) FROM pg_trigger WHERE tgname = 'trg_feed_source_entries_tsv'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(feed_trigger.contains("UPDATE OF title, author, excerpt, content_html, language"));

    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL enable_seqscan = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    for config in ["english", "simple"] {
        let plan: Vec<String> = sqlx::query_scalar(
            "EXPLAIN SELECT id FROM content_vectors \
             WHERE search_config = $1::regconfig \
               AND content_tsv @@ fts_relaxed_query($1::regconfig, $2)",
        )
        .bind(config)
        .bind("running")
        .fetch_all(&mut *tx)
        .await
        .unwrap();
        assert!(
            plan.join("\n")
                .contains("idx_content_vectors_user_content_tsv"),
            "{config}: {}",
            plan.join("\n")
        );
    }
}
