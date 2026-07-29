use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{ApiError, FieldError};
use crate::extract::Validate;
use ind_domain::FilterNode;

// Schema-only mirror of ind_domain::FilterNode; keep serde tags and variants in lockstep.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FilterExpressionNode {
    And {
        #[schema(no_recursion)]
        conditions: Vec<FilterExpressionNode>,
    },
    Or {
        #[schema(no_recursion)]
        conditions: Vec<FilterExpressionNode>,
    },
    Not {
        #[schema(no_recursion)]
        condition: Box<FilterExpressionNode>,
    },
    Condition {
        field: String,
        op: FilterExpressionOperator,
        value: FilterExpressionValue,
    },
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterExpressionOperator {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    Contains,
    In,
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(untagged)]
pub enum FilterExpressionValue {
    String(String),
    Bool(bool),
    Number(f64),
    Strings(Vec<String>),
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSmartListBody {
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[schema(value_type = FilterExpressionNode)]
    pub filter_expression: FilterNode,
    #[serde(default)]
    pub default_sort: Option<String>,
}

impl Validate for CreateSmartListBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if self.name.trim().is_empty() {
            errors.push(FieldError {
                field: "name".into(),
                message: "must not be empty".into(),
            });
        }
        if let Err(message) = self.filter_expression.validate() {
            errors.push(FieldError {
                field: "filter_expression".into(),
                message,
            });
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSmartListBody {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[schema(value_type = Option<String>, nullable)]
    pub icon: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[schema(value_type = Option<String>, nullable)]
    pub color: Option<Option<String>>,
    #[serde(default)]
    #[schema(value_type = Option<FilterExpressionNode>)]
    #[serde(deserialize_with = "deserialize_optional_filter_node")]
    pub filter_expression: Option<FilterNode>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[schema(value_type = Option<String>, nullable)]
    pub default_sort: Option<Option<String>>,
    #[serde(default)]
    pub is_pinned: Option<bool>,
}

impl Validate for UpdateSmartListBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if let Some(ref name) = self.name
            && name.trim().is_empty()
        {
            errors.push(FieldError {
                field: "name".into(),
                message: "must not be empty".into(),
            });
        }
        let has_field = self.name.is_some()
            || self.icon.is_some()
            || self.color.is_some()
            || self.filter_expression.is_some()
            || self.default_sort.is_some()
            || self.is_pinned.is_some();
        if !has_field {
            errors.push(FieldError {
                field: "_".into(),
                message: "at least one field must be provided".into(),
            });
        }
        if let Some(filter_expression) = self.filter_expression.as_ref()
            && let Err(message) = filter_expression.validate()
        {
            errors.push(FieldError {
                field: "filter_expression".into(),
                message,
            });
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PinSmartListBody {
    pub is_pinned: bool,
}

impl Validate for PinSmartListBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SmartListResponse {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub is_pinned: bool,
    #[schema(value_type = FilterExpressionNode)]
    pub filter_expression: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_sort: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

impl SmartListResponse {
    pub fn from_domain(sl: ind_domain::SmartList) -> Self {
        #[expect(
            clippy::expect_used,
            reason = "FilterNode is a derived-Serialize enum of struct variants; serializing it to a JSON value is infallible"
        )]
        let filter_expression = serde_json::to_value(sl.filter_expression)
            .expect("filter expression output serializes");
        Self {
            id: sl.id.to_string(),
            object: "smart_list",
            name: sl.name,
            icon: sl.icon,
            color: sl.color,
            is_pinned: sl.is_pinned,
            filter_expression,
            default_sort: sl.default_sort,
            created_at: sl.created_at,
            updated_at: sl.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListSmartListsParams {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

pub(crate) fn parse_smart_list_id(s: &str) -> Result<ind_domain::SmartListId, ApiError> {
    s.parse().map_err(|_| ApiError::NotFound {
        entity: "SmartList",
        id: s.to_string(),
    })
}

fn deserialize_optional_filter_node<'de, D>(deserializer: D) -> Result<Option<FilterNode>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Err(serde::de::Error::custom(
            "filter_expression must not be null",
        ));
    };

    FilterNode::deserialize(value)
        .map(Some)
        .map_err(serde::de::Error::custom)
}

fn deserialize_optional_nullable<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Visitor;

    struct V;

    impl<'de> Visitor<'de> for V {
        type Value = Option<Option<String>>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "null or a string")
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(Some(None))
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(Some(None))
        }

        fn visit_some<D2: serde::Deserializer<'de>>(self, d: D2) -> Result<Self::Value, D2::Error> {
            Ok(Some(Some(String::deserialize(d)?)))
        }
    }

    deserializer.deserialize_option(V)
}
