use super::*;

#[tokio::test]
async fn permission_extractor_enforces_exact_pat_membership() {
    let fixture = ExtractorFixture::new().await;
    let exact = fixture
        .pat(fixture.verified.user.id, vec![ApiPermission::LibraryRead])
        .await;
    assert_eq!(
        fixture
            .request("/permission/library-read", &exact)
            .await
            .status(),
        StatusCode::OK
    );

    let unrelated = fixture
        .pat(fixture.verified.user.id, vec![ApiPermission::FeedsRead])
        .await;
    assert_insufficient(
        fixture
            .request("/permission/library-read", &unrelated)
            .await,
        "Bearer error=\"insufficient_scope\", scope=\"library:read\"",
    )
    .await;
}

#[tokio::test]
async fn composite_permission_extractors_require_every_member() {
    let fixture = ExtractorFixture::new().await;
    for (path, complete, incomplete, required_scope) in [
        (
            "/permission/ai-read-library-read",
            vec![ApiPermission::AiRead, ApiPermission::LibraryRead],
            vec![ApiPermission::AiRead],
            "Bearer error=\"insufficient_scope\", scope=\"ai:read library:read\"",
        ),
        (
            "/permission/ai-use-library-read",
            vec![ApiPermission::AiUse, ApiPermission::LibraryRead],
            vec![ApiPermission::AiUse],
            "Bearer error=\"insufficient_scope\", scope=\"ai:use library:read\"",
        ),
    ] {
        let complete = fixture.pat(fixture.verified.user.id, complete).await;
        assert_eq!(
            fixture.request(path, &complete).await.status(),
            StatusCode::OK
        );

        let incomplete = fixture.pat(fixture.verified.user.id, incomplete).await;
        assert_insufficient(fixture.request(path, &incomplete).await, required_scope).await;
    }
}

#[tokio::test]
async fn permission_extractor_isolates_extension_unless_policy_allows_it() {
    let fixture = ExtractorFixture::new().await;
    for client_type in [
        ClientType::Web,
        ClientType::Ios,
        ClientType::Android,
        ClientType::Desktop,
        ClientType::Cli,
    ] {
        let token = fixture.jwt(&fixture.verified.user, client_type);
        assert_eq!(
            fixture
                .request("/permission/library-read", &token)
                .await
                .status(),
            StatusCode::OK,
            "{client_type:?} JWT must satisfy a permission policy"
        );
    }

    let extension = fixture.jwt(&fixture.verified.user, ClientType::Extension);
    assert_problem(
        fixture
            .request("/permission/library-read", &extension)
            .await,
        "forbidden: extension access is not permitted for this resource",
    )
    .await;
    assert_eq!(
        fixture
            .request("/permission/extension-allowed-library-read", &extension)
            .await
            .status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn permission_extractor_orders_credential_rejection_before_verification() {
    let fixture = ExtractorFixture::new().await;
    let extension = fixture.jwt(&fixture.unverified, ClientType::Extension);
    assert_problem(
        fixture
            .request("/permission/library-read", &extension)
            .await,
        "forbidden: extension access is not permitted for this resource",
    )
    .await;
    assert_problem(
        fixture
            .request("/permission/extension-allowed-library-read", &extension)
            .await,
        "forbidden: email verification required",
    )
    .await;

    let under_scoped = fixture
        .pat(fixture.unverified.id, vec![ApiPermission::AiUse])
        .await;
    assert_insufficient(
        fixture
            .request("/permission/ai-use-library-read", &under_scoped)
            .await,
        "Bearer error=\"insufficient_scope\", scope=\"ai:use library:read\"",
    )
    .await;

    let admitted = fixture
        .pat(fixture.unverified.id, vec![ApiPermission::LibraryRead])
        .await;
    assert_problem(
        fixture.request("/permission/library-read", &admitted).await,
        "forbidden: email verification required",
    )
    .await;
}
