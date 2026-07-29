use ind_auth::api_token::{ALL_API_PERMISSIONS, ApiPermission, canonicalize_permissions};

#[test]
fn api_permissions_serialize_and_parse_the_complete_wire_catalogue() {
    let cases = [
        (ApiPermission::LibraryRead, "library:read"),
        (ApiPermission::LibraryWrite, "library:write"),
        (ApiPermission::FeedsRead, "feeds:read"),
        (ApiPermission::FeedsWrite, "feeds:write"),
        (ApiPermission::IntegrationsRead, "integrations:read"),
        (ApiPermission::IntegrationsWrite, "integrations:write"),
        (ApiPermission::WebhooksRead, "webhooks:read"),
        (ApiPermission::WebhooksWrite, "webhooks:write"),
        (ApiPermission::AiRead, "ai:read"),
        (ApiPermission::AiWrite, "ai:write"),
        (ApiPermission::AiUse, "ai:use"),
        (ApiPermission::ObsidianSync, "obsidian:sync"),
    ];

    assert_eq!(ALL_API_PERMISSIONS.len(), cases.len());
    for (permission, wire_value) in cases {
        assert_eq!(permission.as_str(), wire_value);
        assert_eq!(wire_value.parse::<ApiPermission>(), Ok(permission));
        assert_eq!(
            serde_json::to_string(&permission).unwrap(),
            format!("\"{wire_value}\"")
        );
        assert_eq!(
            serde_json::from_str::<ApiPermission>(&format!("\"{wire_value}\"")).unwrap(),
            permission
        );
    }
}

#[test]
fn canonicalization_rejects_an_empty_permission_list() {
    assert!(canonicalize_permissions(&[]).is_err());
}

#[test]
fn permission_parsing_rejects_unknown_values() {
    assert!("admin".parse::<ApiPermission>().is_err());
}

#[test]
fn canonicalization_deduplicates_in_catalogue_order() {
    let canonical = canonicalize_permissions(&[
        ApiPermission::AiUse,
        ApiPermission::LibraryRead,
        ApiPermission::AiUse,
        ApiPermission::FeedsRead,
        ApiPermission::LibraryRead,
    ])
    .unwrap();

    assert_eq!(
        canonical,
        vec![
            ApiPermission::LibraryRead,
            ApiPermission::FeedsRead,
            ApiPermission::AiUse,
        ]
    );
}

#[test]
fn canonicalization_expands_every_write_permission_with_its_read_permission() {
    let cases = [
        (
            ApiPermission::LibraryWrite,
            vec![ApiPermission::LibraryRead, ApiPermission::LibraryWrite],
        ),
        (
            ApiPermission::FeedsWrite,
            vec![ApiPermission::FeedsRead, ApiPermission::FeedsWrite],
        ),
        (
            ApiPermission::IntegrationsWrite,
            vec![
                ApiPermission::IntegrationsRead,
                ApiPermission::IntegrationsWrite,
            ],
        ),
        (
            ApiPermission::WebhooksWrite,
            vec![ApiPermission::WebhooksRead, ApiPermission::WebhooksWrite],
        ),
        (
            ApiPermission::AiWrite,
            vec![ApiPermission::AiRead, ApiPermission::AiWrite],
        ),
    ];

    for (write_permission, expected) in cases {
        assert_eq!(
            canonicalize_permissions(&[write_permission]).unwrap(),
            expected
        );
    }
}
