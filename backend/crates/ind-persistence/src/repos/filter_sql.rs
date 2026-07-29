//! Column-parameterized SQL fragments for the Library smart-list `FilterNode` DSL.

use chrono::{DateTime, Duration, Utc};
use sqlx::{Postgres, QueryBuilder};

use ind_application::AppError;
use ind_domain::{DomainError, FilterOp};

pub(crate) fn validation_error(message: &str) -> AppError {
    AppError::Domain(DomainError::Validation {
        field: "filter_expression".into(),
        message: message.to_string(),
    })
}

pub(crate) fn push_boolean_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    column: &str,
    field: &str,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    if !matches!(op, FilterOp::Eq) {
        return Err(validation_error(&format!("{field} only supports eq")));
    }

    let parsed = value
        .as_bool()
        .ok_or_else(|| validation_error(&format!("{field} value must be boolean")))?;

    builder.push(column);
    builder.push(" = ");
    builder.push_bind(parsed);
    Ok(())
}

pub(crate) fn push_text_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    column: &str,
    field: &str,
    op: &FilterOp,
    value: &serde_json::Value,
    case_insensitive: bool,
) -> Result<(), AppError> {
    let sql_column = if case_insensitive {
        format!("LOWER({column})")
    } else {
        column.to_string()
    };

    match op {
        FilterOp::Eq => {
            let parsed = value
                .as_str()
                .ok_or_else(|| validation_error(&format!("{field} value must be a string")))?;
            let normalized = normalize_text_value(parsed, case_insensitive);

            builder.push(&sql_column);
            builder.push(" = ");
            builder.push_bind(normalized);
            Ok(())
        }
        FilterOp::Neq => {
            let parsed = value
                .as_str()
                .ok_or_else(|| validation_error(&format!("{field} value must be a string")))?;
            let normalized = normalize_text_value(parsed, case_insensitive);

            builder.push(&sql_column);
            builder.push(" != ");
            builder.push_bind(normalized);
            Ok(())
        }
        FilterOp::Contains => {
            let parsed = value
                .as_str()
                .ok_or_else(|| validation_error(&format!("{field} value must be a string")))?;
            let normalized = normalize_text_value(parsed, case_insensitive);

            builder.push(&sql_column);
            builder.push(" LIKE '%' || ");
            builder.push_bind(normalized);
            builder.push(" || '%'");
            Ok(())
        }
        FilterOp::In => {
            let values = value
                .as_array()
                .ok_or_else(|| validation_error(&format!("{field} IN value must be an array")))?
                .iter()
                .map(|entry| {
                    entry
                        .as_str()
                        .map(|text| normalize_text_value(text, case_insensitive))
                        .ok_or_else(|| {
                            validation_error(&format!("{field} IN values must be strings"))
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;

            builder.push(&sql_column);
            builder.push(" = ANY(");
            builder.push_bind(values);
            builder.push(")");
            Ok(())
        }
        _ => Err(validation_error(&format!(
            "{field} only supports eq, neq, contains, in"
        ))),
    }
}

pub(crate) fn push_timestamp_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    column: &str,
    field: &str,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    let timestamp = value
        .as_str()
        .ok_or_else(|| validation_error(&format!("{field} value must be an ISO 8601 string")))?
        .parse::<DateTime<Utc>>()
        .map_err(|_| validation_error(&format!("{field} value is not a valid timestamp")))?;
    let next_day = timestamp + Duration::days(1);

    match op {
        FilterOp::Eq => {
            builder.push("(");
            builder.push(column);
            builder.push(" >= ");
            builder.push_bind(timestamp);
            builder.push(" AND ");
            builder.push(column);
            builder.push(" < ");
            builder.push_bind(next_day);
            builder.push(")");
            Ok(())
        }
        FilterOp::Gt => {
            builder.push(column);
            builder.push(" >= ");
            builder.push_bind(next_day);
            Ok(())
        }
        FilterOp::Lt => {
            builder.push(column);
            builder.push(" < ");
            builder.push_bind(timestamp);
            Ok(())
        }
        FilterOp::Gte => {
            builder.push(column);
            builder.push(" >= ");
            builder.push_bind(timestamp);
            Ok(())
        }
        FilterOp::Lte => {
            builder.push(column);
            builder.push(" < ");
            builder.push_bind(next_day);
            Ok(())
        }
        _ => Err(validation_error(&format!(
            "{field} does not support this operator"
        ))),
    }
}

fn normalize_text_value(value: &str, case_insensitive: bool) -> String {
    if case_insensitive {
        value.to_lowercase()
    } else {
        value.to_string()
    }
}

pub(crate) fn parse_prefixed_uuid(value: &str, prefix: &str) -> Result<uuid::Uuid, uuid::Error> {
    value.strip_prefix(prefix).unwrap_or(value).parse()
}
