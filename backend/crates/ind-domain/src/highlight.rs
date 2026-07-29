use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{DocumentId, HighlightId, HighlightNoteId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PdfRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A highlight is anchored to a document.
#[derive(Debug)]
pub struct NewHighlight {
    pub id: HighlightId,
    pub document_id: DocumentId,
    pub user_id: UserId,
    pub color: String,
    pub text_content: String,
    pub locator: Option<HighlightLocator>,
    pub source_locator: Option<HighlightSourceLocator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub id: HighlightId,
    pub document_id: DocumentId,
    pub user_id: UserId,
    pub color: String,
    pub text_content: String,
    pub locator: Option<HighlightLocator>,
    pub source_locator: Option<HighlightSourceLocator>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HighlightLocator {
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
        rects: Option<Vec<PdfRect>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HighlightSourceLocator {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightNote {
    pub id: HighlightNoteId,
    pub highlight_id: HighlightId,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_pdf_locator_without_rects_remains_readable() {
        let locator: HighlightLocator = serde_json::from_value(serde_json::json!({
            "type": "pdf", "page": 2, "x": 1.0, "y": 2.0,
            "width": 3.0, "height": 4.0, "text_snapshot": "quote"
        }))
        .unwrap();
        assert!(matches!(locator, HighlightLocator::Pdf { rects: None, .. }));
    }
}
