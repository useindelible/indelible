use ind_application::repos::epub_format::EpubChapterListState;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EpubTocResponse {
    pub metadata: EpubMetadata,
    pub toc: Vec<EpubTocEntry>,
    pub chapter_list: EpubChapterList,
}

/// Whether `metadata.chapters` is present, and what brings it when it is not: `pending` waits
/// for the server's backfill, `needs_reprocess` for a reprocess, `needs_reupload` for a new upload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EpubChapterList {
    Ready,
    Pending,
    NeedsReprocess,
    NeedsReupload,
}

impl From<EpubChapterListState> for EpubChapterList {
    fn from(state: EpubChapterListState) -> Self {
        match state {
            EpubChapterListState::Ready => Self::Ready,
            EpubChapterListState::Pending => Self::Pending,
            EpubChapterListState::NeedsReprocess => Self::NeedsReprocess,
            EpubChapterListState::NeedsReupload => Self::NeedsReupload,
        }
    }
}

/// The TOC as stored in object storage.
#[derive(Debug, Deserialize)]
pub struct StoredEpubToc {
    pub metadata: EpubMetadata,
    pub toc: Vec<EpubTocEntry>,
}

impl EpubTocResponse {
    pub fn from_stored(stored: StoredEpubToc, chapter_list: EpubChapterListState) -> Self {
        Self {
            metadata: stored.metadata,
            toc: stored.toc,
            chapter_list: chapter_list.into(),
        }
    }
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
