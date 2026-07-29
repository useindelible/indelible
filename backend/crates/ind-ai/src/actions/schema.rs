use ind_domain::AiPromptAction;
use serde_json::{Value, json};

use crate::types::ResponseFormat;

/// Strict json_schema response_format for structured Mila actions; chat/custom never use one.
/// No maxItems/minimum constraints: OpenAI documents them as unsupported for some models and
/// OpenAI-compatible local providers may be narrower; parse.rs already truncates and clamps.
pub(super) fn response_format_for(action: AiPromptAction) -> Option<ResponseFormat> {
    match action {
        AiPromptAction::Entities => {
            Some(ResponseFormat::json_schema("entities", entities_schema()))
        }
        AiPromptAction::Tags => Some(ResponseFormat::json_schema("tags", tags_schema())),
        AiPromptAction::Summary => Some(ResponseFormat::json_schema("summary", summary_schema())),
        AiPromptAction::Chat | AiPromptAction::Custom => None,
    }
}

fn entities_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["entities"],
        "properties": {
            "entities": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["name", "entity_type", "description", "mention_count", "aliases"],
                    "properties": {
                        "name": { "type": "string" },
                        "entity_type": {
                            "type": "string",
                            "enum": ["person", "organization", "location", "event", "work"]
                        },
                        "description": { "type": ["string", "null"] },
                        "mention_count": { "type": "integer" },
                        "aliases": { "type": "array", "items": { "type": "string" } }
                    }
                }
            }
        }
    })
}

fn tags_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["tags"],
        "properties": {
            "tags": { "type": "array", "items": { "type": "string" } }
        }
    })
}

fn summary_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["summary"],
        "properties": { "summary": { "type": "string" } }
    })
}
