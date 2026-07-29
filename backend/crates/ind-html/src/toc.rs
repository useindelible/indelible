//! Article table-of-contents derivation: a read-only pass over prepared reader
//! HTML (see `prepare`) that turns the heading structure into a flat outline.
//!
//! Depths are relative, not raw tag levels — the shallowest tag in the document
//! is depth 0 and level jumps clamp to one step — because real articles root
//! their outline anywhere (`h2` on Wikipedia) and skip levels freely.

use scraper::{ElementRef, Html, Node};
use serde::{Deserialize, Serialize};

/// Below this many surviving entries an outline is navigation noise, so the
/// document is marked as having no ToC — a terminal, persisted answer.
const MIN_ENTRIES: usize = 2;
/// Pathological pages (auto-generated glossaries, scraped indexes) can carry
/// thousands of headings; the outline stops being navigation long before that.
const MAX_ENTRIES: usize = 200;
/// A first heading at least this long that prefixes the document title is the
/// article restating its own title (document titles usually carry site suffixes,
/// so exact equality alone never fires). Shorter prefixes ("Intro" vs
/// "Introduction to X") are legitimate sections and survive.
const TITLE_PREFIX_MIN_CHARS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArticleTocStatus {
    Ready,
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticleTocEntry {
    /// Document-order ordinal among ALL headings, counted before empty-heading
    /// and title dedupe — the client's positional fallback when an anchor id is
    /// missing from cached content.
    pub source_heading_index: u32,
    pub id: String,
    pub title: String,
    pub depth: u8,
    pub word_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticleToc {
    pub status: ArticleTocStatus,
    pub truncated: bool,
    pub entries: Vec<ArticleTocEntry>,
}

impl ArticleToc {
    fn none() -> Self {
        Self {
            status: ArticleTocStatus::None,
            truncated: false,
            entries: Vec::new(),
        }
    }
}

/// Derive the outline of a prepared article. Infallible by construction: the
/// worst input yields `status: none`.
pub fn derive_article_toc(prepared_html: &str, document_title: &str) -> ArticleToc {
    let doc = Html::parse_document(prepared_html);

    let mut entries: Vec<ArticleTocEntry> = Vec::new();
    let mut levels: Vec<u8> = Vec::new();
    let mut heading_ordinal: u32 = 0;
    // Word counts attribute to the last surviving entry at the point the text
    // appears; text before the first surviving heading is preamble and uncounted.
    let mut current_entry: Option<usize> = None;

    for node in doc.root_element().descendants() {
        if let Some(element) = ElementRef::wrap(node) {
            let Some(level) = heading_level(element.value().name()) else {
                continue;
            };
            let ordinal = heading_ordinal;
            heading_ordinal += 1;

            let title = normalize_text(&element.text().collect::<String>());
            if title.is_empty() {
                continue;
            }
            if ordinal == 0 && restates_document_title(&title, document_title) {
                continue;
            }
            entries.push(ArticleTocEntry {
                source_heading_index: ordinal,
                id: element.value().attr("id").unwrap_or_default().to_string(),
                title,
                depth: 0,
                word_count: 0,
            });
            levels.push(level);
            current_entry = Some(entries.len() - 1);
            continue;
        }

        let Node::Text(text) = node.value() else {
            continue;
        };
        let Some(entry_index) = current_entry else {
            continue;
        };
        // Text inside any heading element is a (current or skipped) title, not
        // section content.
        if node
            .ancestors()
            .filter_map(ElementRef::wrap)
            .any(|el| heading_level(el.value().name()).is_some())
        {
            continue;
        }
        entries[entry_index].word_count += text.split_whitespace().count() as u32;
    }

    if entries.len() < MIN_ENTRIES {
        return ArticleToc::none();
    }

    for (entry, depth) in entries.iter_mut().zip(normalized_depths(&levels)) {
        entry.depth = depth;
    }

    let truncated = entries.len() > MAX_ENTRIES;
    entries.truncate(MAX_ENTRIES);

    ArticleToc {
        status: ArticleTocStatus::Ready,
        truncated,
        entries,
    }
}

fn heading_level(name: &str) -> Option<u8> {
    match name {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}

/// Relative outline depths: shallowest tag = depth 0, level jumps clamp to +1.
fn normalized_depths(levels: &[u8]) -> Vec<u8> {
    let mut stack: Vec<(u8, u8)> = Vec::new();
    let mut out = Vec::with_capacity(levels.len());
    for &level in levels {
        while stack.last().is_some_and(|&(l, _)| l >= level) {
            stack.pop();
        }
        let depth = stack.last().map_or(0, |&(_, d)| d + 1);
        stack.push((level, depth));
        out.push(depth);
    }
    out
}

fn restates_document_title(heading: &str, document_title: &str) -> bool {
    let heading = comparable(heading);
    let title = comparable(document_title);
    if heading.is_empty() || title.is_empty() {
        return false;
    }
    heading == title
        || (title.starts_with(&heading) && heading.chars().count() >= TITLE_PREFIX_MIN_CHARS)
}

/// Lowercased, punctuation-free, whitespace-collapsed comparison form.
fn comparable(text: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    for c in text.chars() {
        if c.is_alphanumeric() {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            for lower in c.to_lowercase() {
                out.push(lower);
            }
        } else {
            pending_space = true;
        }
    }
    out
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depths_reset_after_pops() {
        assert_eq!(
            normalized_depths(&[2, 3, 4, 5, 3, 2]),
            vec![0, 1, 2, 3, 1, 0]
        );
        assert_eq!(normalized_depths(&[2, 5, 2]), vec![0, 1, 0]);
        assert_eq!(normalized_depths(&[3, 2, 4]), vec![0, 0, 1]);
    }

    #[test]
    fn title_restatement_needs_exact_or_long_prefix() {
        assert!(restates_document_title(
            "The Sci-Fi Nuclear Core Battery Lamp",
            "The Sci-Fi Nuclear Core Battery Lamp : 13 Steps (with Pictures) - Instructables",
        ));
        assert!(restates_document_title("Same Title", "Same, Title!"));
        assert!(!restates_document_title("Intro", "Introduction to Systems"));
        assert!(!restates_document_title(
            "History",
            "Oyo Empire - Wikipedia"
        ));
    }
}
