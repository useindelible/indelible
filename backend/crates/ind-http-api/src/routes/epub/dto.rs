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
