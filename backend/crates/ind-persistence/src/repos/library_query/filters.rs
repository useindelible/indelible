//! `FilterNode` to SQL for saved Library content.

use sqlx::{Postgres, QueryBuilder};

use ind_application::AppError;
use ind_domain::{ALLOWED_FILTER_FIELDS, FilterNode, FilterOp};

use crate::repos::filter_sql::{
    parse_prefixed_uuid, push_boolean_filter, push_text_filter, push_timestamp_filter,
    validation_error,
};

pub(super) fn push_filter_node(
    builder: &mut QueryBuilder<'_, Postgres>,
    node: &FilterNode,
) -> Result<(), AppError> {
    match node {
        FilterNode::And { conditions } => {
            if conditions.is_empty() {
                builder.push("TRUE");
                return Ok(());
            }
            builder.push("(");
            for (index, condition) in conditions.iter().enumerate() {
                if index > 0 {
                    builder.push(" AND ");
                }
                push_filter_node(builder, condition)?;
            }
            builder.push(")");
        }
        FilterNode::Or { conditions } => {
            if conditions.is_empty() {
                builder.push("FALSE");
                return Ok(());
            }
            builder.push("(");
            for (index, condition) in conditions.iter().enumerate() {
                if index > 0 {
                    builder.push(" OR ");
                }
                push_filter_node(builder, condition)?;
            }
            builder.push(")");
        }
        FilterNode::Not { condition } => {
            builder.push("NOT (");
            push_filter_node(builder, condition)?;
            builder.push(")");
        }
        FilterNode::Condition { field, op, value } => {
            push_filter_condition(builder, field, op, value)?;
        }
    }

    Ok(())
}

fn push_filter_condition(
    builder: &mut QueryBuilder<'_, Postgres>,
    field: &str,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    if !ALLOWED_FILTER_FIELDS.contains(&field) {
        return Err(validation_error(&format!(
            "unsupported filter field: {field}"
        )));
    }

    match field {
        "tag" => push_tag_filter(builder, op, value),
        "collection" => push_collection_filter(builder, op, value),
        "is_favorite" => push_boolean_filter(builder, "le.is_favorite", field, op, value),
        "item_type" => push_text_filter(builder, "d.document_type", field, op, value, false),
        "triage_state" => push_text_filter(builder, "le.triage_state", field, op, value, false),
        // domain accepts eq/neq/in only (matches the validator and FE field defs); contains is
        // rejected here so the evaluator and validator agree on the operator matrix.
        "domain" => {
            reject_contains(field, op)?;
            push_text_filter(builder, "d.domain", field, op, value, true)
        }
        "subject" => push_text_filter(builder, "d.title", field, op, value, true),
        "sender" => push_sender_filter(builder, op, value),
        "sender_domain" => push_sender_addr_part_filter(builder, op, value),
        "list_id" => push_sender_text_filter(builder, "s_filter.list_id", field, op, value),
        "has_unsubscribe" => push_has_unsubscribe_filter(builder, op, value),
        "sender_blocked" => push_sender_blocked_filter(builder, op, value),
        "saved_at" => push_timestamp_filter(builder, "le.saved_at", field, op, value),
        "published_at" => push_timestamp_filter(builder, "d.published_at", field, op, value),
        _ => Err(validation_error(&format!("unsupported field: {field}"))),
    }
}

fn reject_contains(field: &str, op: &FilterOp) -> Result<(), AppError> {
    if matches!(op, FilterOp::Contains) {
        return Err(validation_error(&format!(
            "{field} only supports eq, neq, in"
        )));
    }
    Ok(())
}

fn push_tag_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    let tag_name = value
        .as_str()
        .ok_or_else(|| validation_error("tag value must be a string"))?
        .to_string();

    match op {
        FilterOp::Eq | FilterOp::Contains => {
            builder.push(
                "EXISTS (SELECT 1 FROM library_entry_tags let_filter \
                 JOIN tags t_filter ON t_filter.id = let_filter.tag_id \
                 WHERE let_filter.library_entry_id = le.id \
                 AND t_filter.user_id = le.user_id \
                 AND LOWER(t_filter.name) = LOWER(",
            );
            builder.push_bind(tag_name);
            builder.push("))");
            Ok(())
        }
        FilterOp::Neq => {
            builder.push(
                "NOT EXISTS (SELECT 1 FROM library_entry_tags let_filter \
                 JOIN tags t_filter ON t_filter.id = let_filter.tag_id \
                 WHERE let_filter.library_entry_id = le.id \
                 AND t_filter.user_id = le.user_id \
                 AND LOWER(t_filter.name) = LOWER(",
            );
            builder.push_bind(tag_name);
            builder.push("))");
            Ok(())
        }
        _ => Err(validation_error(
            "tag field only supports eq, neq, contains",
        )),
    }
}

fn push_collection_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    let raw_collection_id = value
        .as_str()
        .ok_or_else(|| validation_error("collection value must be a string (collection ID)"))?;
    let collection_id = parse_prefixed_uuid(raw_collection_id, "col_")
        .map_err(|_| validation_error("collection value is not a valid UUID"))?;

    match op {
        FilterOp::Eq | FilterOp::Contains => {
            builder.push(
                "EXISTS (SELECT 1 FROM collection_entries ce_filter \
                 WHERE ce_filter.library_entry_id = le.id \
                 AND ce_filter.user_id = le.user_id \
                 AND ce_filter.collection_id = ",
            );
            builder.push_bind(collection_id);
            builder.push(")");
            Ok(())
        }
        _ => Err(validation_error(
            "collection field only supports eq, contains",
        )),
    }
}

/// Open an `EXISTS` over the email sender linked to the document. The caller appends the inner
/// predicate (referencing `s_filter`) and then calls [`close_sender_exists`].
fn open_sender_exists(builder: &mut QueryBuilder<'_, Postgres>) {
    builder.push(
        "EXISTS (SELECT 1 FROM email_senders s_filter \
         WHERE s_filter.id = d.sender_id \
         AND s_filter.user_id = le.user_id AND ",
    );
}

fn close_sender_exists(builder: &mut QueryBuilder<'_, Postgres>) {
    builder.push(")");
}

/// `sender`: eq/neq/in match the exact canonical address; contains matches either the canonical
/// address or the sender display name. All comparisons are case-insensitive.
fn push_sender_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    if matches!(op, FilterOp::Contains) {
        let needle = ensure_text(value, "sender")?.to_lowercase();
        open_sender_exists(builder);
        builder.push("(LOWER(s_filter.canonical_addr) LIKE '%' || ");
        builder.push_bind(needle.clone());
        builder.push(" || '%' OR LOWER(COALESCE(s_filter.display_name, '')) LIKE '%' || ");
        builder.push_bind(needle);
        builder.push(" || '%')");
        close_sender_exists(builder);
        return Ok(());
    }
    push_sender_text_filter(builder, "s_filter.canonical_addr", "sender", op, value)
}

/// `sender_domain`: the domain part of the sender canonical address.
fn push_sender_addr_part_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    push_sender_text_filter(
        builder,
        "split_part(s_filter.canonical_addr, '@', 2)",
        "sender_domain",
        op,
        value,
    )
}

/// Case-insensitive text predicate on a sender column, wrapped in the sender `EXISTS`.
/// Supports eq/neq/contains/in matching the validator and FE field defs.
fn push_sender_text_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    column: &str,
    field: &str,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    open_sender_exists(builder);
    match op {
        FilterOp::Eq => {
            builder.push(format!("LOWER({column}) = "));
            builder.push_bind(ensure_text(value, field)?.to_lowercase());
        }
        FilterOp::Neq => {
            builder.push(format!("LOWER({column}) != "));
            builder.push_bind(ensure_text(value, field)?.to_lowercase());
        }
        FilterOp::Contains => {
            builder.push(format!("LOWER({column}) LIKE '%' || "));
            builder.push_bind(ensure_text(value, field)?.to_lowercase());
            builder.push(" || '%'");
        }
        FilterOp::In => {
            let values = value
                .as_array()
                .ok_or_else(|| validation_error(&format!("{field} IN value must be an array")))?
                .iter()
                .map(|entry| {
                    entry
                        .as_str()
                        .map(|text| text.to_lowercase())
                        .ok_or_else(|| {
                            validation_error(&format!("{field} IN values must be strings"))
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            builder.push(format!("LOWER({column}) = ANY("));
            builder.push_bind(values);
            builder.push(")");
        }
        _ => {
            return Err(validation_error(&format!(
                "{field} only supports eq, neq, contains, in"
            )));
        }
    }
    close_sender_exists(builder);
    Ok(())
}

/// `has_unsubscribe` (boolean): whether the linked sender has any unsubscribe target.
fn push_has_unsubscribe_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    let want = ensure_bool(value, "has_unsubscribe", op)?;
    if !want {
        builder.push("NOT ");
    }
    open_sender_exists(builder);
    builder.push(
        "EXISTS (SELECT 1 FROM email_unsubscribe_targets t_filter \
         WHERE t_filter.sender_id = s_filter.id)",
    );
    close_sender_exists(builder);
    Ok(())
}

/// `sender_blocked` (boolean): whether the linked sender is blocked.
fn push_sender_blocked_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    let want = ensure_bool(value, "sender_blocked", op)?;
    if !want {
        builder.push("NOT ");
    }
    open_sender_exists(builder);
    builder.push("s_filter.blocked_at IS NOT NULL");
    close_sender_exists(builder);
    Ok(())
}

fn ensure_text<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str, AppError> {
    value
        .as_str()
        .ok_or_else(|| validation_error(&format!("{field} value must be a string")))
}

fn ensure_bool(value: &serde_json::Value, field: &str, op: &FilterOp) -> Result<bool, AppError> {
    if !matches!(op, FilterOp::Eq) {
        return Err(validation_error(&format!("{field} only supports eq")));
    }
    value
        .as_bool()
        .ok_or_else(|| validation_error(&format!("{field} value must be boolean")))
}
