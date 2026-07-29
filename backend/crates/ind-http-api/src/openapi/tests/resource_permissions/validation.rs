use serde_json::Value;

const HTTP_METHODS: &[&str] = &[
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

#[rustfmt::skip]
pub(super) const PUBLIC_LIFECYCLE_OPERATIONS: &[(&str, &str)] = &[
    ("post", "/api/v1/auth/register"),
    ("post", "/api/v1/auth/login"),
    ("post", "/api/v1/auth/refresh"),
    ("post", "/api/v1/auth/logout"),
    ("post", "/api/v1/auth/password/forgot"),
    ("post", "/api/v1/auth/password/reset"),
    ("post", "/api/v1/auth/email/verify"),
    ("get", "/api/v1/auth/providers"),
    ("get", "/api/v1/auth/oauth/{provider}/start"),
    ("get", "/api/v1/auth/oauth/{provider}/native/start"),
    ("post", "/api/v1/auth/oauth/native/token"),
    ("get", "/api/v1/auth/extension/start"),
    ("post", "/api/v1/auth/extension/token"),
    ("post", "/api/v1/auth/extension/refresh"),
];

#[rustfmt::skip]
const ADDITIONAL_PUBLIC_LIFECYCLE_OPERATIONS: &[(&str, &str)] = &[
    ("post", "/api/v1/auth/extension/revoke"),
    ("get", "/api/v1/auth/oauth/{provider}/callback"),
    ("post", "/api/v1/auth/oauth/{provider}/callback"),
    ("get", "/api/v1/integrations/{provider}/callback"),
];

fn has_security_scheme(security: Option<&Value>, scheme: &str) -> bool {
    security
        .and_then(Value::as_array)
        .is_some_and(|requirements| {
            requirements
                .iter()
                .any(|requirement| requirement.get(scheme).is_some())
        })
}

fn has_no_effective_security(operation: &Value, spec: &Value) -> bool {
    operation
        .get("security")
        .or_else(|| spec.get("security"))
        .is_none_or(|security| security.as_array().is_some_and(Vec::is_empty))
}

fn is_intentional_public_lifecycle_operation(method: &str, path: &str) -> bool {
    PUBLIC_LIFECYCLE_OPERATIONS
        .iter()
        .chain(ADDITIONAL_PUBLIC_LIFECYCLE_OPERATIONS)
        .any(|(expected_method, expected_path)| {
            method == *expected_method && path == *expected_path
        })
}

pub(super) fn validate_permission_contracts(spec: &Value) -> Result<(), String> {
    let paths = spec["paths"]
        .as_object()
        .ok_or_else(|| "OpenAPI document has no paths object".to_owned())?;
    let mut effective_public_operations = Vec::new();
    let mut documents_known_public_operation = false;

    for (path, path_item) in paths {
        for method in HTTP_METHODS {
            let Some(operation) = path_item.get(*method) else {
                continue;
            };
            documents_known_public_operation |=
                is_intentional_public_lifecycle_operation(method, path);
            let effective_security = operation.get("security").or_else(|| spec.get("security"));
            let has_api_token = has_security_scheme(effective_security, "api_token");
            let metadata = operation.get("x-indelible-permissions");

            if has_no_effective_security(operation, spec) {
                if !is_intentional_public_lifecycle_operation(method, path) {
                    return Err(format!(
                        "{method} {path} is an unlisted public or lifecycle operation"
                    ));
                }
                if metadata.is_some() {
                    return Err(format!(
                        "{method} {path} publishes x-indelible-permissions without api_token security"
                    ));
                }
                effective_public_operations.push((*method, path.as_str()));
                continue;
            }

            if !has_api_token {
                if metadata.is_some() {
                    return Err(format!(
                        "{method} {path} publishes x-indelible-permissions without api_token security"
                    ));
                }
                continue;
            }

            let permissions = metadata
                .and_then(Value::as_array)
                .filter(|items| !items.is_empty())
                .ok_or_else(|| {
                    format!(
                        "{method} {path} with api_token security requires non-empty x-indelible-permissions"
                    )
                })?;
            for value in permissions {
                let permission = value
                    .as_str()
                    .ok_or_else(|| format!("{method} {path} has a non-string API permission"))?;
                if !ind_auth::api_token::ALL_API_PERMISSIONS
                    .iter()
                    .any(|known| known.as_str() == permission)
                {
                    return Err(format!(
                        "{method} {path} declares unknown API permission {permission}"
                    ));
                }
            }
        }
    }

    if documents_known_public_operation {
        for (method, path) in PUBLIC_LIFECYCLE_OPERATIONS
            .iter()
            .chain(ADDITIONAL_PUBLIC_LIFECYCLE_OPERATIONS)
        {
            if !effective_public_operations.contains(&(*method, *path)) {
                return Err(format!(
                    "{method} {path} must remain an intentional public or lifecycle operation"
                ));
            }
        }
    }

    Ok(())
}

#[test]
fn permission_contract_validator_rejects_each_invalid_shape() {
    use serde_json::json;

    let cases = [
        (
            "missing metadata",
            json!({
                "paths": {"/test": {"get": {"security": [{"api_token": []}]}}}
            }),
            "non-empty x-indelible-permissions",
        ),
        (
            "empty metadata",
            json!({
                "paths": {"/test": {"get": {
                    "security": [{"api_token": []}],
                    "x-indelible-permissions": []
                }}}
            }),
            "non-empty x-indelible-permissions",
        ),
        (
            "unknown permission",
            json!({
                "paths": {"/test": {"get": {
                    "security": [{"api_token": []}],
                    "x-indelible-permissions": ["future:permission"]
                }}}
            }),
            "unknown API permission",
        ),
        (
            "non-string permission",
            json!({
                "paths": {"/test": {"get": {
                    "security": [{"api_token": []}],
                    "x-indelible-permissions": [42]
                }}}
            }),
            "non-string API permission",
        ),
        (
            "inherited PAT security",
            json!({
                "security": [{"api_token": []}],
                "paths": {"/test": {"get": {}}}
            }),
            "non-empty x-indelible-permissions",
        ),
        (
            "metadata without PAT security",
            json!({
                "paths": {"/test": {"get": {
                    "security": [{"bearer": []}],
                    "x-indelible-permissions": ["library:read"]
                }}}
            }),
            "without api_token security",
        ),
        (
            "unlisted public operation",
            json!({
                "paths": {"/test": {"get": {}}}
            }),
            "unlisted public or lifecycle operation",
        ),
    ];

    for (name, spec, expected) in cases {
        let error = validate_permission_contracts(&spec).expect_err(name);
        assert!(
            error.contains(expected),
            "{name} returned the wrong error: {error}"
        );
    }
}
