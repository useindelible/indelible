use thiserror::Error;

#[derive(Debug, Error)]
pub enum PdfExtractionError {
    #[error("failed to parse PDF: {0}")]
    Parse(String),
    #[error("PDF is password-protected")]
    PasswordProtected,
    #[error("failed to extract text from PDF: {0}")]
    Extract(String),
}

pub fn extract_pdf_text(bytes: &[u8]) -> Result<String, PdfExtractionError> {
    let document = pdf_oxide::document::PdfDocument::from_bytes(bytes.to_vec())
        .map_err(|err| PdfExtractionError::Parse(err.to_string()))?;
    if document.is_encrypted() {
        match document.authenticate(b"") {
            Ok(false) => return Err(PdfExtractionError::PasswordProtected),
            Ok(true) => {}
            Err(err) => return Err(PdfExtractionError::Parse(err.to_string())),
        }
    }
    let text = document
        .extract_all_text()
        .map_err(|err| PdfExtractionError::Extract(err.to_string()))?;
    let normalized = normalize_whitespace(&text);
    if normalized.trim().is_empty() {
        Err(PdfExtractionError::Extract("no extractable text".into()))
    } else {
        Ok(normalized)
    }
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_pdf_text_rejects_invalid_bytes() {
        let result = extract_pdf_text(b"not a pdf");
        assert!(result.is_err());
    }

    #[test]
    fn normalize_whitespace_collapses_runs() {
        assert_eq!(normalize_whitespace("  hello   world  "), "hello world");
    }
}
