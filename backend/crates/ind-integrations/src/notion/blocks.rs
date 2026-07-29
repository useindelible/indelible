use chrono::{DateTime, Utc};
use ind_domain::{HighlightId, NotionExportSettings};

pub const MAX_BLOCKS_PER_REQUEST: usize = 100;
pub const MAX_RICH_TEXT_CHARS: usize = 2000;
pub const MAX_PAYLOAD_BYTES: usize = 500 * 1024;
// ── Block types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum NotionBlock {
    Paragraph {
        text: String,
        href: Option<String>,
        highlight_cursor: Option<(DateTime<Utc>, HighlightId)>,
    },
    Callout {
        text: String,
        highlight_cursor: Option<(DateTime<Utc>, HighlightId)>,
    },
    Divider,
}

impl NotionBlock {
    pub fn highlight_cursor(&self) -> Option<(DateTime<Utc>, HighlightId)> {
        match self {
            Self::Paragraph {
                highlight_cursor, ..
            }
            | Self::Callout {
                highlight_cursor, ..
            } => *highlight_cursor,
            Self::Divider => None,
        }
    }
}

pub fn notion_block_to_json(block: &NotionBlock) -> serde_json::Value {
    match block {
        NotionBlock::Paragraph { text, href, .. } => serde_json::json!({
            "object": "block",
            "type": "paragraph",
            "paragraph": {"rich_text": [rich_text_json(text, href.as_deref())]}
        }),
        NotionBlock::Callout { text, .. } => serde_json::json!({
            "object": "block",
            "type": "callout",
            "callout": {
                "rich_text": [{"type": "text", "text": {"content": text}}]
            }
        }),
        NotionBlock::Divider => serde_json::json!({
            "object": "block",
            "type": "divider",
            "divider": {}
        }),
    }
}

fn rich_text_json(text: &str, href: Option<&str>) -> serde_json::Value {
    match href.filter(|s| !s.trim().is_empty()) {
        Some(url) => serde_json::json!({
            "type": "text",
            "text": {"content": text, "link": {"url": url}}
        }),
        None => serde_json::json!({"type": "text", "text": {"content": text}}),
    }
}

// ── Highlight input ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HighlightText {
    pub id: HighlightId,
    pub created_at: DateTime<Utc>,
    pub text: String,
    pub note: Option<String>,
    pub tags: Vec<String>,
    pub location: Option<HighlightLocation>,
}

#[derive(Debug, Clone)]
pub struct HighlightLocation {
    pub label: String,
    pub href: Option<String>,
}

// ── Block building ───────────────────────────────────────────────────────────

pub fn build_highlight_blocks(highlights: &[HighlightText]) -> Vec<NotionBlock> {
    build_highlight_blocks_with_options(highlights, &NotionExportSettings::default())
}

pub fn build_highlight_blocks_with_options(
    highlights: &[HighlightText],
    settings: &NotionExportSettings,
) -> Vec<NotionBlock> {
    if highlights.is_empty() {
        return Vec::new();
    }
    let mut blocks = Vec::with_capacity(highlights.len() * 2 + 1);
    for (idx, h) in highlights.iter().enumerate() {
        let cursor = Some((h.created_at, h.id));
        let location = settings
            .include_highlight_locations
            .then_some(h.location.as_ref())
            .flatten();

        if !settings.compact_layout {
            blocks.push(NotionBlock::Divider);
            if let Some(location) = location {
                blocks.push(NotionBlock::Paragraph {
                    text: truncate_notion_text(&format!("Location: {}", location.label)),
                    href: location.href.clone(),
                    highlight_cursor: None,
                });
            }
        } else if idx > 0 {
            // Compact mode intentionally omits dividers.
        }

        let text = if settings.compact_layout {
            match location {
                Some(location) => format!("{} ({})", h.text, location.label),
                None => h.text.clone(),
            }
        } else {
            h.text.clone()
        };
        blocks.push(NotionBlock::Paragraph {
            text: truncate_notion_text(&text),
            href: None,
            highlight_cursor: cursor,
        });
        if let Some(note) = &h.note {
            blocks.push(NotionBlock::Callout {
                text: truncate_notion_text(&format!("Note: {note}")),
                highlight_cursor: cursor,
            });
        }
        if !h.tags.is_empty() {
            blocks.push(NotionBlock::Callout {
                text: truncate_notion_text(&format!("Tags: {}", h.tags.join(", "))),
                highlight_cursor: cursor,
            });
        }
    }
    blocks
}

pub fn chunk_blocks_for_request(blocks: &[NotionBlock]) -> Vec<Vec<NotionBlock>> {
    let mut chunks: Vec<Vec<NotionBlock>> = Vec::new();
    let mut current: Vec<NotionBlock> = Vec::new();
    for block in blocks {
        let mut tentative = current.clone();
        tentative.push(block.clone());
        let body = serde_json::json!({
            "children": tentative.iter().map(notion_block_to_json).collect::<Vec<_>>()
        });
        let too_many = tentative.len() > MAX_BLOCKS_PER_REQUEST;
        let too_large = serde_json::to_vec(&body)
            .map(|b| b.len())
            .unwrap_or(MAX_PAYLOAD_BYTES + 1)
            > MAX_PAYLOAD_BYTES;
        if !current.is_empty() && (too_many || too_large) {
            chunks.push(std::mem::take(&mut current));
        }
        current.push(block.clone());
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

pub(super) fn truncate_notion_text(text: &str) -> String {
    text.chars().take(MAX_RICH_TEXT_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn highlight() -> HighlightText {
        HighlightText {
            id: HighlightId::new(),
            created_at: Utc::now(),
            text: "quoted text".into(),
            note: Some("reader note".into()),
            tags: vec!["rust".into(), "systems".into()],
            location: Some(HighlightLocation {
                label: "Page 7".into(),
                href: Some("https://example.com/page/7".into()),
            }),
        }
    }

    #[test]
    fn highlight_layouts_preserve_cursor_location_note_and_tags() {
        let expanded = build_highlight_blocks_with_options(
            &[highlight()],
            &NotionExportSettings {
                compact_layout: false,
                include_highlight_locations: true,
                ..NotionExportSettings::default()
            },
        );
        assert_eq!(expanded.len(), 5);
        assert!(expanded[0].highlight_cursor().is_none());
        assert!(
            expanded[2..]
                .iter()
                .all(|block| block.highlight_cursor().is_some())
        );
        assert_eq!(
            notion_block_to_json(&expanded[1])["paragraph"]["rich_text"][0]["text"]["link"]["url"],
            "https://example.com/page/7"
        );

        let compact = build_highlight_blocks_with_options(
            &[highlight()],
            &NotionExportSettings {
                compact_layout: true,
                include_highlight_locations: true,
                ..NotionExportSettings::default()
            },
        );
        assert_eq!(compact.len(), 3);
        assert!(
            notion_block_to_json(&compact[0])
                .to_string()
                .contains("Page 7")
        );
    }

    #[test]
    fn block_chunking_enforces_count_and_unicode_text_limits() {
        let blocks = vec![NotionBlock::Divider; MAX_BLOCKS_PER_REQUEST + 1];
        let chunks = chunk_blocks_for_request(&blocks);
        assert_eq!(chunks.iter().map(Vec::len).collect::<Vec<_>>(), [100, 1]);
        let truncated = truncate_notion_text(&"é".repeat(MAX_RICH_TEXT_CHARS + 1));
        assert_eq!(truncated.chars().count(), MAX_RICH_TEXT_CHARS);
    }
}
