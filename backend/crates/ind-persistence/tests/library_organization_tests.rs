use ind_application::repos::collection::CollectionRepository;
use ind_application::repos::email_sender::EmailSenderRepository;
use ind_application::repos::email_unsubscribe_target::{
    EmailUnsubscribeTargetRepository, UnsubscribeTargetUpsert,
};
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::library::LibraryRepository;
use ind_application::repos::lifecycle_outbox::search_reindex_document_outbox;
use ind_application::repos::smart_list::SmartListRepository;
use ind_domain::{CanonicalAddress, DocumentType, FilterNode, FilterOp};
use ind_persistence::repos::{
    PgCollectionRepository, PgEmailSenderRepository, PgEmailUnsubscribeTargetRepository,
    PgLibraryRepository, PgSmartListRepository,
};
use ind_test_support::{
    CollectionFactory, DocumentFactory, LibraryEntryFactory, TestDb, UserFactory,
};

async fn saved_entry(
    pool: &sqlx::PgPool,
    user_id: ind_domain::UserId,
) -> (ind_domain::DocumentId, ind_domain::LibraryEntryId) {
    let document = DocumentFactory::new(user_id)
        .with_document_type(DocumentType::Article)
        .insert(pool)
        .await;
    let entry = LibraryEntryFactory::new(user_id, document.id)
        .insert(pool)
        .await;
    (document.id, entry.id)
}

#[tokio::test]
async fn purge_keeps_document_and_cascades_membership() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let collections = PgCollectionRepository::new(pool.clone());
    let library = PgLibraryRepository::new(pool.clone());

    let (document_id, entry_id) = saved_entry(&pool, user.id).await;
    let collection = CollectionFactory::new(user.id).insert(&pool).await;
    collections
        .add_library_entry_to_collection(user.id, collection.id, entry_id)
        .await
        .unwrap();

    library
        .purge(entry_id, user.id, MutationSideEffects::none())
        .await
        .unwrap();

    let document_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1)")
            .bind(document_id.into_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(document_exists);
    assert_eq!(collections.count_items(collection.id).await.unwrap(), 0);
}

#[tokio::test]
async fn trash_restore_and_purge_are_tenant_scoped_with_atomic_side_effects() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let owner = UserFactory::default().insert(&pool).await;
    let foreign = UserFactory::default().insert(&pool).await;
    let (document_id, entry_id) = saved_entry(&pool, owner.id).await;
    let library = PgLibraryRepository::new(pool.clone());
    let side_effects = || {
        MutationSideEffects::with_outbox(search_reindex_document_outbox(
            document_id,
            chrono::Utc::now(),
        ))
    };

    assert!(
        library
            .soft_delete(entry_id, foreign.id, side_effects())
            .await
            .is_err()
    );
    assert!(
        library
            .find_by_id(entry_id, owner.id)
            .await
            .unwrap()
            .unwrap()
            .entry
            .deleted_at
            .is_none()
    );

    library
        .soft_delete(entry_id, owner.id, side_effects())
        .await
        .unwrap();
    library
        .restore(entry_id, owner.id, side_effects())
        .await
        .unwrap();
    library
        .purge(entry_id, owner.id, side_effects())
        .await
        .unwrap();

    let events: Vec<String> = sqlx::query_scalar(
        "SELECT event_type FROM domain_events \
         WHERE user_id = $1 AND payload->>'library_entry_id' = $2 ORDER BY created_at, id",
    )
    .bind(owner.id.into_uuid())
    .bind(entry_id.to_string())
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        events,
        [
            "library_entry.trashed",
            "library_entry.restored",
            "library_entry.permanently_deleted"
        ]
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM job_outbox WHERE job_type = 'search.reindex_document'",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
}

#[tokio::test]
async fn saved_email_filter_matrix_resolves_sender_state_and_boolean_composition() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let senders = PgEmailSenderRepository::new(pool.clone());
    let primary = senders
        .upsert_for_user(
            user.id,
            &CanonicalAddress::new("newsletter@acme.com"),
            Some("<weekly.acme>"),
            Some("Acme Weekly"),
        )
        .await
        .unwrap();
    let secondary = senders
        .upsert_for_user(
            user.id,
            &CanonicalAddress::new("editor@globex.com"),
            None,
            Some("Globex Editor"),
        )
        .await
        .unwrap();
    senders.block(primary.id).await.unwrap();
    PgEmailUnsubscribeTargetRepository::new(pool.clone())
        .upsert(
            primary.id,
            UnsubscribeTargetUpsert {
                one_click_post_url: Some("https://acme.com/unsubscribe".into()),
                mailto_addr: None,
                web_url: None,
            },
        )
        .await
        .unwrap();
    let primary_document = DocumentFactory::new(user.id)
        .with_document_type(DocumentType::Email)
        .insert(&pool)
        .await;
    sqlx::query("UPDATE documents SET sender_id = $1 WHERE id = $2")
        .bind(primary.id.into_uuid())
        .bind(primary_document.id.into_uuid())
        .execute(&pool)
        .await
        .unwrap();
    LibraryEntryFactory::new(user.id, primary_document.id)
        .insert(&pool)
        .await;
    let secondary_document = DocumentFactory::new(user.id)
        .with_document_type(DocumentType::Email)
        .insert(&pool)
        .await;
    sqlx::query("UPDATE documents SET sender_id = $1 WHERE id = $2")
        .bind(secondary.id.into_uuid())
        .bind(secondary_document.id.into_uuid())
        .execute(&pool)
        .await
        .unwrap();
    LibraryEntryFactory::new(user.id, secondary_document.id)
        .insert(&pool)
        .await;
    let repo = PgSmartListRepository::new(pool);

    let condition = |field: &str, op: FilterOp, value: serde_json::Value| FilterNode::Condition {
        field: field.into(),
        op,
        value,
    };
    for (name, filter, expected) in [
        (
            "exact sender",
            condition(
                "sender",
                FilterOp::Eq,
                serde_json::json!("newsletter@acme.com"),
            ),
            primary_document.id,
        ),
        (
            "display contains",
            condition(
                "sender",
                FilterOp::Contains,
                serde_json::json!("acme weekly"),
            ),
            primary_document.id,
        ),
        (
            "sender list",
            condition(
                "sender",
                FilterOp::In,
                serde_json::json!(["missing@example.com", "editor@globex.com"]),
            ),
            secondary_document.id,
        ),
        (
            "sender domain",
            condition(
                "sender_domain",
                FilterOp::Eq,
                serde_json::json!("globex.com"),
            ),
            secondary_document.id,
        ),
        (
            "list id",
            condition("list_id", FilterOp::Eq, serde_json::json!("<weekly.acme>")),
            primary_document.id,
        ),
        (
            "has unsubscribe",
            condition("has_unsubscribe", FilterOp::Eq, serde_json::json!(true)),
            primary_document.id,
        ),
        (
            "not blocked",
            condition("sender_blocked", FilterOp::Eq, serde_json::json!(false)),
            secondary_document.id,
        ),
    ] {
        let page = repo
            .evaluate_filter(user.id, &filter, None, 10)
            .await
            .unwrap();
        assert_eq!(page.items.len(), 1, "{name}");
        assert_eq!(page.items[0].document.id, expected, "{name}");
    }

    let composed = FilterNode::And {
        conditions: vec![
            condition("sender_blocked", FilterOp::Eq, serde_json::json!(true)),
            FilterNode::Not {
                condition: Box::new(condition(
                    "sender_domain",
                    FilterOp::Eq,
                    serde_json::json!("globex.com"),
                )),
            },
        ],
    };
    let page = repo
        .evaluate_filter(user.id, &composed, None, 10)
        .await
        .unwrap();
    assert_eq!(page.items[0].document.id, primary_document.id);
}
