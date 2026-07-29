use std::env;
use std::fs;
use std::path::PathBuf;

use ind_http_api::ApiDoc;
use utoipa::OpenApi;

/// Replace inline `data` array items in `PaginatedResponse_*` schemas with `$ref` to the
/// standalone component type. utoipa inlines generic type parameters, producing duplicate
/// inline schemas that code generators (Fabrikt, openapi-ts) emit as separate `*Data` classes.
fn dedup_paginated_data_items(spec: &mut serde_json::Value) {
    let schemas = match spec
        .pointer_mut("/components/schemas")
        .and_then(|v| v.as_object_mut())
    {
        Some(s) => s,
        None => return,
    };

    // Collect the standalone schema names so we know what $refs are available.
    let standalone_names: Vec<String> = schemas.keys().cloned().collect();

    // For each PaginatedResponse_Foo, check if Foo exists as a standalone schema.
    // If so, replace the inline data.items with a $ref.
    let paginated_keys: Vec<String> = schemas
        .keys()
        .filter(|k| k.starts_with("PaginatedResponse_"))
        .cloned()
        .collect();

    for key in paginated_keys {
        let Some(item_type) = key.strip_prefix("PaginatedResponse_") else {
            continue;
        };
        if !standalone_names.contains(&item_type.to_string()) {
            continue;
        }

        if let Some(items) = schemas
            .get_mut(&key)
            .and_then(|s| s.pointer_mut("/properties/data/items"))
        {
            // Only replace if it's currently an inline object (not already a $ref)
            if items.get("$ref").is_some() {
                continue;
            }
            *items = serde_json::json!({
                "$ref": format!("#/components/schemas/{item_type}")
            });
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("expected output path argument")?;

    let openapi = ApiDoc::openapi();
    let mut json = serde_json::to_value(&openapi)?;
    dedup_paginated_data_items(&mut json);
    let pretty = serde_json::to_vec_pretty(&json)?;
    fs::write(output_path, pretty)?;

    Ok(())
}
