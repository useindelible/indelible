use ind_domain::{Document, SearchIndexedHighlight};

/// Metadata text for a durable document root row (TASK-233). Excludes tags/collections/entities,
/// which are not yet document-keyed (Phase 10).
pub(crate) fn build_document_metadata_text(document: &Document, note: Option<&str>) -> String {
    [
        document.author.as_deref().unwrap_or_default(),
        document.domain.as_deref().unwrap_or_default(),
        document.original_url.as_deref().unwrap_or_default(),
        document.canonical_url.as_deref().unwrap_or_default(),
        document.excerpt.as_deref().unwrap_or_default(),
        note.unwrap_or_default(),
    ]
    .iter()
    .filter(|segment| !segment.trim().is_empty())
    .map(|segment| segment.trim())
    .collect::<Vec<_>>()
    .join("\n")
}

/// Metadata text for a durable document section (EPUB chapter) row.
pub(crate) fn build_document_section_metadata_text(document: &Document) -> String {
    [
        document.author.as_deref().unwrap_or_default(),
        document.domain.as_deref().unwrap_or_default(),
        document.original_url.as_deref().unwrap_or_default(),
        document.canonical_url.as_deref().unwrap_or_default(),
    ]
    .iter()
    .filter(|segment| !segment.trim().is_empty())
    .map(|segment| segment.trim())
    .collect::<Vec<_>>()
    .join("\n")
}

pub(crate) fn join_highlights(highlights: &[SearchIndexedHighlight]) -> String {
    highlights
        .iter()
        .map(|highlight| match highlight.note.as_deref() {
            Some(note) if !note.trim().is_empty() => {
                format!("{}\n{}", highlight.text.trim(), note.trim())
            }
            _ => highlight.text.trim().to_string(),
        })
        .filter(|segment| !segment.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
