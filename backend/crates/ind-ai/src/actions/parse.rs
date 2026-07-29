use std::collections::{HashMap, HashSet};

use ind_application::AppError;
use ind_domain::{EntityType, ExtractedEntity};

use super::normalize::{
    normalize_entity_name, normalize_optional_text, normalize_tag, parse_entity_type,
};

pub(super) fn parse_summary_output(raw: &str) -> Result<String, AppError> {
    let stripped_owned = strip_markdown_fences(raw);
    let stripped = stripped_owned.trim();

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(stripped) {
        if let Some(summary) = value.as_str() {
            return non_empty_summary(summary);
        }
        if let Some(summary) = value.get("summary").and_then(|value| value.as_str()) {
            return non_empty_summary(summary);
        }
    }

    non_empty_summary(stripped)
}

fn non_empty_summary(value: &str) -> Result<String, AppError> {
    let summary = value.trim();
    if summary.is_empty() {
        Err(AppError::ExternalService {
            service: "mila-provider".into(),
            message: "summary response was empty".into(),
        })
    } else {
        Ok(summary.to_string())
    }
}

pub(super) fn parse_tags_output(raw: &str) -> Result<Vec<String>, AppError> {
    let value = parse_json_value(raw)?;
    let tag_values = match value {
        serde_json::Value::Array(values) => values,
        serde_json::Value::Object(map) => map
            .get("tags")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default(),
        _ => {
            return Err(AppError::ExternalService {
                service: "mila-provider".into(),
                message: "tags response must be a JSON array".into(),
            });
        }
    };

    let mut seen = HashSet::new();
    let mut tags = Vec::new();
    for value in tag_values {
        let Some(tag) = value.as_str() else {
            continue;
        };
        let normalized = normalize_tag(tag);
        if normalized.is_empty() || !seen.insert(normalized.clone()) {
            continue;
        }
        tags.push(normalized);
        if tags.len() >= 8 {
            break;
        }
    }

    Ok(tags)
}

pub(super) fn parse_entities_output(raw: &str) -> Result<Vec<ExtractedEntity>, AppError> {
    let value = parse_json_value(raw)?;
    let entries = match value {
        serde_json::Value::Array(values) => values,
        serde_json::Value::Object(map) => map
            .get("entities")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default(),
        _ => {
            return Err(AppError::ExternalService {
                service: "mila-provider".into(),
                message: "entities response must be a JSON array".into(),
            });
        }
    };

    let mut merged = HashMap::<(String, EntityType), ExtractedEntity>::new();
    for entry in entries {
        let Some(object) = entry.as_object() else {
            continue;
        };

        let Some(name) = object
            .get("name")
            .and_then(|value| value.as_str())
            .map(normalize_entity_name)
        else {
            continue;
        };
        if name.is_empty() {
            continue;
        }

        let entity_type = object
            .get("entity_type")
            .and_then(|value| value.as_str())
            .and_then(parse_entity_type)
            .unwrap_or(EntityType::Work);
        let description = object
            .get("description")
            .and_then(|value| value.as_str())
            .map(normalize_optional_text)
            .filter(|value| !value.is_empty());
        let mention_count = object
            .get("mention_count")
            .and_then(|value| value.as_i64())
            .unwrap_or(1)
            .clamp(1, i64::from(i32::MAX)) as i32;

        let aliases = extract_aliases(object);

        let key = (name.to_ascii_lowercase(), entity_type);
        match merged.get_mut(&key) {
            Some(existing) => {
                existing.mention_count = existing.mention_count.saturating_add(mention_count);
                if existing.description.is_none() {
                    existing.description = description.clone();
                }
                for alias in aliases {
                    if existing.aliases.len() >= 5 {
                        break;
                    }
                    if !existing.aliases.contains(&alias) {
                        existing.aliases.push(alias);
                    }
                }
            }
            None => {
                merged.insert(
                    key,
                    ExtractedEntity {
                        name,
                        entity_type,
                        description,
                        mention_count,
                        aliases,
                    },
                );
            }
        }
    }

    let mut entities = merged.into_values().collect::<Vec<_>>();
    entities.sort_by(|left, right| {
        right
            .mention_count
            .cmp(&left.mention_count)
            .then_with(|| left.name.cmp(&right.name))
    });
    entities.truncate(16);
    Ok(entities)
}

pub(crate) fn extract_aliases(object: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    let Some(values) = object.get("aliases").and_then(|value| value.as_array()) else {
        return Vec::new();
    };
    let mut aliases = Vec::new();
    for value in values {
        let Some(text) = value.as_str() else {
            continue;
        };
        let normalized = normalize_entity_name(text);
        if normalized.is_empty() || aliases.contains(&normalized) {
            continue;
        }
        aliases.push(normalized);
        if aliases.len() >= 5 {
            break;
        }
    }
    aliases
}

pub(crate) fn parse_json_value(raw: &str) -> Result<serde_json::Value, AppError> {
    for candidate in json_candidates(raw) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&candidate) {
            return Ok(value);
        }
    }

    Err(AppError::ExternalService {
        service: "mila-provider".into(),
        message: "model response did not contain valid JSON".into(),
    })
}

fn json_candidates(raw: &str) -> Vec<String> {
    let stripped = strip_markdown_fences(raw).trim().to_string();
    let mut candidates = vec![stripped.clone()];

    if let (Some(start), Some(end)) = (stripped.find('['), stripped.rfind(']'))
        && start < end
    {
        candidates.push(stripped[start..=end].to_string());
    }
    if let (Some(start), Some(end)) = (stripped.find('{'), stripped.rfind('}'))
        && start < end
    {
        candidates.push(stripped[start..=end].to_string());
    }

    candidates
}

fn strip_markdown_fences(raw: &str) -> String {
    let trimmed = raw.trim();
    let Some(rest) = trimmed.strip_prefix("```") else {
        return trimmed.to_string();
    };

    let after_header = rest
        .find('\n')
        .map(|index| &rest[index + 1..])
        .unwrap_or(rest);
    after_header
        .trim_end()
        .strip_suffix("```")
        .unwrap_or(after_header)
        .trim()
        .to_string()
}
