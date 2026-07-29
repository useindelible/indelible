use ind_application::AppError;
use ind_application::repos::highlight::HighlightRepository;
use ind_application::repos::prepared_content::PreparedContentProvider;
use ind_domain::{
    Document, DocumentType, Highlight, HighlightLocator, HighlightSourceLocator, ItemType,
    LibraryEntry, UserId,
};
use ind_ingest::AssetBackedPreparedContentProvider;
use ind_integrations::obsidian::{
    ObsidianExportSettings, ObsidianRenderDocument, ObsidianRenderHighlight,
    format_full_document_text,
};
use tracing::warn;

use crate::context::IntegrationJobDeps;

/// `DocumentType` and `ItemType` are identical 1:1 enums; the Obsidian render model still speaks
/// `ItemType` (renamed in a later phase), so map across at the boundary.
fn item_type_from_document_type(document_type: DocumentType) -> ItemType {
    match document_type {
        DocumentType::Article => ItemType::Article,
        DocumentType::Book => ItemType::Book,
        DocumentType::Email => ItemType::Email,
        DocumentType::Pdf => ItemType::Pdf,
        DocumentType::Tweet => ItemType::Tweet,
        DocumentType::Video => ItemType::Video,
        DocumentType::Podcast => ItemType::Podcast,
    }
}

pub(super) async fn build_render_document(
    ctx: &IntegrationJobDeps,
    highlight_repo: &dyn HighlightRepository,
    full_content_provider: Option<&AssetBackedPreparedContentProvider>,
    user_id: UserId,
    document: &Document,
    entry: &LibraryEntry,
    settings: &ObsidianExportSettings,
) -> Result<ObsidianRenderDocument, AppError> {
    let document_tags = ctx
        .tag_repo
        .list_by_library_entry(user_id, entry.id)
        .await?
        .into_iter()
        .map(|tag| tag.name)
        .collect();

    let mut highlights = highlight_repo
        .list_by_document(document.id, user_id)
        .await?;
    highlights.sort_by_key(|highlight| (highlight.created_at, highlight.id.into_uuid()));
    let highlight_ids = highlights.iter().map(|h| h.id).collect::<Vec<_>>();
    let tags_by_highlight = highlight_repo
        .list_tags_for_highlights(&highlight_ids, user_id)
        .await?;

    let mut rendered_highlights = Vec::with_capacity(highlights.len());
    for highlight in highlights {
        let note = highlight_repo
            .get_note(highlight.id, user_id)
            .await?
            .map(|note| note.body);
        let tags = tags_by_highlight
            .get(&highlight.id)
            .map(|tags| tags.iter().map(|tag| tag.name.clone()).collect())
            .unwrap_or_default();
        rendered_highlights.push(to_render_highlight(highlight, note, tags));
    }

    let full_document_text = if settings.export_all_reader_documents {
        match full_content_provider {
            Some(provider) => match provider.load_readable_text_for_document(document.id).await {
                Ok(Some(markdown)) if !markdown.trim().is_empty() => {
                    format_full_document_text(&document.title, &markdown)
                }
                Ok(_) => {
                    warn!(
                        document_id = %document.id,
                        "skipping Obsidian full Reader companion because no readable content asset is available"
                    );
                    None
                }
                Err(error) => {
                    warn!(
                        document_id = %document.id,
                        error = %error,
                        "skipping Obsidian full Reader companion after prepared-content load failed"
                    );
                    None
                }
            },
            None => {
                warn!(
                    document_id = %document.id,
                    "skipping Obsidian full Reader companion because prepared-content provider is unavailable"
                );
                None
            }
        }
    } else {
        None
    };

    let summary = ctx
        .export_summary_provider
        .summary_for_document(document.id, document.excerpt.as_deref())
        .await?;

    Ok(ObsidianRenderDocument {
        subject_id: entry.id.to_string(),
        subject_kind: ind_application::repos::export_subject::ExportSubjectKind::LibraryEntry
            .as_str()
            .to_string(),
        title: document.title.clone(),
        full_title: document.title.clone(),
        url: document
            .original_url
            .clone()
            .or_else(|| document.canonical_url.clone()),
        author: document.author.clone(),
        item_type: item_type_from_document_type(document.document_type),
        image_url: document.lead_image_url.clone(),
        summary,
        full_document_text,
        document_tags,
        highlights: rendered_highlights,
    })
}

fn to_render_highlight(
    highlight: Highlight,
    note: Option<String>,
    tags: Vec<String>,
) -> ObsidianRenderHighlight {
    let (location, location_url) = highlight_location(
        highlight.locator.as_ref(),
        highlight.source_locator.as_ref(),
    );
    ObsidianRenderHighlight {
        id: highlight.id.to_string(),
        text: highlight.text_content,
        note,
        color: highlight.color,
        tags,
        location,
        location_url,
        created_at: highlight.created_at,
    }
}

fn highlight_location(
    locator: Option<&HighlightLocator>,
    source_locator: Option<&HighlightSourceLocator>,
) -> (Option<String>, Option<String>) {
    if let Some(HighlightSourceLocator::WebPageDomRange { url, location, .. }) = source_locator {
        return (Some(location.clone()), Some(url.clone()));
    }

    match locator {
        Some(HighlightLocator::Html { start_offset, .. }) => {
            (Some(format!("Location {start_offset}")), None)
        }
        Some(HighlightLocator::Epub {
            chapter,
            start_offset,
            ..
        }) => (Some(format!("{chapter}: {start_offset}")), None),
        Some(HighlightLocator::Pdf { page, .. }) => (Some(format!("Page {page}")), None),
        None => (None, None),
    }
}
