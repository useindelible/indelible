use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{ApiError, FieldError};
use crate::extract::Validate;
use ind_application::TagWithMeta;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTagBody {
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
}

impl Validate for CreateTagBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if self.name.trim().is_empty() {
            errors.push(FieldError {
                field: "name".into(),
                message: "must not be empty".into(),
            });
        }
        if let Some(ref pid) = self.parent_id
            && parse_tag_id(pid).is_err()
        {
            errors.push(FieldError {
                field: "parent_id".into(),
                message: "invalid tag ID".into(),
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
pub struct UpdateTagBody {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[schema(value_type = Option<String>, nullable)]
    pub color: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_tag_id")]
    #[schema(value_type = Option<String>, nullable)]
    pub parent_id: Option<Option<String>>,
}

impl Validate for UpdateTagBody {
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
        let has_field = self.name.is_some() || self.color.is_some() || self.parent_id.is_some();
        if !has_field {
            errors.push(FieldError {
                field: "_".into(),
                message: "at least one field must be provided".into(),
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
pub struct MergeTagsBody {
    pub source_ids: Vec<String>,
    pub target_id: String,
}

impl Validate for MergeTagsBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if self.source_ids.is_empty() {
            errors.push(FieldError {
                field: "source_ids".into(),
                message: "must contain at least one source tag".into(),
            });
        }
        for (i, sid) in self.source_ids.iter().enumerate() {
            if parse_tag_id(sid).is_err() {
                errors.push(FieldError {
                    field: format!("source_ids[{i}]"),
                    message: "invalid tag ID".into(),
                });
            }
        }
        if parse_tag_id(&self.target_id).is_err() {
            errors.push(FieldError {
                field: "target_id".into(),
                message: "invalid tag ID".into(),
            });
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TagResponse {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub item_count: i64,
    pub highlight_count: i64,
    pub aliases: Vec<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl TagResponse {
    pub fn from_domain(twm: TagWithMeta) -> Self {
        Self {
            id: twm.tag.id.to_string(),
            object: "tag",
            name: twm.tag.name,
            color: twm.tag.color,
            parent_id: twm.tag.parent_id.map(|p| p.to_string()),
            item_count: twm.item_count,
            highlight_count: twm.highlight_count,
            aliases: twm.aliases,
            created_at: twm.tag.created_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListTagsParams {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub scope: Option<String>,
}

pub(crate) fn parse_tag_id(s: &str) -> Result<ind_domain::TagId, ApiError> {
    s.parse().map_err(|_| ApiError::NotFound {
        entity: "Tag",
        id: s.to_string(),
    })
}

fn deserialize_optional_nullable<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val: Option<Option<String>> = Option::deserialize(deserializer)?;
    Ok(val)
}

fn deserialize_optional_nullable_tag_id<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val: Option<Option<String>> = Option::deserialize(deserializer)?;
    Ok(val)
}
