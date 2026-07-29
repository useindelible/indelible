use std::sync::Arc;

use ind_application::repos::oauth_identity::OAuthIdentityRepository;
use ind_domain::{OAuthProvider, UserStatus};
use ind_persistence::repos::{PgOAuthIdentityRepository, PgUserRepository};
use ind_test_support::TestDb;

use super::{OAuthConfig, OAuthError, OAuthService, OAuthUserInfo};

fn user_info(
    provider: OAuthProvider,
    provider_user_id: &str,
    email: Option<&str>,
) -> OAuthUserInfo {
    OAuthUserInfo {
        provider,
        provider_user_id: provider_user_id.into(),
        email: email.map(str::to_string),
        display_name: Some("Boundary User".into()),
        avatar_url: Some("https://example.com/avatar.png".into()),
        access_token: "access-token".into(),
        refresh_token: None,
        email_verified: Some(true),
        allow_auto_create: true,
    }
}

#[tokio::test]
async fn identity_lifecycle_creates_links_reuses_rejects_and_unlinks_real_rows() {
    let db = TestDb::new().await;
    let users = Arc::new(PgUserRepository::new(db.pool().clone()));
    let identities = Arc::new(PgOAuthIdentityRepository::new(db.pool().clone()));
    let service = OAuthService::new(OAuthConfig::default(), users, identities.clone(), true);

    assert!(matches!(
        service.oauth_start(OAuthProvider::Google).await,
        Err(OAuthError::ProviderNotConfigured(OAuthProvider::Google))
    ));
    let google = user_info(
        OAuthProvider::Google,
        "google-boundary",
        Some(" MixedCase@Example.com "),
    );
    let (created, is_new) = service.find_or_create_user(google).await.unwrap();
    assert!(is_new);
    assert_eq!(created.email, "mixedcase@example.com");
    assert_eq!(created.status, UserStatus::Active);

    let (reused, is_new) = service
        .find_or_create_user(user_info(
            OAuthProvider::Google,
            "google-boundary",
            Some("mixedcase@example.com"),
        ))
        .await
        .unwrap();
    assert!(!is_new);
    assert_eq!(reused.id, created.id);

    let (linked, is_new) = service
        .find_or_create_user(user_info(
            OAuthProvider::Apple,
            "apple-boundary",
            Some("MIXEDCASE@example.com"),
        ))
        .await
        .unwrap();
    assert!(!is_new);
    assert_eq!(linked.id, created.id);
    assert_eq!(identities.list_by_user(created.id).await.unwrap().len(), 2);

    service
        .link_oauth(
            created.id,
            user_info(
                OAuthProvider::Oidc,
                "oidc-boundary",
                Some("mixedcase@example.com"),
            ),
        )
        .await
        .unwrap();
    assert!(matches!(
        service
            .link_oauth(
                created.id,
                user_info(
                    OAuthProvider::Oidc,
                    "different-subject",
                    Some("mixedcase@example.com"),
                ),
            )
            .await,
        Err(OAuthError::IdentityAlreadyLinked)
    ));
    let apple = identities
        .find_by_user_and_provider(created.id, OAuthProvider::Apple)
        .await
        .unwrap()
        .unwrap();
    service.unlink_oauth(created.id, apple.id).await.unwrap();
    assert!(identities.find_by_id(apple.id).await.unwrap().is_none());

    let mut unverified = user_info(
        OAuthProvider::Google,
        "unverified",
        Some("unverified@example.com"),
    );
    unverified.email_verified = Some(false);
    assert!(matches!(
        service.find_or_create_user(unverified).await,
        Err(OAuthError::Exchange(message)) if message.contains("unverified")
    ));
    assert!(matches!(
        service
            .find_or_create_user(user_info(OAuthProvider::Oidc, "missing-email", None))
            .await,
        Err(OAuthError::Exchange(message)) if message.contains("email")
    ));
    let mut disabled = user_info(
        OAuthProvider::Oidc,
        "disabled-create",
        Some("disabled@example.com"),
    );
    disabled.allow_auto_create = false;
    assert!(matches!(
        service.find_or_create_user(disabled).await,
        Err(OAuthError::Exchange(message)) if message.contains("disabled")
    ));

    let (sole, _) = service
        .find_or_create_user(user_info(
            OAuthProvider::Google,
            "sole-method",
            Some("sole@example.com"),
        ))
        .await
        .unwrap();
    let sole_identity = identities
        .find_by_user_and_provider(sole.id, OAuthProvider::Google)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        service.unlink_oauth(sole.id, sole_identity.id).await,
        Err(OAuthError::CannotUnlinkOnly)
    ));
}
