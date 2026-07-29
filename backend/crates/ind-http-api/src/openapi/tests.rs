use super::*;

fn response_schema<'a>(spec: &'a serde_json::Value, path: &str) -> &'a serde_json::Value {
    &spec["paths"][path]["get"]["responses"]["200"]["content"]["application/json"]["schema"]
}

#[test]
fn generated_client_contracts_remain_typed() {
    let spec = serde_json::to_value(ApiDoc::openapi()).unwrap();
    for path in [
        "/api/v1/library/trash",
        "/api/v1/feeds/subscriptions",
        "/api/v1/feeds/deliveries",
    ] {
        let reference = response_schema(&spec, path)["$ref"].as_str().unwrap();
        assert!(reference.starts_with("#/components/schemas/PaginatedResponse_"));
    }
    let schemas = &spec["components"]["schemas"];
    assert!(
        schemas["FilterExpressionNode"]
            .to_string()
            .contains("conditions")
    );
    let filter_value = &schemas["FilterExpressionValue"];
    assert!(
        ["oneOf", "anyOf"]
            .iter()
            .any(|key| filter_value.get(key).is_some())
    );
    assert_eq!(
        response_schema(&spec, "/api/v1/documents/{document_id}/epub/toc")["$ref"],
        "#/components/schemas/EpubTocResponse"
    );
    for path in ["/api/v1/auth/refresh", "/api/v1/auth/logout"] {
        assert_ne!(spec["paths"][path]["post"]["requestBody"]["required"], true);
    }
}
