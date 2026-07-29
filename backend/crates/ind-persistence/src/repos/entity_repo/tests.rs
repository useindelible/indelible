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
