use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use ind_domain::{
    ArchiveAssetKind, ArchiveAssetStatus, Document, DocumentId, DomainError, TtsChunkHint,
    TtsElementFeed, TtsElementKind, TtsElementSource, TtsSpokenElement, UserId,
};

use crate::AppError;
use crate::ports::HtmlExtractor;
use crate::repos::document::DocumentRepository;
use crate::repos::document_asset::DocumentAssetRepository;
use crate::storage::ObjectStorage;

pub const TARGET_CHUNK_CHARS: usize = 6_000;
pub const MAX_CHUNK_CHARS: usize = 9_000;
pub const NO_TIMESTAMP_TARGET_CHUNK_CHARS: usize = 1_200;
pub const NO_TIMESTAMP_MAX_CHUNK_CHARS: usize = 1_800;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtsContentUnavailableReason {
    ReadableContentUnavailable,
    ReadableContentEmpty,
    ReadableContentUnreadable,
}

impl TtsContentUnavailableReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadableContentUnavailable => "readable_content_unavailable",
            Self::ReadableContentEmpty => "readable_content_empty",
            Self::ReadableContentUnreadable => "readable_content_unreadable",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReadableTtsMetadata {
    Available(TtsElementFeed),
    Unavailable(TtsContentUnavailableReason),
}

pub struct ReadableTtsElementSource {
    document_repo: Arc<dyn DocumentRepository>,
    document_asset_repo: Arc<dyn DocumentAssetRepository>,
    storage: Arc<dyn ObjectStorage>,
    planner: ReadableHtmlTtsPlanner,
}

impl ReadableTtsElementSource {
    pub fn new(
        document_repo: Arc<dyn DocumentRepository>,
        document_asset_repo: Arc<dyn DocumentAssetRepository>,
        storage: Arc<dyn ObjectStorage>,
        html_extractor: Arc<dyn HtmlExtractor>,
    ) -> Self {
        Self {
            document_repo,
            document_asset_repo,
            storage,
            planner: ReadableHtmlTtsPlanner::new(html_extractor),
        }
    }

    pub async fn metadata(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<ReadableTtsMetadata, AppError> {
        let document = self.resolve_document(user_id, document_id).await?;

        let Some(asset) = self
            .document_asset_repo
            .find_by_document_and_kind(document.id, ArchiveAssetKind::ReadableHtml)
            .await?
        else {
            return Ok(ReadableTtsMetadata::Unavailable(
                TtsContentUnavailableReason::ReadableContentUnavailable,
            ));
        };

        if asset.status != ArchiveAssetStatus::Completed
            || asset.content_type != "text/html"
            || asset.s3_key.trim().is_empty()
        {
            return Ok(ReadableTtsMetadata::Unavailable(
                TtsContentUnavailableReason::ReadableContentUnavailable,
            ));
        }

        let html = match load_text_asset(self.storage.as_ref(), &asset.s3_key).await {
            Ok(html) => html,
            Err(_) => {
                return Ok(ReadableTtsMetadata::Unavailable(
                    TtsContentUnavailableReason::ReadableContentUnreadable,
                ));
            }
        };

        match self.planner.plan(document_id, document.title, &html) {
            Some(feed) => Ok(ReadableTtsMetadata::Available(feed)),
            None => Ok(ReadableTtsMetadata::Unavailable(
                TtsContentUnavailableReason::ReadableContentEmpty,
            )),
        }
    }

    async fn resolve_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Document, AppError> {
        self.document_repo
            .find_by_id(user_id, document_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "Document",
                    id: document_id.to_string(),
                })
            })
    }
}

#[async_trait]
impl TtsElementSource for ReadableTtsElementSource {
    async fn elements(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<TtsElementFeed, DomainError> {
        match self.metadata(user_id, document_id).await {
            Ok(ReadableTtsMetadata::Available(feed)) => Ok(feed),
            Ok(ReadableTtsMetadata::Unavailable(reason)) => Err(DomainError::Validation {
                field: "document_id".into(),
                message: reason.as_str().to_string(),
            }),
            Err(AppError::Domain(err)) => Err(err),
            Err(_) => Err(DomainError::Validation {
                field: "document_id".into(),
                message: TtsContentUnavailableReason::ReadableContentUnreadable
                    .as_str()
                    .to_string(),
            }),
        }
    }
}

pub struct ReadableHtmlTtsPlanner {
    html_extractor: Arc<dyn HtmlExtractor>,
}

impl ReadableHtmlTtsPlanner {
    pub fn new(html_extractor: Arc<dyn HtmlExtractor>) -> Self {
        Self { html_extractor }
    }
}

#[derive(Debug, Clone)]
struct PlannedElement {
    element_index: i32,
    kind: TtsElementKind,
    text: String,
    char_start: i32,
    char_end: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct TtsChunkingProfile {
    target_chars: usize,
    max_chars: usize,
    use_heading_sections: bool,
}

impl TtsChunkingProfile {
    pub fn timestamped_provider() -> Self {
        Self {
            target_chars: TARGET_CHUNK_CHARS,
            max_chars: MAX_CHUNK_CHARS,
            use_heading_sections: true,
        }
    }

    pub fn duration_only_provider() -> Self {
        Self {
            target_chars: NO_TIMESTAMP_TARGET_CHUNK_CHARS,
            max_chars: NO_TIMESTAMP_MAX_CHUNK_CHARS,
            use_heading_sections: false,
        }
    }
}

pub fn replan_feed_chunks(mut feed: TtsElementFeed, profile: TtsChunkingProfile) -> TtsElementFeed {
    let elements = feed
        .elements
        .iter()
        .map(|element| PlannedElement {
            element_index: element.element_index,
            kind: element.kind,
            text: element.text.clone(),
            char_start: element.char_start,
            char_end: element.char_end,
        })
        .collect::<Vec<_>>();

    let chunks = plan_chunks_with_profile(&elements, profile);
    for element in &mut feed.elements {
        if let Some(chunk) = chunks.iter().find(|chunk| {
            chunk.start_element_index <= element.element_index
                && element.element_index <= chunk.end_element_index
        }) {
            element.chunk_id = chunk.chunk_id.clone();
        }
    }
    feed.chunk_hints = chunks;
    feed
}

impl ReadableHtmlTtsPlanner {
    pub fn plan(
        &self,
        document_id: DocumentId,
        title: String,
        html: &str,
    ) -> Option<TtsElementFeed> {
        let mut elements = extract_elements(self.html_extractor.as_ref(), html);
        if elements.is_empty() {
            return None;
        }

        let chunks = plan_chunks(&elements);
        if chunks.is_empty() {
            return None;
        }

        let mut spoken = Vec::with_capacity(elements.len());
        for chunk in &chunks {
            for index in chunk.start_element_index..=chunk.end_element_index {
                if let Some(element) = elements.get_mut(index as usize) {
                    spoken.push(TtsSpokenElement {
                        element_index: element.element_index,
                        kind: element.kind,
                        text: element.text.clone(),
                        char_start: element.char_start,
                        char_end: element.char_end,
                        chunk_id: chunk.chunk_id.clone(),
                    });
                }
            }
        }

        Some(TtsElementFeed {
            document_id,
            title,
            chunk_hints: chunks,
            elements: spoken,
        })
    }
}

fn extract_elements(html_extractor: &dyn HtmlExtractor, html: &str) -> Vec<PlannedElement> {
    let spoken = html_extractor.extract_spoken_elements(html);
    let mut elements = Vec::with_capacity(spoken.len());
    let mut next_start = 0_i32;

    for entry in spoken {
        if entry.text.is_empty() {
            continue;
        }
        let text = entry.text;
        let char_start = next_start;
        let char_end = char_start + text.chars().count() as i32;
        next_start = char_end + 1;

        elements.push(PlannedElement {
            element_index: elements.len() as i32,
            kind: kind_for_tag(&entry.tag),
            text,
            char_start,
            char_end,
        });
    }

    elements
}

fn plan_chunks(elements: &[PlannedElement]) -> Vec<TtsChunkHint> {
    plan_chunks_with_profile(elements, TtsChunkingProfile::timestamped_provider())
}

fn plan_chunks_with_profile(
    elements: &[PlannedElement],
    profile: TtsChunkingProfile,
) -> Vec<TtsChunkHint> {
    if elements.is_empty() {
        return Vec::new();
    }

    let total_chars = range_chars(elements);
    let heading_count = elements
        .iter()
        .filter(|e| e.kind == TtsElementKind::Heading)
        .count();

    if total_chars <= profile.target_chars && (!profile.use_heading_sections || heading_count <= 1)
    {
        return vec![TtsChunkHint {
            chunk_id: "single-chunk-full-content".into(),
            start_element_index: 0,
            end_element_index: elements.len() as i32 - 1,
        }];
    }

    if profile.use_heading_sections && heading_count >= 2 {
        return plan_heading_chunks(elements, profile);
    }

    plan_window_chunks(elements, profile)
}

fn plan_heading_chunks(
    elements: &[PlannedElement],
    profile: TtsChunkingProfile,
) -> Vec<TtsChunkHint> {
    let mut ranges = Vec::<(i32, i32)>::new();
    let mut start = 0_i32;

    for element in elements.iter().skip(1) {
        if element.kind == TtsElementKind::Heading {
            ranges.push((start, element.element_index - 1));
            start = element.element_index;
        }
    }
    ranges.push((start, elements.len() as i32 - 1));

    let mut chunks = Vec::new();
    for (start, end) in ranges {
        push_window_chunks_for_range(elements, start, end, profile, &mut chunks);
    }
    chunks
}

fn plan_window_chunks(
    elements: &[PlannedElement],
    profile: TtsChunkingProfile,
) -> Vec<TtsChunkHint> {
    let mut chunks = Vec::new();
    push_window_chunks_for_slice(elements, profile, &mut chunks);
    chunks
}

fn push_window_chunks_for_range(
    elements: &[PlannedElement],
    start_index: i32,
    end_index: i32,
    profile: TtsChunkingProfile,
    chunks: &mut Vec<TtsChunkHint>,
) {
    let range = elements
        .iter()
        .filter(|element| {
            element.element_index >= start_index && element.element_index <= end_index
        })
        .cloned()
        .collect::<Vec<_>>();
    push_window_chunks_for_slice(&range, profile, chunks);
}

fn push_window_chunks_for_slice(
    elements: &[PlannedElement],
    profile: TtsChunkingProfile,
    chunks: &mut Vec<TtsChunkHint>,
) {
    if elements.is_empty() {
        return;
    }

    let mut start = 0_usize;
    let mut current_chars = 0_usize;

    for (idx, element) in elements.iter().enumerate() {
        let element_chars = element.text.chars().count();
        let candidate_chars = if current_chars == 0 {
            element_chars
        } else {
            current_chars + 1 + element_chars
        };
        if idx > start
            && (candidate_chars > profile.max_chars || current_chars >= profile.target_chars)
        {
            chunks.push(TtsChunkHint {
                chunk_id: format!("section_{:03}", chunks.len() + 1),
                start_element_index: elements[start].element_index,
                end_element_index: elements[idx - 1].element_index,
            });
            start = idx;
            current_chars = 0;
        }

        current_chars = if current_chars == 0 {
            element_chars
        } else {
            current_chars + 1 + element_chars
        };
    }

    #[expect(
        clippy::expect_used,
        reason = "the function returns early when elements is empty, so by here elements has a last item"
    )]
    let last_element = elements.last().expect("non-empty elements");
    chunks.push(TtsChunkHint {
        chunk_id: format!("section_{:03}", chunks.len() + 1),
        start_element_index: elements[start].element_index,
        end_element_index: last_element.element_index,
    });
}

fn range_chars(elements: &[PlannedElement]) -> usize {
    elements
        .iter()
        .map(|element| element.text.chars().count())
        .sum::<usize>()
        + elements.len().saturating_sub(1)
}

fn kind_for_tag(tag: &str) -> TtsElementKind {
    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => TtsElementKind::Heading,
        "blockquote" => TtsElementKind::Blockquote,
        "li" => TtsElementKind::ListItem,
        "figcaption" | "caption" => TtsElementKind::Caption,
        _ => TtsElementKind::Paragraph,
    }
}

async fn load_text_asset(storage: &dyn ObjectStorage, key: &str) -> Result<String, AppError> {
    let object = storage.get_object(key).await?;
    let mut stream = object.body;
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|err| AppError::Repository(Box::new(err)))?;
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|err| AppError::Repository(Box::new(err)))
}
