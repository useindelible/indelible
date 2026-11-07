use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocEntry {
    pub id: String,
    pub title: String,
    pub depth: u32,
    pub spine_index: usize,
    /// Stable manifest/spine chapter ID (same for all subsections within one file).
    #[serde(default)]
    pub chapter_id: String,
    /// Fragment anchor within the chapter HTML (e.g. "deserved-respect"). None for top-level entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fragment: Option<String>,
    pub word_count: u32,
    pub start_page: u32,
    /// Basename of the spine item's manifest href (e.g. "notes.xhtml"). Used by the reader to
    /// resolve intra-EPUB cross-chapter link targets without re-fetching the OPF.
    #[serde(default)]
    pub spine_href: String,
}

#[derive(Debug, Clone)]
pub struct EpubChapter {
    pub id: String,
    pub title: String,
    pub html: String,
    pub word_count: u32,
    pub spine_index: usize,
}

#[derive(Debug, Clone)]
pub struct ProcessedEpub {
    pub toc: Vec<EpubTocEntry>,
    pub chapters: Vec<EpubChapter>,
    pub metadata: EpubMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub isbn: Option<String>,
    pub total_chapters: usize,
    pub total_words: u32,
    pub estimated_pages: u32,
    /// Every stored chapter by spine index, gaps included; None for books processed before it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapters: Option<Vec<EpubStoredChapter>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpubStoredChapter {
    pub spine_index: usize,
    /// Basename of the spine item's manifest href (e.g. "ch02.xhtml").
    pub href: String,
    /// Title of the first TOC entry pointing at this spine item; empty when none does.
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocResponse {
    pub metadata: EpubMetadata,
    pub toc: Vec<EpubTocEntry>,
}

#[derive(Debug, thiserror::Error)]
pub enum EpubError {
    #[error("invalid EPUB: {0}")]
    Invalid(String),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub(super) struct ManifestItem {
    pub(super) id: String,
    pub(super) href: String,
    pub(super) media_type: String,
}

pub(super) struct SpineEntry {
    pub(super) idref: String,
}

pub(super) struct ParsedOpf {
    pub(super) manifest: Vec<ManifestItem>,
    pub(super) spine: Vec<SpineEntry>,
    pub(super) title: Option<String>,
    pub(super) author: Option<String>,
    pub(super) publisher: Option<String>,
    pub(super) language: Option<String>,
    pub(super) isbn: Option<String>,
}

pub(super) struct NavPoint {
    pub(super) label: String,
    pub(super) content_src: String,
    pub(super) depth: u32,
}
