use ind_domain::{DocumentType, DomainError, HighlightLocator, HighlightSourceLocator};

use crate::AppError;

const ALLOWED_COLORS: &[&str] = &["yellow", "blue", "green", "pink", "purple"];

pub(crate) fn validate_color(color: &str) -> Result<(), AppError> {
    if ALLOWED_COLORS.contains(&color) {
        Ok(())
    } else {
        Err(AppError::Domain(DomainError::Validation {
            field: "color".into(),
            message: format!(
                "invalid color '{color}'; allowed values: {}",
                ALLOWED_COLORS.join(", ")
            ),
        }))
    }
}

pub(super) fn validate_locator(locator: &HighlightLocator) -> Result<(), AppError> {
    match locator {
        HighlightLocator::Html {
            start_offset,
            end_offset,
        }
        | HighlightLocator::Epub {
            start_offset,
            end_offset,
            ..
        } => {
            if *start_offset < 0 {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "locator.start_offset".into(),
                    message: "start_offset must be >= 0".into(),
                }));
            }
            if *end_offset <= *start_offset {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "locator.end_offset".into(),
                    message: "end_offset must be greater than start_offset".into(),
                }));
            }
            Ok(())
        }
        HighlightLocator::Pdf {
            page,
            x,
            y,
            width,
            height,
            text_snapshot,
            rects,
        } => {
            validate_pdf_locator(*page, *x, *y, *width, *height, text_snapshot)?;
            if let Some(rect_list) = rects {
                for (i, r) in rect_list.iter().enumerate() {
                    let prefix = format!("locator.rects[{i}]");
                    validate_pdf_rect(&prefix, r.x, r.y, r.width, r.height)?;
                }
            }
            Ok(())
        }
    }
}

pub(crate) fn validate_highlight_locators_for_document(
    document_type: DocumentType,
    locator: Option<&HighlightLocator>,
    source_locator: Option<&HighlightSourceLocator>,
) -> Result<(), AppError> {
    if locator.is_none() && source_locator.is_none() {
        return Err(AppError::Domain(DomainError::Validation {
            field: "locator".into(),
            message: "at least one of locator or source_locator is required".into(),
        }));
    }

    if let Some(locator) = locator {
        validate_locator(locator)?;
        let compatible = match document_type {
            DocumentType::Pdf => matches!(locator, HighlightLocator::Pdf { .. }),
            DocumentType::Book => matches!(locator, HighlightLocator::Epub { .. }),
            DocumentType::Article
            | DocumentType::Email
            | DocumentType::Tweet
            | DocumentType::Video
            | DocumentType::Podcast => matches!(locator, HighlightLocator::Html { .. }),
        };
        if !compatible {
            return Err(AppError::Domain(DomainError::Validation {
                field: "locator.type".into(),
                message: format!("locator is not valid for document type {document_type}"),
            }));
        }
    }
    if let Some(source_locator) = source_locator {
        validate_source_locator(source_locator)?;
    }

    Ok(())
}

fn validate_source_locator(locator: &HighlightSourceLocator) -> Result<(), AppError> {
    match locator {
        HighlightSourceLocator::WebPageDomRange {
            url,
            location,
            text_content,
            ..
        } => {
            if url.trim().is_empty() {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "source_locator.url".into(),
                    message: "url must not be empty".into(),
                }));
            }
            if location.trim().is_empty() {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "source_locator.location".into(),
                    message: "location must not be empty".into(),
                }));
            }
            if text_content.trim().is_empty() {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "source_locator.text_content".into(),
                    message: "text_content must not be empty".into(),
                }));
            }
            Ok(())
        }
    }
}

fn validate_pdf_locator(
    page: i32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    text_snapshot: &str,
) -> Result<(), AppError> {
    if page < 1 {
        return validation_error("locator.page", "page must be >= 1");
    }
    validate_pdf_rect("locator", x, y, width, height)?;
    if text_snapshot.trim().is_empty() {
        return validation_error("locator.text_snapshot", "text_snapshot must not be empty");
    }
    Ok(())
}

fn validate_pdf_rect(
    prefix: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), AppError> {
    if !x.is_nan() && !(0.0..=1.0).contains(&x) {
        return validation_error(&format!("{prefix}.x"), "x must be in [0.0, 1.0]");
    }
    if !y.is_nan() && !(0.0..=1.0).contains(&y) {
        return validation_error(&format!("{prefix}.y"), "y must be in [0.0, 1.0]");
    }
    if width <= 0.0 || width > 1.0 {
        return validation_error(&format!("{prefix}.width"), "width must be in (0.0, 1.0]");
    }
    if height <= 0.0 || height > 1.0 {
        return validation_error(&format!("{prefix}.height"), "height must be in (0.0, 1.0]");
    }
    if x + width > 1.0 {
        return validation_error(&format!("{prefix}.width"), "x + width must be <= 1.0");
    }
    if y + height > 1.0 {
        return validation_error(&format!("{prefix}.height"), "y + height must be <= 1.0");
    }
    Ok(())
}

fn validation_error(field: &str, message: &str) -> Result<(), AppError> {
    Err(AppError::Domain(DomainError::Validation {
        field: field.into(),
        message: message.into(),
    }))
}
