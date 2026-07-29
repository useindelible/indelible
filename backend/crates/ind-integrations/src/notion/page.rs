use chrono::{DateTime, Utc};

use super::blocks::truncate_notion_text;
use super::config::NotionPropertyIds;

pub struct NotionPageSpec {
    pub indelible_id: String,
    pub title: String,
    pub url: Option<String>,
    pub canonical_url: Option<String>,
    pub author: Option<String>,
    pub source: String,
    pub saved_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub item_type: String,
    pub triage_state: String,
    pub property_ids: NotionPropertyIds,
}

pub(super) fn build_page_properties(spec: &NotionPageSpec) -> serde_json::Value {
    let title = truncate_notion_text(non_empty(&spec.title).unwrap_or("Untitled"));
    let ids = &spec.property_ids;
    serde_json::json!({
        spec_property_name_or_id(&ids.title, "Title"): {"title": [{"text": {"content": title}}]},
        spec_property_name_or_id(&ids.url, "URL"): {"url": spec.url.as_deref().filter(|s| !s.is_empty())},
        spec_property_name_or_id(&ids.canonical_url, "Canonical URL"): {"url": spec.canonical_url.as_deref().filter(|s| !s.is_empty())},
        spec_property_name_or_id(&ids.author, "Author"): {
            "rich_text": spec.author.as_deref().and_then(non_empty)
                .map(|a| vec![serde_json::json!({"text": {"content": a}})])
                .unwrap_or_default()
        },
        spec_property_name_or_id(&ids.source, "Source"): {"select": {"name": &spec.source}},
        spec_property_name_or_id(&ids.saved_at, "Saved At"): {"date": {"start": spec.saved_at.to_rfc3339()}},
        spec_property_name_or_id(&ids.tags, "Tags"): {
            "multi_select": spec.tags.iter()
                .filter_map(|t| non_empty(t))
                .map(|t| serde_json::json!({"name": t}))
                .collect::<Vec<_>>()
        },
        spec_property_name_or_id(&ids.category, "Category"): {"select": {"name": &spec.item_type}},
        spec_property_name_or_id(&ids.reading_status, "Reading Status"): {"select": {"name": &spec.triage_state}},
        spec_property_name_or_id(&ids.indelible_id, "Indelible ID"): {"rich_text": [{"text": {"content": &spec.indelible_id}}]},
        spec_property_name_or_id(&ids.last_synced_at, "Last Synced At"): {"date": {"start": Utc::now().to_rfc3339()}},
    })
}

pub(super) fn spec_property_name_or_id(id: &str, fallback: &'static str) -> String {
    non_empty(id).unwrap_or(fallback).to_string()
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}
