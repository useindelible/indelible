use chrono::Utc;
use ind_domain::{ApiPermission, ApiTokenId, ClientType, Theme, User, UserId, UserStatus};

use super::{
    AccessPolicy, AiReadAndLibraryReadPolicy, AiReadPolicy, AiUseAndLibraryReadPolicy, AiUsePolicy,
    AiWriteAndAiUseAndLibraryReadPolicy, AiWriteAndAiUsePolicy, AiWritePolicy, DocumentAssetPolicy,
    FeedsReadPolicy, FeedsWritePolicy, IntegrationsReadPolicy, IntegrationsWritePolicy,
    LibraryReadPolicy, LibraryWritePolicy, ObsidianSyncPolicy, WebhooksReadPolicy,
    WebhooksWritePolicy, authorize_permission_access,
};
use crate::error::ApiError;
use crate::middleware::{ApiCredential, Principal};

type Authorize = fn(&Principal) -> Result<(), ApiError>;

struct EmptyPolicy;

impl AccessPolicy for EmptyPolicy {
    const REQUIRED: &'static [ApiPermission] = &[];
}

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

fn pat(permissions: Vec<ApiPermission>) -> Principal {
    principal(
        ApiCredential::PersonalAccessToken {
            token_id: ApiTokenId::new(),
            permissions,
        },
        true,
    )
}

fn jwt(client_type: ClientType) -> Principal {
    principal(ApiCredential::UserAccessJwt { client_type }, true)
}

fn assert_insufficient_permissions(error: ApiError, expected: Vec<ApiPermission>) {
    let ApiError::InsufficientPermissions { required } = error else {
        panic!("expected insufficient permissions, got {error:?}");
    };
    assert_eq!(required, expected);
}

fn assert_verification_required(error: ApiError) {
    let ApiError::Forbidden { message } = error else {
        panic!("expected forbidden, got {error:?}");
    };
    assert_eq!(message, "email verification required");
}

#[test]
fn permission_policies_require_exact_membership() {
    let write_only = pat(vec![ApiPermission::LibraryWrite]);
    let error = authorize_permission_access::<LibraryReadPolicy>(&write_only)
        .expect_err("write membership must not imply read membership");
    assert_insufficient_permissions(error, vec![ApiPermission::LibraryRead]);

    let exact = pat(vec![ApiPermission::LibraryRead]);
    authorize_permission_access::<LibraryReadPolicy>(&exact)
        .expect("the exact required permission must authorize access");
}

#[test]
fn empty_permission_policy_denies_personal_access_tokens() {
    let error = authorize_permission_access::<EmptyPolicy>(&pat(vec![ApiPermission::LibraryRead]))
        .expect_err("a PAT must not be authorized by an empty permission policy");
    assert_insufficient_permissions(error, vec![]);
}

#[test]
fn every_named_single_permission_policy_enforces_its_permission() {
    let cases: &[(ApiPermission, Authorize)] = &[
        (ApiPermission::LibraryRead, |principal| {
            authorize_permission_access::<LibraryReadPolicy>(principal)
        }),
        (ApiPermission::LibraryWrite, |principal| {
            authorize_permission_access::<LibraryWritePolicy>(principal)
        }),
        (ApiPermission::FeedsRead, |principal| {
            authorize_permission_access::<FeedsReadPolicy>(principal)
        }),
        (ApiPermission::FeedsWrite, |principal| {
            authorize_permission_access::<FeedsWritePolicy>(principal)
        }),
        (ApiPermission::IntegrationsRead, |principal| {
            authorize_permission_access::<IntegrationsReadPolicy>(principal)
        }),
        (ApiPermission::IntegrationsWrite, |principal| {
            authorize_permission_access::<IntegrationsWritePolicy>(principal)
        }),
        (ApiPermission::WebhooksRead, |principal| {
            authorize_permission_access::<WebhooksReadPolicy>(principal)
        }),
        (ApiPermission::WebhooksWrite, |principal| {
            authorize_permission_access::<WebhooksWritePolicy>(principal)
        }),
        (ApiPermission::AiRead, |principal| {
            authorize_permission_access::<AiReadPolicy>(principal)
        }),
        (ApiPermission::AiWrite, |principal| {
            authorize_permission_access::<AiWritePolicy>(principal)
        }),
        (ApiPermission::AiUse, |principal| {
            authorize_permission_access::<AiUsePolicy>(principal)
        }),
        (ApiPermission::ObsidianSync, |principal| {
            authorize_permission_access::<ObsidianSyncPolicy>(principal)
        }),
    ];

    for (required, authorize) in cases {
        authorize(&pat(vec![*required])).expect("exact permission must authorize access");

        let unrelated = if *required == ApiPermission::LibraryRead {
            ApiPermission::ObsidianSync
        } else {
            ApiPermission::LibraryRead
        };
        let error = authorize(&pat(vec![unrelated]))
            .expect_err("an unrelated permission must not authorize access");
        assert_insufficient_permissions(error, vec![*required]);
    }
}

#[test]
fn composite_policies_report_the_complete_required_list() {
    let cases: &[(&[ApiPermission], Authorize)] = &[
        (
            &[ApiPermission::AiRead, ApiPermission::LibraryRead],
            |principal| authorize_permission_access::<AiReadAndLibraryReadPolicy>(principal),
        ),
        (
            &[ApiPermission::AiUse, ApiPermission::LibraryRead],
            |principal| authorize_permission_access::<AiUseAndLibraryReadPolicy>(principal),
        ),
        (
            &[ApiPermission::AiWrite, ApiPermission::AiUse],
            |principal| authorize_permission_access::<AiWriteAndAiUsePolicy>(principal),
        ),
        (
            &[
                ApiPermission::AiWrite,
                ApiPermission::AiUse,
                ApiPermission::LibraryRead,
            ],
            |principal| {
                authorize_permission_access::<AiWriteAndAiUseAndLibraryReadPolicy>(principal)
            },
        ),
    ];

    for (required, authorize) in cases {
        let incomplete = pat(vec![required[0]]);
        let error =
            authorize(&incomplete).expect_err("a PAT missing one composite member must be denied");
        assert_insufficient_permissions(error, required.to_vec());

        authorize(&pat(required.to_vec()))
            .expect("a PAT containing every composite member must be authorized");
    }
}

#[test]
fn ordinary_user_access_jwts_satisfy_permission_policies() {
    for client_type in [
        ClientType::Web,
        ClientType::Ios,
        ClientType::Android,
        ClientType::Desktop,
        ClientType::Cli,
    ] {
        authorize_permission_access::<WebhooksWritePolicy>(&jwt(client_type))
            .unwrap_or_else(|error| panic!("{client_type:?} JWT was denied: {error:?}"));
    }
}

#[test]
fn extension_jwts_are_isolated_unless_the_policy_explicitly_allows_them() {
    let extension = jwt(ClientType::Extension);
    let error = authorize_permission_access::<LibraryReadPolicy>(&extension)
        .expect_err("ordinary resource policies must isolate Extension JWTs");
    let ApiError::Forbidden { message } = error else {
        panic!("expected forbidden, got {error:?}");
    };
    assert_eq!(
        message,
        "extension access is not permitted for this resource"
    );

    authorize_permission_access::<DocumentAssetPolicy>(&extension)
        .expect("an explicitly extension-enabled policy must accept an Extension JWT");

    let unverified_extension = principal(
        ApiCredential::UserAccessJwt {
            client_type: ClientType::Extension,
        },
        false,
    );
    let error = authorize_permission_access::<LibraryReadPolicy>(&unverified_extension)
        .expect_err("Extension isolation must take precedence over email verification");
    let ApiError::Forbidden { message } = error else {
        panic!("expected forbidden, got {error:?}");
    };
    assert_eq!(
        message,
        "extension access is not permitted for this resource"
    );

    let error = authorize_permission_access::<DocumentAssetPolicy>(&unverified_extension)
        .expect_err("an admitted Extension JWT must still require verified email");
    assert_verification_required(error);
}

#[test]
fn resource_policies_preserve_verified_email_enforcement() {
    let unverified_jwt = principal(
        ApiCredential::UserAccessJwt {
            client_type: ClientType::Web,
        },
        false,
    );
    let error = authorize_permission_access::<LibraryReadPolicy>(&unverified_jwt)
        .expect_err("unverified user JWT must be denied");
    assert_verification_required(error);

    let unverified_pat = principal(
        ApiCredential::PersonalAccessToken {
            token_id: ApiTokenId::new(),
            permissions: vec![ApiPermission::LibraryRead],
        },
        false,
    );
    let error = authorize_permission_access::<LibraryReadPolicy>(&unverified_pat)
        .expect_err("unverified PAT owner must be denied");
    assert_verification_required(error);
}

#[test]
fn under_scoped_pat_reports_the_complete_policy_before_email_verification() {
    let unverified_pat = principal(
        ApiCredential::PersonalAccessToken {
            token_id: ApiTokenId::new(),
            permissions: vec![ApiPermission::AiUse],
        },
        false,
    );
    let error = authorize_permission_access::<AiUseAndLibraryReadPolicy>(&unverified_pat)
        .expect_err("an under-scoped PAT must be denied");
    assert_insufficient_permissions(
        error,
        vec![ApiPermission::AiUse, ApiPermission::LibraryRead],
    );
}
