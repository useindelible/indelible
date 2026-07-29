use chrono::Utc;
use ind_domain::{ApiPermission, ApiTokenId, ClientType, Theme, User, UserId, UserStatus};

use super::{
    EXTENSION_ACCESS_POLICY, MOBILE_ACCESS_POLICY, USER_ACCESS_POLICY, VERIFIED_USER_ACCESS_POLICY,
    VERIFIED_WEB_ACCESS_POLICY, WEB_ACCESS_POLICY, authorize_jwt_access,
};
use crate::error::ApiError;
use crate::middleware::{ApiCredential, Principal};

fn principal(credential: ApiCredential, email_verified: bool) -> Principal {
    let user_id = UserId::new();
    let now = Utc::now();
    Principal {
        user: User {
            id: user_id,
            email: "reader@example.com".to_string(),
            password_hash: None,
            display_name: "Reader".to_string(),
            avatar_url: None,
            locale: "en".to_string(),
            timezone: "UTC".to_string(),
            theme: Theme::System,
            email_verified,
            onboarding_completed: true,
            onboarding_step: 0,
            email_token: "email-token".to_string(),
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
        },
        user_id,
        credential,
    }
}

fn jwt(client_type: ClientType, email_verified: bool) -> Principal {
    principal(ApiCredential::UserAccessJwt { client_type }, email_verified)
}

fn assert_forbidden(error: ApiError, expected: &str) {
    let ApiError::Forbidden { message } = error else {
        panic!("expected forbidden, got {error:?}");
    };
    assert_eq!(message, expected);
}

#[test]
fn user_access_jwt_policy_accepts_all_ordinary_clients_and_rejects_other_credentials() {
    for client_type in [
        ClientType::Web,
        ClientType::Ios,
        ClientType::Android,
        ClientType::Desktop,
        ClientType::Cli,
    ] {
        authorize_jwt_access(&jwt(client_type, false), USER_ACCESS_POLICY)
            .unwrap_or_else(|error| panic!("{client_type:?} JWT was denied: {error:?}"));
    }

    let extension_error =
        authorize_jwt_access(&jwt(ClientType::Extension, true), USER_ACCESS_POLICY)
            .expect_err("Extension must remain isolated from ordinary user access");
    assert_forbidden(extension_error, "account session required");

    let pat = principal(
        ApiCredential::PersonalAccessToken {
            token_id: ApiTokenId::new(),
            permissions: vec![ApiPermission::LibraryRead],
        },
        true,
    );
    let pat_error = authorize_jwt_access(&pat, USER_ACCESS_POLICY)
        .expect_err("PAT must not satisfy a JWT-only policy");
    assert_forbidden(pat_error, "account session required");
}

#[test]
fn verified_user_access_jwt_policy_preserves_email_verification() {
    let error = authorize_jwt_access(
        &jwt(ClientType::Desktop, false),
        VERIFIED_USER_ACCESS_POLICY,
    )
    .expect_err("unverified ordinary JWT must be denied");
    assert_forbidden(error, "email verification required");

    authorize_jwt_access(&jwt(ClientType::Cli, true), VERIFIED_USER_ACCESS_POLICY)
        .expect("verified CLI JWT must be accepted");
}

#[test]
fn verified_web_access_jwt_policy_requires_both_web_and_verified_email() {
    let mobile_error =
        authorize_jwt_access(&jwt(ClientType::Ios, true), VERIFIED_WEB_ACCESS_POLICY)
            .expect_err("non-Web JWT must be denied");
    assert_forbidden(mobile_error, "web access required");

    let unverified_error =
        authorize_jwt_access(&jwt(ClientType::Web, false), VERIFIED_WEB_ACCESS_POLICY)
            .expect_err("unverified Web JWT must be denied");
    assert_forbidden(unverified_error, "email verification required");

    authorize_jwt_access(&jwt(ClientType::Web, true), VERIFIED_WEB_ACCESS_POLICY)
        .expect("verified Web JWT must be accepted");
}

#[test]
fn web_access_compatibility_policy_accepts_only_web_without_requiring_verified_email() {
    authorize_jwt_access(&jwt(ClientType::Web, false), WEB_ACCESS_POLICY)
        .expect("the extension exchange must accept an unverified Web JWT");

    for client_type in [ClientType::Ios, ClientType::Extension] {
        let error = authorize_jwt_access(&jwt(client_type, true), WEB_ACCESS_POLICY)
            .expect_err("non-Web JWT must be denied");
        assert_forbidden(error, "web access required");
    }

    let pat = principal(
        ApiCredential::PersonalAccessToken {
            token_id: ApiTokenId::new(),
            permissions: vec![ApiPermission::LibraryRead],
        },
        true,
    );
    let error = authorize_jwt_access(&pat, WEB_ACCESS_POLICY)
        .expect_err("PAT must not satisfy the Web-only compatibility policy");
    assert_forbidden(error, "web access required");
}

#[test]
fn extension_access_jwt_policy_accepts_only_extension_without_changing_verification_semantics() {
    authorize_jwt_access(&jwt(ClientType::Extension, false), EXTENSION_ACCESS_POLICY)
        .expect("Extension access has never required verified email");

    let error = authorize_jwt_access(&jwt(ClientType::Web, true), EXTENSION_ACCESS_POLICY)
        .expect_err("Web JWT must not satisfy Extension-only access");
    assert_forbidden(error, "extension access required");
}

#[test]
fn mobile_access_jwt_policy_accepts_only_ios_and_android() {
    for client_type in [ClientType::Ios, ClientType::Android] {
        authorize_jwt_access(&jwt(client_type, false), MOBILE_ACCESS_POLICY)
            .unwrap_or_else(|error| panic!("{client_type:?} JWT was denied: {error:?}"));
    }

    for client_type in [ClientType::Web, ClientType::Desktop, ClientType::Cli] {
        let error = authorize_jwt_access(&jwt(client_type, true), MOBILE_ACCESS_POLICY)
            .expect_err("non-mobile JWT must be denied");
        assert_forbidden(error, "mobile access required");
    }
}
