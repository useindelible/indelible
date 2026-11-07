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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    TextQuote {
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

    /// Idempotent highlight create decides replay-versus-divergence by comparing locators, so
    /// equality has to be structural across every variant and every field, not variant-deep.
    #[test]
    fn locator_equality_is_structural_across_variants_and_fields() {
        let pdf = |page, snapshot: &str, rects| HighlightLocator::Pdf {
            page,
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
            text_snapshot: snapshot.into(),
            rects,
        };
        let rect = |x| {
            Some(vec![PdfRect {
                x,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            }])
        };

        assert_eq!(pdf(2, "quote", None), pdf(2, "quote", None));
        assert_ne!(pdf(2, "quote", None), pdf(3, "quote", None));
        assert_ne!(pdf(2, "quote", None), pdf(2, "other", None));
        assert_ne!(pdf(2, "quote", None), pdf(2, "quote", rect(1.0)));
        assert_ne!(pdf(2, "quote", rect(1.0)), pdf(2, "quote", rect(9.0)));

        let epub = |chapter: &str, start| HighlightLocator::Epub {
            chapter: chapter.into(),
            start_offset: start,
            end_offset: 20,
        };
        assert_eq!(epub("ch1", 10), epub("ch1", 10));
        assert_ne!(epub("ch1", 10), epub("ch2", 10));
        assert_ne!(epub("ch1", 10), epub("ch1", 11));

        assert_ne!(
            epub("ch1", 10),
            HighlightLocator::Html {
                start_offset: 10,
                end_offset: 20,
            },
            "different variants are never equal, even with matching offsets"
        );
    }
}
