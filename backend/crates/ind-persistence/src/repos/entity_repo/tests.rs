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

#[tokio::test]
async fn merge_keeps_source_aliases_and_remembers_the_source_name() {
    let db = TestDb::new().await;
    let repo = PgEntityRepository::new(db.pool().clone());
    let owner = UserFactory::new().insert(db.pool()).await;

    let target = repo
        .insert_canonical(owner.id, "DeepSeek", EntityType::Organization, None)
        .await
        .unwrap();
    let source = repo
        .insert_canonical(owner.id, "DeepSeek", EntityType::Work, None)
        .await
        .unwrap();
    repo.insert_alias(owner.id, "DeepSeek AI", EntityType::Work, source.id)
        .await
        .unwrap();

    repo.merge_entities(owner.id, source.id, target.id)
        .await
        .unwrap();

    let via_alias = repo
        .find_for_resolution(owner.id, "DeepSeek AI", EntityType::Work)
        .await
        .unwrap();
    assert_eq!(via_alias.map(|entity| entity.id), Some(target.id));

    let via_source_name = repo
        .find_for_resolution(owner.id, "DeepSeek", EntityType::Work)
        .await
        .unwrap();
    assert_eq!(via_source_name.map(|entity| entity.id), Some(target.id));
}

#[tokio::test]
async fn merge_resolves_alias_collisions_in_favour_of_the_target() {
    let db = TestDb::new().await;
    let repo = PgEntityRepository::new(db.pool().clone());
    let owner = UserFactory::new().insert(db.pool()).await;

    let target = repo
        .insert_canonical(owner.id, "Meta", EntityType::Organization, None)
        .await
        .unwrap();
    let source = repo
        .insert_canonical(owner.id, "Meta Platforms", EntityType::Organization, None)
        .await
        .unwrap();
    repo.insert_alias(owner.id, "Facebook", EntityType::Organization, target.id)
        .await
        .unwrap();
    repo.insert_alias(owner.id, "FB", EntityType::Organization, source.id)
        .await
        .unwrap();
    // Same (type, name) alias on both sides: the target's row must win and the merge must not fail.
    sqlx::query(
        "INSERT INTO entity_aliases (user_id, entity_type, name, entity_id) \
         VALUES ($1, 'organization', 'Facebook', $2) \
         ON CONFLICT (user_id, entity_type, name) DO NOTHING",
    )
    .bind(owner.id.into_uuid())
    .bind(source.id.into_uuid())
    .execute(db.pool())
    .await
    .unwrap();

    repo.merge_entities(owner.id, source.id, target.id)
        .await
        .unwrap();

    for alias in ["Facebook", "FB", "Meta Platforms"] {
        let resolved = repo
            .find_for_resolution(owner.id, alias, EntityType::Organization)
            .await
            .unwrap();
        assert_eq!(resolved.map(|entity| entity.id), Some(target.id), "{alias}");
    }
}
