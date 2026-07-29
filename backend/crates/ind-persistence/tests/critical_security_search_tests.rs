use chrono::Utc;
use ind_application::repos::api_token::ApiTokenRepository;
use ind_application::repos::content_vector::{ContentVectorRepository, CrossDocumentVectorQuery};
use ind_domain::{
    ApiToken, ApiTokenId, ContentVector, ContentVectorId, DocumentId, SearchSectionKind, UserId,
};
use ind_persistence::repos::{PgApiTokenRepository, PgContentVectorRepository};
use ind_test_support::{SavedDocumentFactory, TestDb, UserFactory};

fn embedding(first: f32, second: f32) -> Vec<f32> {
    let mut values = vec![0.0; 768];
    values[0] = first;
    values[1] = second;
    values
}

fn vector(
    user_id: UserId,
    document_id: DocumentId,
    chunk_index: i32,
    content: &str,
) -> ContentVector {
    ContentVector {
        id: ContentVectorId::new(),
        document_id,
        user_id,
        embedding_model: "current-model".into(),
        embedding_dim: 768,
        section_kind: SearchSectionKind::Item,
        section_key: String::new(),
        chunk_index,
        content: content.into(),
        token_count: 4,
        search_config: "english".into(),
        embedding: embedding(1.0, 0.0),
        created_at: Utc::now(),
    }
}

async fn saved_document(db: &TestDb, user_id: UserId, title: &str) -> DocumentId {
    SavedDocumentFactory::new(user_id)
        .with_title(title)
        .insert(db.pool())
        .await
        .document_id
}

#[tokio::test]
async fn vector_replacement_is_atomic() {
    let db = TestDb::new().await;
    let user = UserFactory::new().insert(db.pool()).await;
    let document_id = saved_document(&db, user.id, "Atomic vectors").await;
    let repo = PgContentVectorRepository::new(db.pool().clone());

    repo.replace_for_document(document_id, &[vector(user.id, document_id, 0, "original")])
        .await
        .unwrap();

    let mut invalid = vector(user.id, document_id, 1, "invalid");
    invalid.embedding = vec![0.1];
    assert!(
        repo.replace_for_document(
            document_id,
            &[vector(user.id, document_id, 0, "replacement"), invalid],
        )
        .await
        .is_err()
    );

    let stored: Vec<String> = sqlx::query_scalar(
        "SELECT content FROM content_vectors WHERE document_id = $1 ORDER BY chunk_index",
    )
    .bind(document_id.into_uuid())
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(stored, ["original"]);
}

#[tokio::test]
async fn vector_search_enforces_tenant_visibility_and_embedding_identity() {
    let db = TestDb::new().await;
    let owner = UserFactory::new().insert(db.pool()).await;
    let foreign = UserFactory::new().insert(db.pool()).await;
    let visible = saved_document(&db, owner.id, "Visible").await;
    let deleted = saved_document(&db, owner.id, "Deleted").await;
    let stale = saved_document(&db, owner.id, "Stale").await;
    let foreign_document = saved_document(&db, foreign.id, "Foreign").await;
    let repo = PgContentVectorRepository::new(db.pool().clone());

    for (user_id, document_id, title) in [
        (owner.id, visible, "visible"),
        (owner.id, deleted, "deleted"),
        (owner.id, stale, "stale"),
        (foreign.id, foreign_document, "foreign"),
    ] {
        repo.replace_for_document(document_id, &[vector(user_id, document_id, 0, title)])
            .await
            .unwrap();
    }
    repo.upsert_chunk(&vector(owner.id, visible, 1, "visible second chunk"))
        .await
        .unwrap();
    sqlx::query(
        "UPDATE library_entries SET deleted_at = now() WHERE user_id = $1 AND document_id = $2",
    )
    .bind(owner.id.into_uuid())
    .bind(deleted.into_uuid())
    .execute(db.pool())
    .await
    .unwrap();
    sqlx::query("UPDATE content_vectors SET embedding_model = 'old-model' WHERE document_id = $1")
        .bind(stale.into_uuid())
        .execute(db.pool())
        .await
        .unwrap();

    let hits = repo
        .search_cross_document(&CrossDocumentVectorQuery {
            user_id: owner.id,
            query_embedding: embedding(1.0, 0.0),
            embedding_model: "current-model".into(),
            embedding_dim: 768,
            section_kind: None,
            limit: 10,
        })
        .await
        .unwrap();
    assert_eq!(hits.len(), 2);
    assert!(hits.iter().all(|hit| hit.document_id == Some(visible)));
    assert_eq!(repo.count_documents_by_user(owner.id).await.unwrap(), 3);
    assert_eq!(repo.count_documents_by_user(foreign.id).await.unwrap(), 1);
}

fn api_token(user_id: UserId) -> ApiToken {
    ApiToken {
        id: ApiTokenId::new(),
        user_id,
        name: "automation".into(),
        token_hash: uuid::Uuid::now_v7().to_string(),
        prefix: "ind_test".into(),
        scopes: vec!["read".into()],
        last_used_at: None,
        expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        created_at: Utc::now(),
    }
}

#[tokio::test]
async fn api_token_mutations_are_owner_scoped_and_revocation_is_immediate() {
    let db = TestDb::new().await;
    let owner = UserFactory::new().insert(db.pool()).await;
    let foreign = UserFactory::new().insert(db.pool()).await;
    let repo = PgApiTokenRepository::new(db.pool().clone());
    let token = repo.create(api_token(owner.id)).await.unwrap();

    assert!(
        repo.find_by_id(token.id, foreign.id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(repo.delete(token.id, foreign.id).await.is_err());
    assert!(
        repo.find_by_token_hash(&token.token_hash)
            .await
            .unwrap()
            .is_some()
    );

    repo.delete(token.id, owner.id).await.unwrap();
    assert!(
        repo.find_by_token_hash(&token.token_hash)
            .await
            .unwrap()
            .is_none()
    );
}
