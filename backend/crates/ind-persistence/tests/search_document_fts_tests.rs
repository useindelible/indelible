#![allow(clippy::unwrap_used)]

use ind_application::repos::content_vector::{ContentVectorRepository, SingleDocumentFtsQuery};
use ind_application::repos::document::DocumentRepository;
use ind_application::repos::email_sender::EmailSenderRepository;
use ind_application::repos::feed::FeedRepository;
use ind_application::repos::search::{SearchFtsQuery, SearchRepository};
use ind_domain::{
    CanonicalAddress, FeedSourceEntry, FeedSourceEntryId, ItemType, SearchDocument,
    SearchDocumentId, SearchDocumentKind, SearchDocumentSource, UserId,
};
use ind_persistence::repos::{
    PgContentVectorRepository, PgDocumentRepository, PgEmailSenderRepository, PgFeedRepository,
    PgSearchRepository,
};
use ind_test_support::{
    DocumentFactory, FeedDeliveryFactory, FeedSourceFactory, FeedSubscriptionFactory,
    LibraryEntryFactory, TestDb, UserFactory,
};

fn query(user_id: UserId, text: &str) -> SearchFtsQuery {
    SearchFtsQuery {
        user_id,
        text_query: Some(text.into()),
        tag_values: vec![],
        negated_tag_values: vec![],
        collection_values: vec![],
        negated_collection_values: vec![],
        type_values: vec![],
        negated_type_values: vec![],
        author_values: vec![],
        negated_author_values: vec![],
        url_values: vec![],
        negated_url_values: vec![],
        entity_values: vec![],
        negated_entity_values: vec![],
        sender_values: vec![],
        negated_sender_values: vec![],
        sender_domain_values: vec![],
        negated_sender_domain_values: vec![],
        list_values: vec![],
        negated_list_values: vec![],
        subject_values: vec![],
        negated_subject_values: vec![],
        before_saved_at: None,
        after_saved_at: None,
        require_read: false,
        exclude_read: false,
        require_unread: false,
        exclude_unread: false,
        require_archived: false,
        exclude_archived: false,
        require_favorited: false,
        exclude_favorited: false,
        require_has_highlights: false,
        exclude_has_highlights: false,
        require_has_notes: false,
        exclude_has_notes: false,
        require_has_unsubscribe: false,
        exclude_has_unsubscribe: false,
        require_pinned: false,
        exclude_pinned: false,
        require_sender_blocked: false,
        exclude_sender_blocked: false,
        require_feed_only: false,
        exclude_feed_only: false,
        score_reference_at: chrono::Utc::now(),
        cursor_score: None,
        cursor_saved_at: None,
        cursor_result_id: None,
        cursor_section_key: None,
        limit: 1,
    }
}

#[tokio::test]
async fn cursor_pagination_does_not_repeat_boundary_when_scores_decay() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let repo = PgSearchRepository::new(pool.clone());
    let user = UserFactory::default().insert(&pool).await;
    let document = DocumentFactory::new(user.id)
        .with_title("Boundary search result")
        .insert(&pool)
        .await;
    LibraryEntryFactory::new(user.id, document.id)
        .insert(&pool)
        .await;
    repo.replace_search_documents_for_document(
        document.id,
        &[SearchDocument {
            id: SearchDocumentId::new(),
            user_id: user.id,
            source: SearchDocumentSource::Document {
                document_id: document.id,
            },
            document_kind: SearchDocumentKind::Item,
            section_key: String::new(),
            section_title: None,
            title: "Boundary search result".into(),
            body_text: "boundarytoken searchable body".into(),
            highlight_text: String::new(),
            metadata_text: String::new(),
            search_config: "simple".into(),
            saved_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }],
    )
    .await
    .unwrap();

    let first_query = query(user.id, "boundarytoken");
    let score_reference_at = first_query.score_reference_at;
    let first = repo.search_fts(&first_query).await.unwrap();
    assert_eq!(first.len(), 1);

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut second_query = query(user.id, "boundarytoken");
    second_query.score_reference_at = score_reference_at;
    second_query.cursor_score = Some(first[0].score);
    second_query.cursor_saved_at = Some(first[0].saved_at);
    second_query.cursor_result_id = Some(document.id.into_uuid());
    second_query.cursor_section_key = Some(String::new());
    assert!(repo.search_fts(&second_query).await.unwrap().is_empty());

    let sender = PgEmailSenderRepository::new(pool.clone())
        .upsert_for_user(
            user.id,
            &CanonicalAddress::new("newsletter@acme.com"),
            None,
            Some("Acme Newsletter"),
        )
        .await
        .unwrap();
    sqlx::query("UPDATE documents SET sender_id = $1 WHERE id = $2")
        .bind(sender.id.into_uuid())
        .bind(document.id.into_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let mut sender_query = query(user.id, "boundarytoken");
    sender_query.sender_values = vec!["  NEWSLETTER@ACME.COM ".into()];
    let matches = repo.search_fts(&sender_query).await.unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].document_id, Some(document.id));
}

#[tokio::test]
async fn adaptive_vectors_support_english_morphology_simple_tokens_mila_and_gin() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let search_repo = PgSearchRepository::new(pool.clone());
    let document_repo = PgDocumentRepository::new(pool.clone());
    let mila_repo = PgContentVectorRepository::new(pool.clone());
    let user = UserFactory::default().insert(&pool).await;

    let english = DocumentFactory::new(user.id)
        .with_title("Running research systems")
        .insert(&pool)
        .await;
    LibraryEntryFactory::new(user.id, english.id)
        .insert(&pool)
        .await;
    search_repo
        .replace_search_documents_for_document(
            english.id,
            &[search_document(
                user.id,
                english.id,
                "Running research systems",
                "The researchers are running controlled experiments across several systems.",
                "english",
            )],
        )
        .await
        .unwrap();

    let english_hits = search_repo
        .search_fts(&query(user.id, "run"))
        .await
        .unwrap();
    assert_eq!(english_hits.len(), 1);
    assert_eq!(english_hits[0].document_id, Some(english.id));
    assert!(
        english_hits[0]
            .snippet
            .to_lowercase()
            .contains("<mark>running</mark>")
    );

    let mila_hits = mila_repo
        .fts_single_document(&SingleDocumentFtsQuery {
            user_id: user.id,
            document_id: english.id,
            text_query: "run".into(),
            limit: 10,
        })
        .await
        .unwrap();
    assert_eq!(mila_hits.len(), 1);
    assert_eq!(mila_hits[0].document_id, Some(english.id));

    let non_english = DocumentFactory::new(user.id)
        .with_title("Crónica española")
        .insert(&pool)
        .await;
    LibraryEntryFactory::new(user.id, non_english.id)
        .insert(&pool)
        .await;
    search_repo
        .replace_search_documents_for_document(
            non_english.id,
            &[search_document(
                user.id,
                non_english.id,
                "Crónica española",
                "Los atletas siguen corriendo durante toda la mañana.",
                "simple",
            )],
        )
        .await
        .unwrap();
    let simple_hits = search_repo
        .search_fts(&query(user.id, "corriendo"))
        .await
        .unwrap();
    assert_eq!(simple_hits.len(), 1);
    assert_eq!(simple_hits[0].document_id, Some(non_english.id));

    assert!(
        document_repo
            .set_language_if_missing(user.id, english.id, "ENG_us")
            .await
            .unwrap()
    );
    assert!(
        !document_repo
            .set_language_if_missing(user.id, english.id, "fr")
            .await
            .unwrap()
    );
    let language: Option<String> =
        sqlx::query_scalar("SELECT language FROM documents WHERE id = $1")
            .bind(english.id.into_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(language.as_deref(), Some("en-us"));

    sqlx::query("UPDATE documents SET language = 'de-DE' WHERE id = $1")
        .bind(english.id.into_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let synced_config: String = sqlx::query_scalar(
        "SELECT search_config::text FROM search_documents WHERE document_id = $1",
    )
    .bind(english.id.into_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(synced_config, "simple");

    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL enable_seqscan = off")
        .execute(&mut *tx)
        .await
        .unwrap();
    let document_plan: Vec<String> = sqlx::query_scalar(
        "EXPLAIN SELECT id FROM search_documents \
         WHERE user_id = $1 AND document_tsv @@ ( \
             websearch_to_tsquery('english'::regconfig, $2) \
             || websearch_to_tsquery('simple'::regconfig, $2) \
         )",
    )
    .bind(user.id.into_uuid())
    .bind("run")
    .fetch_all(&mut *tx)
    .await
    .unwrap();
    assert!(
        document_plan
            .join("\n")
            .contains("idx_search_documents_user_tsv"),
        "{}",
        document_plan.join("\n")
    );

    let feed_plan: Vec<String> = sqlx::query_scalar(
        "EXPLAIN SELECT id FROM feed_source_entries \
         WHERE search_tsv @@ ( \
             websearch_to_tsquery('english'::regconfig, $1) \
             || websearch_to_tsquery('simple'::regconfig, $1) \
         )",
    )
    .bind("run")
    .fetch_all(&mut *tx)
    .await
    .unwrap();
    assert!(
        feed_plan
            .join("\n")
            .contains("idx_feed_source_entries_search_tsv"),
        "{}",
        feed_plan.join("\n")
    );
}

#[tokio::test]
async fn podcast_feed_previews_search_as_articles() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let source = FeedSourceFactory.insert(&pool).await;
    sqlx::query("UPDATE feed_sources SET feed_type = 'podcast' WHERE id = $1")
        .bind(source.id.into_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let subscription = FeedSubscriptionFactory::new(user.id)
        .with_source(source.clone())
        .insert(&pool)
        .await;
    let entry = PgFeedRepository::new(pool.clone())
        .create_source_entry(FeedSourceEntry {
            id: FeedSourceEntryId::new(),
            source_id: source.id,
            guid: "podcast-preview-guid".into(),
            title: "Podcast preview sentinel".into(),
            url: Some("https://example.com/episode".into()),
            canonical_url: Some("https://example.com/episode".into()),
            author: None,
            excerpt: Some("Ordinary show notes".into()),
            content_html: None,
            language: None,
            lead_image_url: None,
            published_at: None,
            discovered_at: chrono::Utc::now(),
        })
        .await
        .unwrap();
    FeedDeliveryFactory::new(user.id, subscription.id, source.id, entry.id)
        .insert(&pool)
        .await;
    let repo = PgSearchRepository::new(pool);

    let hits = repo.search_fts(&query(user.id, "sentinel")).await.unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].content_type, ItemType::Article);

    let mut article_query = query(user.id, "sentinel");
    article_query.type_values = vec!["article".into()];
    assert_eq!(repo.search_fts(&article_query).await.unwrap().len(), 1);

    let mut podcast_query = query(user.id, "sentinel");
    podcast_query.type_values = vec!["podcast".into()];
    assert!(repo.search_fts(&podcast_query).await.unwrap().is_empty());

    let mut negated_article_query = query(user.id, "sentinel");
    negated_article_query.negated_type_values = vec!["article".into()];
    assert!(
        repo.search_fts(&negated_article_query)
            .await
            .unwrap()
            .is_empty()
    );
}

fn search_document(
    user_id: UserId,
    document_id: ind_domain::DocumentId,
    title: &str,
    body_text: &str,
    search_config: &str,
) -> SearchDocument {
    SearchDocument {
        id: SearchDocumentId::new(),
        user_id,
        source: SearchDocumentSource::Document { document_id },
        document_kind: SearchDocumentKind::Item,
        section_key: String::new(),
        section_title: None,
        title: title.into(),
        body_text: body_text.into(),
        highlight_text: String::new(),
        metadata_text: String::new(),
        search_config: search_config.into(),
        saved_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}
