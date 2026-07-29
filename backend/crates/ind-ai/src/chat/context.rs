use std::collections::{HashMap, HashSet};

use ind_application::AppError;
use ind_domain::{DocumentId, PreparedContentParent, PreparedItemContent, SearchHit};

use crate::prompt::RetrievedPassage;

use super::MilaChatService;

impl MilaChatService {
    pub(super) async fn passages_from_hits_with_parent_context(
        &self,
        hits: &[SearchHit],
    ) -> Result<Vec<RetrievedPassage>, AppError> {
        let mut prepared_by_document = HashMap::<DocumentId, Option<PreparedItemContent>>::new();
        let mut source_index_by_document = HashMap::<DocumentId, usize>::new();
        let mut expanded_parents = HashSet::<(DocumentId, String)>::new();
        let mut passages = Vec::with_capacity(hits.len());

        for hit in hits {
            // Group passages by document so every source label maps to one saved item.
            let Some(document_id) = hit.document_id else {
                continue;
            };
            let next_source_index = source_index_by_document.len() + 1;
            let source_index = *source_index_by_document
                .entry(document_id)
                .or_insert(next_source_index);
            let source_label = format!("S{source_index}");

            let section_key = hit
                .section
                .as_ref()
                .map(|section| section.key.as_str())
                .filter(|key| !key.is_empty());
            if section_key.is_none() {
                passages.push(RetrievedPassage {
                    source_label,
                    title: hit.title.clone(),
                    snippet: hit.snippet.clone(),
                    child_excerpt: None,
                    section_title: hit
                        .section
                        .as_ref()
                        .and_then(|section| section.title.clone()),
                    url: hit.url.clone(),
                });
                continue;
            }

            // Parent context resolves via the id-reuse bridge (load_for_document); net-new
            // feed-prepared documents have no structured parents, so the snippet stands alone.
            let prepared = match prepared_by_document.get(&document_id) {
                Some(prepared) => prepared.as_ref(),
                None => {
                    let loaded = self.content_provider.load_for_document(document_id).await?;
                    prepared_by_document.insert(document_id, loaded);
                    prepared_by_document
                        .get(&document_id)
                        .and_then(|prepared| prepared.as_ref())
                }
            };

            let parent = prepared.and_then(|prepared| parent_for_hit(prepared, document_id, hit));
            if let Some(section_key) = section_key
                && parent.is_some()
                && !expanded_parents.insert((document_id, section_key.to_owned()))
            {
                continue;
            }
            let section_title = parent.and_then(|parent| parent.title.clone()).or_else(|| {
                hit.section
                    .as_ref()
                    .and_then(|section| section.title.clone())
            });
            let snippet = parent
                .map(|parent| parent.text.clone())
                .unwrap_or_else(|| hit.snippet.clone());
            let child_excerpt = parent
                .filter(|parent| !parent.text.contains(&hit.snippet))
                .map(|_| hit.snippet.clone());

            passages.push(RetrievedPassage {
                source_label,
                title: hit.title.clone(),
                snippet,
                child_excerpt,
                section_title,
                url: hit.url.clone(),
            });
        }

        Ok(passages)
    }
}

fn parent_for_hit<'a>(
    prepared: &'a PreparedItemContent,
    document_id: DocumentId,
    hit: &SearchHit,
) -> Option<&'a PreparedContentParent> {
    if document_id != prepared.document_id {
        return None;
    }

    let section_key = hit
        .section
        .as_ref()
        .map(|section| section.key.as_str())
        .filter(|key| !key.is_empty())?;
    prepared
        .parents
        .iter()
        .find(|parent| parent.key == section_key)
}
