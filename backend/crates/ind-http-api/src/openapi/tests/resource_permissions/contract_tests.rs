use serde_json::{Value, json};
use utoipa::OpenApi;

use crate::openapi::ApiDoc;

use super::catalogue::*;
use super::validation::{PUBLIC_LIFECYCLE_OPERATIONS, validate_permission_contracts};

fn assert_pat_contracts(spec: &Value, operations: &[OperationContract]) {
    for contract in operations {
        let operation = &spec["paths"][contract.path][contract.method];
        assert_ne!(
            operation,
            &Value::Null,
            "missing OpenAPI operation {} {}",
            contract.method,
            contract.path
        );
        assert_eq!(
            operation["security"],
            json!([{"bearer": []}, {"api_token": []}]),
            "{} {} must advertise bearer OR api_token",
            contract.method,
            contract.path
        );
        assert_eq!(
            operation["x-indelible-permissions"],
            json!([contract.permission]),
            "{} {} must publish its exact permission",
            contract.method,
            contract.path
        );
    }
}

fn assert_composite_pat_contracts(spec: &Value, operations: &[CompositeOperationContract]) {
    for contract in operations {
        let operation = &spec["paths"][contract.path][contract.method];
        assert_ne!(
            operation,
            &Value::Null,
            "missing OpenAPI operation {} {}",
            contract.method,
            contract.path
        );
        assert_eq!(
            operation["security"],
            json!([{"bearer": []}, {"api_token": []}]),
            "{} {} must advertise bearer OR api_token",
            contract.method,
            contract.path
        );
        assert_eq!(
            operation["x-indelible-permissions"],
            json!(contract.permissions),
            "{} {} must publish every exact permission",
            contract.method,
            contract.path
        );
    }
}

#[test]
fn library_operations_publish_explicit_permission_contracts() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");
    assert_pat_contracts(&spec, LIBRARY_OPERATIONS);
}

#[test]
fn feed_operations_publish_explicit_permission_contracts() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");
    assert_pat_contracts(&spec, FEED_OPERATIONS);
}

#[test]
fn integration_operations_publish_explicit_permission_contracts() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");
    assert_pat_contracts(&spec, INTEGRATION_OPERATIONS);
    for path in [
        "/api/v1/auth/oauth/{provider}/callback",
        "/api/v1/integrations/{provider}/callback",
    ] {
        let callback = &spec["paths"][path]["get"];
        assert_eq!(
            callback["security"],
            json!([]),
            "{path} must override document-level auth and stay public"
        );
        assert_eq!(callback["x-indelible-permissions"], Value::Null);
    }
}

#[test]
fn oauth_form_post_callback_is_public_and_documents_its_form_body() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");
    let callback = &spec["paths"]["/api/v1/auth/oauth/{provider}/callback"]["post"];

    assert_ne!(callback, &Value::Null, "missing OAuth form_post callback");
    assert_eq!(callback["security"], json!([]));
    assert_eq!(callback["x-indelible-permissions"], Value::Null);
    assert_eq!(
        callback["requestBody"]["content"]["application/x-www-form-urlencoded"]["schema"]["$ref"],
        "#/components/schemas/OAuthCallbackForm"
    );
}

#[test]
fn webhook_operations_publish_explicit_permission_contracts() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");
    assert_pat_contracts(&spec, WEBHOOK_OPERATIONS);
}

#[test]
fn ai_and_obsidian_operations_publish_explicit_permission_contracts() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");
    assert_composite_pat_contracts(&spec, AI_OPERATIONS);
    assert_composite_pat_contracts(&spec, OBSIDIAN_SYNC_OPERATIONS);
}

#[test]
fn asset_operations_publish_bearer_pat_and_cookie_boundaries_truthfully() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");
    assert_eq!(
        spec["components"]["securitySchemes"]["session_cookie"],
        Value::Null,
        "the unused refresh-cookie security scheme must not be exported"
    );
    assert_ne!(
        spec["components"]["securitySchemes"]["asset_cookie"],
        Value::Null,
        "the signed asset cookie must have an OpenAPI security scheme"
    );

    let metadata = &spec["paths"]["/api/v1/documents/{document_id}/assets/{asset_kind}"]["get"];
    assert_eq!(
        metadata["security"],
        json!([{"bearer": []}, {"api_token": []}])
    );
    assert_eq!(metadata["x-indelible-permissions"], json!(["library:read"]));

    let document = &spec["paths"]["/api/v1/assets/documents/{document_id}/{asset_kind}"]["get"];
    assert_eq!(
        document["security"],
        json!([{"bearer": []}, {"api_token": []}, {"asset_cookie": []}])
    );
    assert_eq!(document["x-indelible-permissions"], json!(["library:read"]));

    let tts = &spec["paths"]["/api/v1/assets/documents/{document_id}/tts/{session_id}/{chunk_file}"]
        ["get"];
    assert_eq!(
        tts["security"],
        json!([{"bearer": []}, {"api_token": []}, {"asset_cookie": []}])
    );
    assert_eq!(
        tts["x-indelible-permissions"],
        json!(["ai:read", "library:read"])
    );

    let avatar = &spec["paths"]["/api/v1/assets/{user_id}/avatars/{filename}"]["get"];
    assert_eq!(
        avatar["security"],
        json!([{"bearer": []}, {"asset_cookie": []}])
    );
    assert_eq!(avatar["x-indelible-permissions"], Value::Null);
}

#[test]
fn account_onboarding_home_settings_and_events_operations_are_jwt_only() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");
    for (method, path) in JWT_ONLY_OPERATIONS {
        let operation = &spec["paths"][path][method];
        assert_eq!(
            operation["security"],
            json!([{"bearer": []}]),
            "{method} {path} must advertise only bearer JWT auth"
        );
        assert_eq!(
            operation["x-indelible-permissions"],
            Value::Null,
            "{method} {path} must not publish PAT permissions"
        );
    }
}

#[test]
fn extension_content_operations_publish_extension_jwt_security() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");
    for (method, path) in EXTENSION_JWT_OPERATIONS {
        let operation = &spec["paths"][path][method];
        assert_eq!(
            operation["security"],
            json!([{"bearer": []}]),
            "{method} {path} must advertise only bearer JWT auth"
        );
        assert_eq!(
            operation["x-indelible-permissions"],
            Value::Null,
            "{method} {path} must not publish PAT permissions"
        );
    }
}

#[test]
fn extension_refresh_revocation_has_no_api_principal_security() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");
    let operation = &spec["paths"]["/api/v1/auth/extension/revoke"]["post"];
    assert_eq!(
        operation["security"],
        json!([]),
        "extension refresh-token revocation must override API credential security"
    );
    assert_eq!(operation["x-indelible-permissions"], Value::Null);
}

#[test]
fn every_api_token_operation_has_global_permission_metadata() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");
    validate_permission_contracts(&spec).expect("exported OpenAPI permission contract");
}

#[test]
fn public_lifecycle_operations_do_not_inherit_api_security() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");
    assert!(
        spec.get("security").is_none(),
        "the OpenAPI document must not apply a global API credential default"
    );

    for (method, path) in PUBLIC_LIFECYCLE_OPERATIONS {
        let operation = &spec["paths"][path][method];
        let local_security = operation.get("security");
        assert!(
            local_security.is_none_or(|security| security == &json!([])),
            "{method} {path} must remain public or lifecycle-credential-only"
        );
        assert_eq!(operation["x-indelible-permissions"], Value::Null);
    }
}

#[test]
fn forbidden_responses_describe_permissions_or_verified_jwt_access() {
    let spec = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI");

    for (method, path) in [
        ("post", "/api/v1/me/email"),
        ("get", "/api/v1/onboarding"),
        ("post", "/api/v1/onboarding/steps/{step}/complete"),
        ("post", "/api/v1/onboarding/skip"),
    ] {
        assert_eq!(
            spec["paths"][path][method]["responses"]["403"]["description"],
            "Verified user access JWT from a supported client and verified email required",
            "{method} {path} must describe its JWT client and email gate"
        );
    }

    for (method, path) in [
        ("get", "/api/v1/documents/{document_id}/epub/toc"),
        (
            "get",
            "/api/v1/documents/{document_id}/epub/chapters/{chapter_index}",
        ),
        ("post", "/api/v1/library/query"),
        ("post", "/api/v1/integrations/{id}/obsidian/preview"),
        (
            "get",
            "/api/v1/assets/documents/{document_id}/tts/{session_id}/{chunk_file}",
        ),
        ("get", "/api/v1/documents/{document_id}/toc"),
    ] {
        assert_eq!(
            spec["paths"][path][method]["responses"]["403"]["description"],
            "Insufficient permissions",
            "{method} {path} must describe its permission gate"
        );
    }
}
