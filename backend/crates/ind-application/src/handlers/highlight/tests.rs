use ind_domain::{DocumentType, DomainError, HighlightLocator, HighlightSourceLocator, PdfRect};

use super::validation::{
    validate_color, validate_highlight_locators_for_document, validate_locator,
};
use crate::AppError;

fn field(result: Result<(), AppError>) -> String {
    match result.unwrap_err() {
        AppError::Domain(DomainError::Validation { field, .. }) => field,
        other => panic!("expected validation error, got {other:?}"),
    }
}

fn pdf(x: f64, y: f64, width: f64, height: f64, text: &str) -> HighlightLocator {
    HighlightLocator::Pdf {
        page: 1,
        x,
        y,
        width,
        height,
        text_snapshot: text.into(),
        rects: None,
    }
}

fn source(url: &str, location: &str, text: &str) -> HighlightSourceLocator {
    HighlightSourceLocator::WebPageDomRange {
        url: url.into(),
        location: location.into(),
        offset: Some(3),
        text_content: text.into(),
        prefix: None,
        suffix: None,
    }
}

#[test]
fn colors_offsets_and_document_types_enforce_the_public_highlight_contract() {
    for color in ["yellow", "blue", "green", "pink", "purple"] {
        validate_color(color).unwrap();
    }
    assert_eq!(field(validate_color("red")), "color");

    let html = HighlightLocator::Html {
        start_offset: 0,
        end_offset: 10,
    };
    let epub = HighlightLocator::Epub {
        chapter: "chapter-1".into(),
        start_offset: 0,
        end_offset: 10,
    };
    validate_highlight_locators_for_document(DocumentType::Article, Some(&html), None).unwrap();
    validate_highlight_locators_for_document(DocumentType::Book, Some(&epub), None).unwrap();
    assert_eq!(
        field(validate_locator(&HighlightLocator::Html {
            start_offset: -1,
            end_offset: 10,
        })),
        "locator.start_offset"
    );
    assert_eq!(
        field(validate_locator(&HighlightLocator::Epub {
            chapter: "chapter-1".into(),
            start_offset: 10,
            end_offset: 10,
        })),
        "locator.end_offset"
    );
    assert_eq!(
        field(validate_highlight_locators_for_document(
            DocumentType::Pdf,
            Some(&html),
            None,
        )),
        "locator.type"
    );
    assert_eq!(
        field(validate_highlight_locators_for_document(
            DocumentType::Article,
            None,
            None,
        )),
        "locator"
    );
}

#[test]
fn pdf_rectangles_and_source_ranges_reject_each_invalid_dimension() {
    let mut valid = pdf(0.1, 0.2, 0.5, 0.1, "selected text");
    if let HighlightLocator::Pdf { rects, .. } = &mut valid {
        *rects = Some(vec![PdfRect {
            x: 0.2,
            y: 0.3,
            width: 0.2,
            height: 0.1,
        }]);
    }
    validate_highlight_locators_for_document(DocumentType::Pdf, Some(&valid), None).unwrap();

    let cases = [
        (pdf(-0.1, 0.2, 0.5, 0.1, "text"), "locator.x"),
        (pdf(0.1, 1.1, 0.5, 0.1, "text"), "locator.y"),
        (pdf(0.1, 0.2, 0.0, 0.1, "text"), "locator.width"),
        (pdf(0.1, 0.2, 0.5, 0.0, "text"), "locator.height"),
        (pdf(0.8, 0.2, 0.3, 0.1, "text"), "locator.width"),
        (pdf(0.1, 0.8, 0.5, 0.3, "text"), "locator.height"),
        (pdf(0.1, 0.2, 0.5, 0.1, " "), "locator.text_snapshot"),
    ];
    for (locator, expected) in cases {
        assert_eq!(field(validate_locator(&locator)), expected);
    }

    let valid_source = source("https://example.com/article", "body > p", "quote");
    validate_highlight_locators_for_document(DocumentType::Article, None, Some(&valid_source))
        .unwrap();
    for (source_locator, expected) in [
        (source(" ", "body", "quote"), "source_locator.url"),
        (
            source("https://example.com", " ", "quote"),
            "source_locator.location",
        ),
        (
            source("https://example.com", "body", " "),
            "source_locator.text_content",
        ),
    ] {
        assert_eq!(
            field(validate_highlight_locators_for_document(
                DocumentType::Article,
                None,
                Some(&source_locator),
            )),
            expected
        );
    }
}
