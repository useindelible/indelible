use ind_application::repos::entity::EntityRepository;
use ind_test_support::{TestDb, UserFactory};

use super::*;

#[tokio::test]
async fn resolution_does_not_leak_entities_across_tenants() {
    let db = TestDb::new().await;
    let repo = PgEntityRepository::new(db.pool().clone());
    let owner = UserFactory::new().insert(db.pool()).await;
    let attacker = UserFactory::new().insert(db.pool()).await;

    let owned = repo
        .insert_canonical(owner.id, "Acme Corp", EntityType::Organization, None)
        .await
        .unwrap();

    repo.insert_alias(
        attacker.id,
        "acme handle",
        EntityType::Organization,
        owned.id,
    )
    .await
    .unwrap();
    assert!(
        repo.find_for_resolution(attacker.id, "acme handle", EntityType::Organization)
            .await
            .unwrap()
            .is_none()
    );

    sqlx::query(
        "INSERT INTO entity_aliases (user_id, entity_type, name, entity_id) \
         VALUES ($1, 'organization', $2, $3)",
    )
    .bind(attacker.id.into_uuid())
    .bind("raw cross")
    .bind(owned.id.into_uuid())
    .execute(db.pool())
    .await
    .unwrap();
    assert!(
        repo.find_for_resolution(attacker.id, "raw cross", EntityType::Organization)
            .await
            .unwrap()
            .is_none()
    );

    let resolved = repo
        .find_for_resolution(owner.id, "Acme Corp", EntityType::Organization)
        .await
        .unwrap();
    assert_eq!(resolved.map(|entity| entity.id), Some(owned.id));
}

#[tokio::test]
async fn block_candidates_surfaces_same_name_entities_of_other_types() {
    let db = TestDb::new().await;
    let repo = PgEntityRepository::new(db.pool().clone());
    let owner = UserFactory::new().insert(db.pool()).await;
    let other = UserFactory::new().insert(db.pool()).await;

    let org = repo
        .insert_canonical(
            owner.id,
            "DeepSeek",
            EntityType::Organization,
            Some("Chinese AI company"),
        )
        .await
        .unwrap();
    repo.insert_canonical(other.id, "DeepSeek", EntityType::Work, None)
        .await
        .unwrap();

    let for_work = repo
        .block_candidates(owner.id, "DeepSeek", EntityType::Work, 5)
        .await
        .unwrap();
    assert_eq!(
        for_work.iter().map(|entity| entity.id).collect::<Vec<_>>(),
        vec![org.id]
    );

    let for_org = repo
        .block_candidates(owner.id, "DeepSeek", EntityType::Organization, 5)
        .await
        .unwrap();
    assert!(
        for_org.is_empty(),
        "the exact (name, type) row is find_for_resolution's hit, never a candidate"
    );
}

#[tokio::test]
async fn block_candidates_ranks_exact_name_before_fuzzy_matches() {
    let db = TestDb::new().await;
    let repo = PgEntityRepository::new(db.pool().clone());
    let owner = UserFactory::new().insert(db.pool()).await;

    repo.insert_canonical(owner.id, "DeepSeek V3", EntityType::Work, None)
        .await
        .unwrap();
    repo.insert_canonical(owner.id, "DeepSeek R1", EntityType::Work, None)
        .await
        .unwrap();
    let exact = repo
        .insert_canonical(owner.id, "DeepSeek", EntityType::Organization, None)
        .await
        .unwrap();

    let candidates = repo
        .block_candidates(owner.id, "DeepSeek", EntityType::Work, 2)
        .await
        .unwrap();
    assert_eq!(candidates.first().map(|entity| entity.id), Some(exact.id));
}
