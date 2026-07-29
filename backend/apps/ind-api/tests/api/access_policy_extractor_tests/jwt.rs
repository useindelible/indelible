use super::*;

#[tokio::test]
async fn require_user_access_jwt_uses_the_ordinary_client_policy() {
    let fixture = ExtractorFixture::new().await;
    for client_type in [
        ClientType::Web,
        ClientType::Ios,
        ClientType::Android,
        ClientType::Desktop,
        ClientType::Cli,
    ] {
        let token = fixture.jwt(&fixture.unverified, client_type);
        assert_eq!(
            fixture.request("/jwt/user", &token).await.status(),
            StatusCode::OK
        );
    }

    let extension = fixture.jwt(&fixture.unverified, ClientType::Extension);
    assert_problem(
        fixture.request("/jwt/user", &extension).await,
        "forbidden: account session required",
    )
    .await;
    let pat = fixture
        .pat(fixture.unverified.id, vec![ApiPermission::LibraryRead])
        .await;
    assert_problem(
        fixture.request("/jwt/user", &pat).await,
        "forbidden: account session required",
    )
    .await;
}

#[tokio::test]
async fn require_verified_user_access_jwt_checks_eligibility_before_verification() {
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
            fixture.request("/jwt/verified-user", &token).await.status(),
            StatusCode::OK
        );
    }

    let unverified = fixture.jwt(&fixture.unverified, ClientType::Cli);
    assert_problem(
        fixture.request("/jwt/verified-user", &unverified).await,
        "forbidden: email verification required",
    )
    .await;
    let extension = fixture.jwt(&fixture.unverified, ClientType::Extension);
    assert_problem(
        fixture.request("/jwt/verified-user", &extension).await,
        "forbidden: user access JWT required",
    )
    .await;
    let pat = fixture
        .pat(fixture.unverified.id, vec![ApiPermission::LibraryRead])
        .await;
    assert_problem(
        fixture.request("/jwt/verified-user", &pat).await,
        "forbidden: user access JWT required",
    )
    .await;
}

#[tokio::test]
async fn require_verified_web_access_jwt_uses_the_verified_web_policy() {
    let fixture = ExtractorFixture::new().await;
    assert_eq!(
        fixture
            .request("/jwt/verified-web", &fixture.verified.token)
            .await
            .status(),
        StatusCode::OK
    );
    let unverified_web = fixture.jwt(&fixture.unverified, ClientType::Web);
    assert_problem(
        fixture.request("/jwt/verified-web", &unverified_web).await,
        "forbidden: email verification required",
    )
    .await;
    let mobile = fixture.jwt(&fixture.unverified, ClientType::Ios);
    assert_problem(
        fixture.request("/jwt/verified-web", &mobile).await,
        "forbidden: web access required",
    )
    .await;
    let pat = fixture
        .pat(fixture.unverified.id, vec![ApiPermission::LibraryRead])
        .await;
    assert_problem(
        fixture.request("/jwt/verified-web", &pat).await,
        "forbidden: web access required",
    )
    .await;
}

#[tokio::test]
async fn require_extension_access_jwt_uses_the_extension_policy() {
    let fixture = ExtractorFixture::new().await;
    let extension = fixture.jwt(&fixture.unverified, ClientType::Extension);
    assert_eq!(
        fixture.request("/jwt/extension", &extension).await.status(),
        StatusCode::OK
    );
    let web = fixture.jwt(&fixture.unverified, ClientType::Web);
    assert_problem(
        fixture.request("/jwt/extension", &web).await,
        "forbidden: extension access required",
    )
    .await;
    let pat = fixture
        .pat(fixture.unverified.id, vec![ApiPermission::LibraryRead])
        .await;
    assert_problem(
        fixture.request("/jwt/extension", &pat).await,
        "forbidden: extension access required",
    )
    .await;
}

#[tokio::test]
async fn require_mobile_access_jwt_uses_the_mobile_policy() {
    let fixture = ExtractorFixture::new().await;
    for client_type in [ClientType::Ios, ClientType::Android] {
        let token = fixture.jwt(&fixture.unverified, client_type);
        assert_eq!(
            fixture.request("/jwt/mobile", &token).await.status(),
            StatusCode::OK
        );
    }
    for client_type in [
        ClientType::Web,
        ClientType::Desktop,
        ClientType::Extension,
        ClientType::Cli,
    ] {
        let token = fixture.jwt(&fixture.unverified, client_type);
        assert_problem(
            fixture.request("/jwt/mobile", &token).await,
            "forbidden: mobile access required",
        )
        .await;
    }
    let pat = fixture
        .pat(fixture.unverified.id, vec![ApiPermission::LibraryRead])
        .await;
    assert_problem(
        fixture.request("/jwt/mobile", &pat).await,
        "forbidden: mobile access required",
    )
    .await;
}
