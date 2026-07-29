use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{SmartListId, UserId};

// Keep this serde shape in sync with ind-http-api's schema-only FilterExpressionNode mirror.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FilterNode {
    And {
        conditions: Vec<FilterNode>,
    },
    Or {
        conditions: Vec<FilterNode>,
    },
    Not {
        condition: Box<FilterNode>,
    },
    Condition {
        field: String,
        op: FilterOp,
        value: serde_json::Value,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    Contains,
    In,
}

pub const ALLOWED_FILTER_FIELDS: &[&str] = &[
    "item_type",
    "tag",
    "collection",
    "triage_state",
    "is_favorite",
    "domain",
    "subject",
    "sender",
    "sender_domain",
    "list_id",
    "has_unsubscribe",
    "sender_blocked",
    "saved_at",
    "published_at",
];

impl FilterNode {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::And { conditions } | Self::Or { conditions } => {
                conditions.iter().try_for_each(Self::validate)
            }
            Self::Not { condition } => condition.validate(),
            Self::Condition { field, op, value } => {
                validate_filter_condition_semantics(field, op, value)
            }
        }
    }
}

fn validate_filter_condition_semantics(
    field: &str,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), String> {
    if !ALLOWED_FILTER_FIELDS.contains(&field) {
        return Err(format!("unsupported filter field: {field}"));
    }

    match field {
        "tag" => {
            if !matches!(op, FilterOp::Eq | FilterOp::Neq | FilterOp::Contains) {
                return Err("tag field only supports eq, neq, contains".into());
            }
            ensure_string_value(field, value).map(|_| ())
        }
        "collection" => {
            if !matches!(op, FilterOp::Eq | FilterOp::Contains) {
                return Err("collection field only supports eq, contains".into());
            }
            let raw = ensure_string_value(field, value)?;
            uuid::Uuid::parse_str(raw.strip_prefix("col_").unwrap_or(raw))
                .map_err(|_| "collection value is not a valid UUID".to_string())?;
            Ok(())
        }
        "is_favorite" => {
            if !matches!(op, FilterOp::Eq) {
                return Err(format!("{field} only supports eq"));
            }
            if value.is_boolean() {
                Ok(())
            } else {
                Err(format!("{field} value must be boolean, got: {value}"))
            }
        }
        "subject" | "sender" | "sender_domain" | "list_id" => match op {
            FilterOp::Eq | FilterOp::Neq | FilterOp::Contains => {
                ensure_string_value(field, value).map(|_| ())
            }
            FilterOp::In => ensure_string_array(field, value),
            _ => Err(format!("{field} only supports eq, neq, contains, in")),
        },
        "has_unsubscribe" | "sender_blocked" => {
            if !matches!(op, FilterOp::Eq) {
                return Err(format!("{field} only supports eq"));
            }
            if value.is_boolean() {
                Ok(())
            } else {
                Err(format!("{field} value must be boolean, got: {value}"))
            }
        }
        "item_type" | "triage_state" | "domain" => match op {
            FilterOp::Eq | FilterOp::Neq => ensure_string_value(field, value).map(|_| ()),
            FilterOp::In => ensure_string_array(field, value),
            _ => Err(format!("{field} only supports eq, neq, in")),
        },
        "saved_at" | "published_at" => {
            if !matches!(
                op,
                FilterOp::Eq | FilterOp::Gt | FilterOp::Lt | FilterOp::Gte | FilterOp::Lte
            ) {
                return Err(format!("{field} does not support this operator"));
            }
            let raw = ensure_string_value(field, value)?;
            raw.parse::<DateTime<Utc>>()
                .map(|_| ())
                .map_err(|_| format!("{field} value is not a valid timestamp"))
        }
        _ => Err(format!("unsupported filter field: {field}")),
    }
}

fn ensure_string_value<'a>(field: &str, value: &'a serde_json::Value) -> Result<&'a str, String> {
    value
        .as_str()
        .ok_or_else(|| format!("{field} value must be a string, got: {value}"))
}

fn ensure_string_array(field: &str, value: &serde_json::Value) -> Result<(), String> {
    let entries = value
        .as_array()
        .ok_or_else(|| format!("{field} IN value must be an array"))?;
    if entries.iter().all(|entry| entry.is_string()) {
        Ok(())
    } else {
        Err(format!("{field} IN values must be strings"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartList {
    pub id: SmartListId,
    pub user_id: UserId,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_pinned: bool,
    pub filter_expression: FilterNode,
    pub default_sort: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn condition(field: &str, op: FilterOp, value: serde_json::Value) -> FilterNode {
        FilterNode::Condition {
            field: field.into(),
            op,
            value,
        }
    }

    #[test]
    fn filter_semantics_accept_and_reject_table() {
        for (node, valid) in [
            (
                condition("sender", FilterOp::Contains, serde_json::json!("example")),
                true,
            ),
            (
                condition("sender", FilterOp::Gt, serde_json::json!("example")),
                false,
            ),
            (
                condition("sender_blocked", FilterOp::Eq, serde_json::json!(true)),
                true,
            ),
            (
                condition("sender_blocked", FilterOp::Eq, serde_json::json!("yes")),
                false,
            ),
            (
                condition(
                    "saved_at",
                    FilterOp::Gte,
                    serde_json::json!("2026-01-01T00:00:00Z"),
                ),
                true,
            ),
            (
                condition("unknown", FilterOp::Eq, serde_json::json!("x")),
                false,
            ),
        ] {
            assert_eq!(node.validate().is_ok(), valid, "{node:?}");
        }
    }
}
