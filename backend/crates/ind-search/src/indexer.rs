use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use ind_application::repos::document::DocumentRepository;
use ind_application::repos::search::SearchRepository;
use ind_application::{AppError, classify_search_language};
use ind_domain::{
    DocumentId, SearchDocument, SearchDocumentId, SearchDocumentKind, SearchDocumentSource,
    SearchIndexedHighlight,
};

use crate::metadata::{
    build_document_metadata_text, build_document_section_metadata_text, join_highlights,
};

pub(crate) const MAX_INDEXED_TEXT_BYTES: usize = 512 * 1024;
const MAX_TITLE_BYTES: usize = 16 * 1024;
const MAX_SECTION_TITLE_BYTES: usize = 8 * 1024;
const MAX_METADATA_BYTES: usize = 64 * 1024;
const MAX_HIGHLIGHT_BYTES: usize = 128 * 1024;

pub struct SearchIndexer {
    document_repo: Arc<dyn DocumentRepository>,
    content_provider: Arc<dyn ind_application::repos::prepared_content::PreparedContentProvider>,
    search_repo: Arc<dyn SearchRepository>,
}

impl SearchIndexer {
    pub fn new(
        document_repo: Arc<dyn DocumentRepository>,
        content_provider: Arc<
            dyn ind_application::repos::prepared_content::PreparedContentProvider,
        >,
        search_repo: Arc<dyn SearchRepository>,
    ) -> Self {
        Self {
            document_repo,
            content_provider,
            search_repo,
        }
    }

    pub async fn reindex_document(&self, document_id: DocumentId) -> Result<(), AppError> {
        let Some(document) = self.document_repo.find_by_id_global(document_id).await? else {
            self.search_repo
                .delete_search_documents_for_document(document_id)
                .await?;
            return Ok(());
        };

        let note = self.search_repo.get_document_note_text(document_id).await?;
        let highlights = self
            .search_repo
            .list_highlights_for_document(document_id)
            .await?;
        let (root_highlights, mut chapter_highlights_by_section) = split_highlights(&highlights);

        let metadata = build_document_metadata_text(&document, note.as_deref());
        let prepared = self.content_provider.load_for_document(document_id).await?;
        let root_body_text = match prepared.as_ref() {
            // Migrated/saved content resolves prepared content (incl. EPUB chapters) via the
            // id-bridge. A net-new feed-prepared document has no legacy item, so fall back to its
            // document-addressable readable_html asset; only then to the excerpt.
            Some(prepared) => prepared.root_text.clone(),
            None => self
                .content_provider
                .load_readable_text_for_document(document_id)
                .await?
                .unwrap_or_else(|| document.excerpt.clone().unwrap_or_default()),
        };
        let language_decision = classify_search_language(
            document.language.as_deref(),
            &[
                &document.title,
                document.excerpt.as_deref().unwrap_or_default(),
                &root_body_text,
            ],
        );
        if document.language.is_none()
            && let Some(language) = language_decision.language.as_deref()
        {
            self.document_repo
                .set_language_if_missing(document.user_id, document.id, language)
                .await?;
        }
        let search_config = language_decision.search_config.as_regconfig().to_string();

        let now = Utc::now();
        let mut documents = Vec::new();
        let mut projected_epub_sections = false;
        if let Some(prepared) = &prepared {
            for parent in &prepared.parents {
                if parent.kind != ind_domain::PreparedSectionKind::Chapter {
                    continue;
                }
                let chapter_highlights = chapter_highlights_by_section
                    .remove(&parent.key)
                    .unwrap_or_default();
                documents.push(cap_search_document(SearchDocument {
                    id: SearchDocumentId::new(),
                    source: SearchDocumentSource::Document { document_id },
                    user_id: document.user_id,
                    document_kind: SearchDocumentKind::EpubChapter,
                    section_key: parent.key.clone(),
                    section_title: parent.title.clone(),
                    title: document.title.clone(),
                    body_text: parent.text.clone(),
                    highlight_text: join_highlights(&chapter_highlights),
                    metadata_text: build_document_section_metadata_text(&document),
                    search_config: search_config.clone(),
                    saved_at: document.created_at,
                    updated_at: now,
                }));
                projected_epub_sections = true;
            }
        }

        let root_highlights = if projected_epub_sections {
            root_highlights
        } else {
            highlights
        };

        documents.insert(
            0,
            cap_search_document(SearchDocument {
                id: SearchDocumentId::new(),
                source: SearchDocumentSource::Document { document_id },
                user_id: document.user_id,
                document_kind: SearchDocumentKind::Item,
                section_key: String::new(),
                section_title: None,
                title: document.title.clone(),
                body_text: root_body_text,
                highlight_text: join_highlights(&root_highlights),
                metadata_text: metadata,
                search_config,
                saved_at: document.created_at,
                updated_at: now,
            }),
        );

        self.search_repo
            .replace_search_documents_for_document(document_id, &documents)
            .await?;

        Ok(())
    }
}

pub(crate) fn cap_search_document(mut document: SearchDocument) -> SearchDocument {
    let mut remaining = MAX_INDEXED_TEXT_BYTES;
    document.title = take_indexed_budget(&document.title, MAX_TITLE_BYTES, &mut remaining);
    document.section_title = document
        .section_title
        .as_deref()
        .map(|value| take_indexed_budget(value, MAX_SECTION_TITLE_BYTES, &mut remaining));
    document.metadata_text =
        take_indexed_budget(&document.metadata_text, MAX_METADATA_BYTES, &mut remaining);
    document.highlight_text = take_indexed_budget(
        &document.highlight_text,
        MAX_HIGHLIGHT_BYTES,
        &mut remaining,
    );
    document.body_text = take_indexed_budget(&document.body_text, remaining, &mut remaining);
    document
}

fn take_indexed_budget(value: &str, field_limit: usize, remaining: &mut usize) -> String {
    let limit = value.len().min(field_limit).min(*remaining);
    let end = value.floor_char_boundary(limit);
    *remaining -= end;
    value[..end].to_string()
}

/// Split indexed highlights into root-scoped (no section) and per-section (EPUB chapter) buckets.
fn split_highlights(
    highlights: &[SearchIndexedHighlight],
) -> (
    Vec<SearchIndexedHighlight>,
    HashMap<String, Vec<SearchIndexedHighlight>>,
) {
    let root_highlights: Vec<SearchIndexedHighlight> = highlights
        .iter()
        .filter(|highlight| highlight.section_key.is_none())
        .cloned()
        .collect();
    let mut by_section: HashMap<String, Vec<SearchIndexedHighlight>> = HashMap::new();
    for highlight in highlights {
        if let Some(section_key) = highlight.section_key.as_ref() {
            by_section
                .entry(section_key.clone())
                .or_default()
                .push(highlight.clone());
        }
    }
    (root_highlights, by_section)
}
