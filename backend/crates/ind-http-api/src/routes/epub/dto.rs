use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct EpubTocResponse {
    pub metadata: EpubMetadata,
    pub toc: Vec<EpubTocEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct EpubMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub isbn: Option<String>,
    pub total_chapters: usize,
    pub total_words: u32,
    pub estimated_pages: u32,
    /// Every chapter `/epub/chapters/{index}` serves, by spine index; absent on older books.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapters: Option<Vec<EpubStoredChapter>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct EpubStoredChapter {
    pub spine_index: usize,
    /// Basename of the spine item's href, the target of intra-book links (e.g. `ch02.xhtml`).
    pub href: String,
    /// Title of the first TOC entry for this spine item; empty when the TOC omits the chapter.
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct EpubTocEntry {
    pub id: String,
    pub title: String,
    pub depth: u32,
    pub spine_index: usize,
    #[serde(default)]
    pub chapter_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fragment: Option<String>,
    pub word_count: u32,
    pub start_page: u32,
    #[serde(default)]
    pub spine_href: String,
}
