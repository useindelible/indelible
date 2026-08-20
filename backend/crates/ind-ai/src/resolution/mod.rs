mod prompt;
mod resolver;

#[cfg(test)]
mod tests;

use ind_application::AppError;

use crate::ResponseFormat;
use crate::actions::parse_json_value;

pub(crate) use prompt::{
    AdjudicationItem, ENTITY_RESOLUTION_SYSTEM_PROMPT, build_batch_resolution_prompt,
};
pub(crate) use resolver::{EntityRepositoryAdapter, EntityResolutionStore, EntityResolver};

/// One per-entity verdict from a batched adjudication call. `entity_index` is the 1-based position
/// of the entity within the batch; `match_index` is the 1-based candidate number chosen for that
/// entity (`None` = no match).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BatchVerdict {
    pub entity_index: usize,
    pub match_index: Option<usize>,
    pub confidence: f32,
}

pub(crate) fn batch_resolution_response_format() -> ResponseFormat {
    ResponseFormat::json_schema("entity_resolution", batch_resolution_schema())
}

fn batch_resolution_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["results"],
        "properties": {
            "results": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["entity", "match", "confidence"],
                    "properties": {
                        "entity": { "type": "integer" },
                        "match": { "type": ["integer", "null"] },
                        "confidence": { "type": "number" }
                    }
                }
            }
        }
    })
}

/// Parse a batched adjudication response into per-entity verdicts. Malformed individual entries are
/// skipped; an unparseable response is an error so callers can fail open.
pub(crate) fn parse_batch_resolution(raw: &str) -> Result<Vec<BatchVerdict>, AppError> {
    let value = parse_json_value(raw)?;
    let results = value
        .as_object()
        .and_then(|object| object.get("results"))
        .or(Some(&value))
        .and_then(serde_json::Value::as_array);
    let Some(results) = results else {
        return Ok(Vec::new());
    };

    let mut verdicts = Vec::with_capacity(results.len());
    for entry in results {
        let Some(object) = entry.as_object() else {
            continue;
        };
        let Some(entity_index) = object
            .get("entity")
            .and_then(serde_json::Value::as_u64)
            .map(|number| number as usize)
        else {
            continue;
        };
        let match_index = object.get("match").and_then(|value| {
            if value.is_null() {
                None
            } else {
                value
                    .as_u64()
                    .map(|number| number as usize)
                    .or_else(|| value.as_f64().map(|number| number as usize))
            }
        });
        let confidence = object
            .get("confidence")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0) as f32;
        verdicts.push(BatchVerdict {
            entity_index,
            match_index,
            confidence,
        });
    }
    Ok(verdicts)
}
