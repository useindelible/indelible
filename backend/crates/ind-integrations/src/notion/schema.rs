use super::config::NotionPropertyIds;
use super::error::NotionError;

const SOURCE_SELECT_OPTIONS: &[(&str, &str)] = &[
    ("manual", "default"),
    ("extension", "blue"),
    ("share_sheet", "purple"),
    ("feed", "green"),
    ("email", "yellow"),
    ("api", "orange"),
    ("cli", "gray"),
    ("import", "brown"),
];

const CATEGORY_SELECT_OPTIONS: &[(&str, &str)] = &[
    ("article", "green"),
    ("book", "blue"),
    ("email", "yellow"),
    ("pdf", "red"),
    ("tweet", "purple"),
    ("video", "orange"),
    ("podcast", "pink"),
];

const READING_STATUS_SELECT_OPTIONS: &[(&str, &str)] =
    &[("inbox", "yellow"), ("later", "blue"), ("archive", "gray")];

pub(super) const DOCUMENT_TYPE_VIEWS: &[(&str, &str)] = &[
    ("Articles", "article"),
    ("Books", "book"),
    ("Emails", "email"),
    ("PDFs", "pdf"),
    ("Tweets", "tweet"),
    ("Videos", "video"),
    ("Podcasts", "podcast"),
];

fn select_options(options: &[(&str, &str)]) -> Vec<serde_json::Value> {
    options
        .iter()
        .map(|(name, color)| serde_json::json!({ "name": name, "color": color }))
        .collect()
}

pub(super) fn managed_database_create_body() -> serde_json::Value {
    serde_json::json!({
        "parent": {"type": "workspace", "workspace": true},
        "title": [{"type": "text", "text": {"content": "Indelible"}}],
        "initial_data_source": {
            "title": [{"type": "text", "text": {"content": "Indelible"}}],
            "properties": {
                "Title": {"title": {}},
                "Author": {"rich_text": {}},
                "URL": {"url": {}},
                "Canonical URL": {"url": {}},
                "Source": {"select": {"options": select_options(SOURCE_SELECT_OPTIONS)}},
                "Saved At": {"date": {}},
                "Tags": {"multi_select": {}},
                "Category": {"select": {"options": select_options(CATEGORY_SELECT_OPTIONS)}},
                "Reading Status": {"select": {"options": select_options(READING_STATUS_SELECT_OPTIONS)}},
                "Indelible ID": {"rich_text": {}},
                "Last Synced At": {"date": {}}
            }
        }
    })
}

pub(super) fn extract_required_property_ids(
    data_source: &serde_json::Value,
) -> Result<NotionPropertyIds, NotionError> {
    let Some(properties) = data_source["properties"].as_object() else {
        return Err(NotionError::State(
            "Notion schema error: managed data source has no properties".into(),
        ));
    };
    Ok(NotionPropertyIds {
        title: required_property_id(properties, "Title", "title")?,
        author: required_property_id(properties, "Author", "rich_text")?,
        url: required_property_id(properties, "URL", "url")?,
        canonical_url: required_property_id(properties, "Canonical URL", "url")?,
        source: required_property_id(properties, "Source", "select")?,
        saved_at: required_property_id(properties, "Saved At", "date")?,
        tags: required_property_id(properties, "Tags", "multi_select")?,
        category: required_property_id(properties, "Category", "select")?,
        reading_status: required_property_id(properties, "Reading Status", "select")?,
        indelible_id: required_property_id(properties, "Indelible ID", "rich_text")?,
        last_synced_at: required_property_id(properties, "Last Synced At", "date")?,
    })
}

fn required_property_id(
    properties: &serde_json::Map<String, serde_json::Value>,
    name: &str,
    property_type: &str,
) -> Result<String, NotionError> {
    let Some(property) = properties.get(name) else {
        return Err(NotionError::State(format!(
            "Notion schema error: required property '{name}' is missing"
        )));
    };
    if !(property.get(property_type).is_some() || property["type"].as_str() == Some(property_type))
    {
        return Err(NotionError::State(format!(
            "Notion schema error: required property '{name}' must be type '{property_type}'"
        )));
    }
    Ok(property["id"].as_str().unwrap_or(name).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_schema_contains_and_extracts_every_required_property() {
        let body = managed_database_create_body();
        let mut properties = body["initial_data_source"]["properties"].clone();
        for (name, property) in properties.as_object_mut().unwrap() {
            property["id"] = format!("id_{}", name.replace(' ', "_")).into();
        }
        let ids =
            extract_required_property_ids(&serde_json::json!({"properties": properties})).unwrap();
        assert!(ids.is_complete());
        assert_eq!(DOCUMENT_TYPE_VIEWS.len(), 7);
    }

    #[test]
    fn managed_schema_rejects_missing_and_wrong_property_types() {
        assert!(extract_required_property_ids(&serde_json::json!({})).is_err());
        let mut properties =
            managed_database_create_body()["initial_data_source"]["properties"].clone();
        properties["Title"] = serde_json::json!({"rich_text": {}, "id": "wrong"});
        let error = extract_required_property_ids(&serde_json::json!({"properties": properties}))
            .unwrap_err();
        assert!(error.to_string().contains("Title"));
    }
}
