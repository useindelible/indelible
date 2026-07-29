use serde::{Deserialize, Serialize};

use crate::{DocumentId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreparedContentKind {
    ReadableHtml,
    Epub,
    Pdf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreparedSectionKind {
    Item,
    Chapter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedContentLocator {
    pub chapter_index: Option<i32>,
    pub page_number: Option<i32>,
    pub spine_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedContentParent {
    pub kind: PreparedSectionKind,
    pub key: String,
    pub title: Option<String>,
    pub ordinal: i32,
    pub text: String,
    pub locator: Option<PreparedContentLocator>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedContentLeaf {
    pub parent_key: String,
    pub kind: PreparedSectionKind,
    pub key: String,
    pub ordinal: i32,
    pub text: String,
    pub locator: Option<PreparedContentLocator>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedItemContent {
    pub document_id: DocumentId,
    pub user_id: UserId,
    pub source_kind: PreparedContentKind,
    pub title: String,
    pub root_text: String,
    pub parents: Vec<PreparedContentParent>,
    pub leaves: Vec<PreparedContentLeaf>,
}

impl PreparedItemContent {
    pub fn enriched_leaf_text(&self, leaf: &PreparedContentLeaf) -> String {
        let parent_label = self
            .parents
            .iter()
            .find(|p| p.key == leaf.parent_key)
            .and_then(|p| p.title.as_deref())
            .unwrap_or(&leaf.parent_key);

        let mut result = format!("Title: {}", self.title);

        if !parent_label.is_empty() {
            result.push_str(&format!("\nSection: {parent_label}"));
        }

        if let Some(locator) = &leaf.locator {
            let mut parts = Vec::new();
            if let Some(ch) = locator.chapter_index {
                parts.push(format!("chapter {ch}"));
            }
            if let Some(pg) = locator.page_number {
                parts.push(format!("page {pg}"));
            }
            if !parts.is_empty() {
                result.push_str(&format!("\nLocator: {}", parts.join(", ")));
            }
        }

        result.push_str(&format!("\n\n{}", leaf.text));
        result
    }
}
