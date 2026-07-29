use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{ApiError, FieldError};
use crate::extract::Validate;

/// Schema-only flat representation of the LocatorSchema tagged union for OpenAPI code generators.
/// The real runtime type is `LocatorSchema` (internally tagged enum), but utoipa's `oneOf`
/// without a discriminator produces `Any` in Kotlin/TS generators. This flat struct captures
/// the union of all variant fields so code generators produce a concrete type.
#[derive(Serialize, ToSchema)]
pub struct LocatorSchemaFlat {
    /// Discriminator: "html", "epub", or "pdf"
    #[serde(rename = "type")]
    pub locator_type: String,
    /// HTML/EPUB: start offset
    pub start_offset: Option<i64>,
    /// HTML/EPUB: end offset
    pub end_offset: Option<i64>,
    /// EPUB: chapter identifier
    pub chapter: Option<String>,
    /// PDF: page number
    pub page: Option<i32>,
    /// PDF: x coordinate
    pub x: Option<f64>,
    /// PDF: y coordinate
    pub y: Option<f64>,
    /// PDF: width
    pub width: Option<f64>,
    /// PDF: height
    pub height: Option<f64>,
    /// PDF: selected text snapshot
    pub text_snapshot: Option<String>,
    /// PDF: bounding rectangles
    pub rects: Option<Vec<PdfRectSchema>>,
}

#[derive(Serialize, ToSchema)]
pub struct SourceLocatorSchemaFlat {
    /// Discriminator: "web_page_dom_range"
    #[serde(rename = "type")]
    pub locator_type: String,
    pub url: Option<String>,
    pub location: Option<String>,
    pub offset: Option<i64>,
    pub text_content: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PdfRectSchema {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LocatorSchema {
    Html {
        start_offset: i64,
        end_offset: i64,
    },
    Epub {
        chapter: String,
        start_offset: i64,
        end_offset: i64,
    },
    Pdf {
        page: i32,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        text_snapshot: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rects: Option<Vec<PdfRectSchema>>,
    },
}

impl From<LocatorSchema> for ind_domain::HighlightLocator {
    fn from(ls: LocatorSchema) -> Self {
        match ls {
            LocatorSchema::Html {
                start_offset,
                end_offset,
            } => Self::Html {
                start_offset,
                end_offset,
            },
            LocatorSchema::Epub {
                chapter,
                start_offset,
                end_offset,
            } => Self::Epub {
                chapter,
                start_offset,
                end_offset,
            },
            LocatorSchema::Pdf {
                page,
                x,
                y,
                width,
                height,
                text_snapshot,
                rects,
            } => Self::Pdf {
                page,
                x,
                y,
                width,
                height,
                text_snapshot,
                rects: rects.map(|rs| {
                    rs.into_iter()
                        .map(|r| ind_domain::PdfRect {
                            x: r.x,
                            y: r.y,
                            width: r.width,
                            height: r.height,
                        })
                        .collect()
                }),
            },
        }
    }
}

impl From<ind_domain::HighlightLocator> for LocatorSchema {
    fn from(hl: ind_domain::HighlightLocator) -> Self {
        match hl {
            ind_domain::HighlightLocator::Html {
                start_offset,
                end_offset,
            } => Self::Html {
                start_offset,
                end_offset,
            },
            ind_domain::HighlightLocator::Epub {
                chapter,
                start_offset,
                end_offset,
            } => Self::Epub {
                chapter,
                start_offset,
                end_offset,
            },
            ind_domain::HighlightLocator::Pdf {
                page,
                x,
                y,
                width,
                height,
                text_snapshot,
                rects,
            } => Self::Pdf {
                page,
                x,
                y,
                width,
                height,
                text_snapshot,
                rects: rects.map(|rs| {
                    rs.into_iter()
                        .map(|r| PdfRectSchema {
                            x: r.x,
                            y: r.y,
                            width: r.width,
                            height: r.height,
                        })
                        .collect()
                }),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SourceLocatorSchema {
    WebPageDomRange {
        url: String,
        location: String,
        offset: Option<i64>,
        text_content: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prefix: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        suffix: Option<String>,
    },
}

impl From<SourceLocatorSchema> for ind_domain::HighlightSourceLocator {
    fn from(ls: SourceLocatorSchema) -> Self {
        match ls {
            SourceLocatorSchema::WebPageDomRange {
                url,
                location,
                offset,
                text_content,
                prefix,
                suffix,
            } => Self::WebPageDomRange {
                url,
                location,
                offset,
                text_content,
                prefix,
                suffix,
            },
        }
    }
}

impl From<ind_domain::HighlightSourceLocator> for SourceLocatorSchema {
    fn from(locator: ind_domain::HighlightSourceLocator) -> Self {
        match locator {
            ind_domain::HighlightSourceLocator::WebPageDomRange {
                url,
                location,
                offset,
                text_content,
                prefix,
                suffix,
            } => Self::WebPageDomRange {
                url,
                location,
                offset,
                text_content,
                prefix,
                suffix,
            },
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateHighlightBody {
    pub color: String,
    pub text_content: String,
    #[schema(value_type = LocatorSchemaFlat)]
    pub locator: LocatorSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<SourceLocatorSchemaFlat>)]
    pub source_locator: Option<SourceLocatorSchema>,
}

impl Validate for CreateHighlightBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if self.color.trim().is_empty() {
            errors.push(FieldError {
                field: "color".into(),
                message: "must not be empty".into(),
            });
        }
        if self.text_content.trim().is_empty() {
            errors.push(FieldError {
                field: "text_content".into(),
                message: "must not be empty".into(),
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
pub struct HighlightResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    pub color: String,
    pub text_content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = LocatorSchemaFlat)]
    pub locator: Option<LocatorSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<SourceLocatorSchemaFlat>)]
    pub source_locator: Option<SourceLocatorSchema>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

impl HighlightResponse {
    pub fn from_domain(h: ind_domain::Highlight) -> Self {
        Self {
            id: h.id.to_string(),
            document_id: Some(h.document_id.to_string()),
            color: h.color,
            text_content: h.text_content,
            locator: h.locator.map(LocatorSchema::from),
            source_locator: h.source_locator.map(SourceLocatorSchema::from),
            created_at: h.created_at,
            updated_at: h.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HighlightNoteResponse {
    pub id: String,
    pub highlight_id: String,
    pub body: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

impl HighlightNoteResponse {
    pub fn from_domain(n: ind_domain::HighlightNote) -> Self {
        Self {
            id: n.id.to_string(),
            highlight_id: n.highlight_id.to_string(),
            body: n.body,
            created_at: n.created_at,
            updated_at: n.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HighlightWithNoteResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    pub color: String,
    pub text_content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = LocatorSchemaFlat)]
    pub locator: Option<LocatorSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<SourceLocatorSchemaFlat>)]
    pub source_locator: Option<SourceLocatorSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<HighlightNoteResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_title: Option<String>,
    pub tags: Vec<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

impl HighlightWithNoteResponse {
    pub fn from_domain(hwn: crate::state::HighlightWithNote) -> Self {
        Self {
            id: hwn.highlight.id.to_string(),
            document_id: Some(hwn.highlight.document_id.to_string()),
            color: hwn.highlight.color,
            text_content: hwn.highlight.text_content,
            locator: hwn.highlight.locator.map(LocatorSchema::from),
            source_locator: hwn.highlight.source_locator.map(SourceLocatorSchema::from),
            note: hwn.note.map(HighlightNoteResponse::from_domain),
            item_title: None,
            tags: hwn.tags.into_iter().map(|t| t.name).collect(),
            created_at: hwn.highlight.created_at,
            updated_at: hwn.highlight.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HighlightListResponse {
    pub highlights: Vec<HighlightWithNoteResponse>,
    pub count: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RecentHighlightsResponse {
    pub highlights: Vec<HighlightWithNoteResponse>,
    pub count: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PatchHighlightBody {
    pub color: Option<String>,
}

impl Validate for PatchHighlightBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        if self.color.is_none() {
            return Err(vec![FieldError {
                field: "color".into(),
                message: "at least one field must be provided".into(),
            }]);
        }
        if let Some(ref c) = self.color
            && c.trim().is_empty()
        {
            return Err(vec![FieldError {
                field: "color".into(),
                message: "must not be empty".into(),
            }]);
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertNoteBody {
    pub body: String,
}

impl Validate for UpsertNoteBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        if self.body.trim().is_empty() {
            return Err(vec![FieldError {
                field: "body".into(),
                message: "must not be empty".into(),
            }]);
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct HighlightTagsBody {
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HighlightTagsResponse {
    pub tags: Vec<String>,
}

pub(crate) fn parse_highlight_id(s: &str) -> Result<ind_domain::HighlightId, ApiError> {
    s.parse().map_err(|_| ApiError::NotFound {
        entity: "Highlight",
        id: s.to_string(),
    })
}
