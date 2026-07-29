use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{ApiError, FieldError};
use crate::extract::Validate;
use ind_application::CollectionWithCount;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCollectionBody {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub parent_id: Option<String>,
}

impl Validate for CreateCollectionBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if self.name.trim().is_empty() {
            errors.push(FieldError {
                field: "name".into(),
                message: "must not be empty".into(),
            });
        }
        if let Some(ref pid) = self.parent_id
            && parse_collection_id(pid).is_err()
        {
            errors.push(FieldError {
                field: "parent_id".into(),
                message: "invalid collection ID".into(),
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
pub struct UpdateCollectionBody {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[schema(value_type = Option<String>, nullable)]
    pub description: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[schema(value_type = Option<String>, nullable)]
    pub icon: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[schema(value_type = Option<String>, nullable)]
    pub color: Option<Option<String>>,
    #[serde(default)]
    pub sort_order: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[schema(value_type = Option<String>, nullable)]
    pub parent_id: Option<Option<String>>,
}

impl Validate for UpdateCollectionBody {
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
            || self.description.is_some()
            || self.icon.is_some()
            || self.color.is_some()
            || self.sort_order.is_some()
            || self.parent_id.is_some();
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

/// Add a saved library entry to a collection (TASK-235, Library-entry-keyed membership).
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddLibraryEntryBody {
    pub library_entry_id: String,
}

impl Validate for AddLibraryEntryBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        if parse_library_entry_id(&self.library_entry_id).is_err() {
            Err(vec![FieldError {
                field: "library_entry_id".into(),
                message: "invalid library entry ID".into(),
            }])
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionResponse {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub sort_order: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub item_count: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

impl CollectionResponse {
    pub fn from_domain(cwc: CollectionWithCount) -> Self {
        Self {
            id: cwc.collection.id.to_string(),
            object: "collection",
            name: cwc.collection.name,
            description: cwc.collection.description,
            icon: cwc.collection.icon,
            color: cwc.collection.color,
            sort_order: cwc.collection.sort_order,
            parent_id: cwc.collection.parent_id.map(|p| p.to_string()),
            item_count: cwc.item_count,
            created_at: cwc.collection.created_at,
            updated_at: cwc.collection.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListCollectionsParams {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

pub(crate) fn parse_collection_id(s: &str) -> Result<ind_domain::CollectionId, ApiError> {
    s.parse().map_err(|_| ApiError::NotFound {
        entity: "Collection",
        id: s.to_string(),
    })
}

pub(crate) fn parse_library_entry_id(s: &str) -> Result<ind_domain::LibraryEntryId, ApiError> {
    s.parse().map_err(|_| ApiError::NotFound {
        entity: "LibraryEntry",
        id: s.to_string(),
    })
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

        // JSON null: field is present and explicitly null → clear the value
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(Some(None))
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(Some(None))
        }

        // JSON string: field is present with a value → set it
        fn visit_some<D2: serde::Deserializer<'de>>(self, d: D2) -> Result<Self::Value, D2::Error> {
            Ok(Some(Some(String::deserialize(d)?)))
        }
    }

    deserializer.deserialize_option(V)
}
