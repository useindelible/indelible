use std::collections::HashMap;
use std::sync::Arc;

pub(crate) mod validation;

#[cfg(test)]
mod tests;

use chrono::Utc;
use futures::future::BoxFuture;
use ind_domain::{DomainError, Highlight, HighlightId, HighlightNote, Tag, UserId};

use crate::AppError;
use crate::event_intents::{highlight_deleted, highlight_noted, highlight_updated};
use crate::ports::HighlightOperations;
use crate::repos::event::MutationSideEffects;
use crate::repos::highlight::HighlightRepository;
use crate::repos::lifecycle_outbox::search_reindex_document_outbox;
use crate::repos::tag::TagRepository;

use validation::validate_color;

#[derive(Debug, Clone)]
pub struct HighlightWithNote {
    pub highlight: Highlight,
    pub note: Option<HighlightNote>,
    pub tags: Vec<Tag>,
}

pub struct HighlightService {
    highlight_repo: Arc<dyn HighlightRepository>,
    tag_repo: Arc<dyn TagRepository>,
}

impl HighlightService {
    pub fn new(
        highlight_repo: Arc<dyn HighlightRepository>,
        tag_repo: Arc<dyn TagRepository>,
    ) -> Self {
        Self {
            highlight_repo,
            tag_repo,
        }
    }

    pub async fn update_highlight_color(
        &self,
        user_id: UserId,
        id: HighlightId,
        color: String,
    ) -> Result<Highlight, AppError> {
        validate_color(&color)?;
        let highlight = self.require_highlight(user_id, id).await?;
        let document_id = highlight.document_id;
        let now = Utc::now();
        let effects = MutationSideEffects {
            events: vec![highlight_updated(user_id, document_id, id)],
            outbox: vec![search_reindex_document_outbox(document_id, now)],
        };
        self.highlight_repo
            .update_color(id, user_id, &color, effects)
            .await
    }

    pub async fn delete_highlight(&self, user_id: UserId, id: HighlightId) -> Result<(), AppError> {
        let highlight = self.require_highlight(user_id, id).await?;
        let document_id = highlight.document_id;
        let now = Utc::now();
        let effects = MutationSideEffects {
            events: vec![highlight_deleted(user_id, document_id, id)],
            outbox: vec![search_reindex_document_outbox(document_id, now)],
        };
        self.highlight_repo.delete(id, user_id, effects).await
    }

    pub async fn list_recent_highlights(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<HighlightWithNote>, AppError> {
        let highlights = self
            .highlight_repo
            .list_recent_by_user(user_id, limit)
            .await?;
        let highlight_ids: Vec<HighlightId> = highlights.iter().map(|hl| hl.id).collect();
        let mut tags_by_highlight: HashMap<HighlightId, Vec<Tag>> = self
            .highlight_repo
            .list_tags_for_highlights(&highlight_ids, user_id)
            .await?;
        let mut result = Vec::with_capacity(highlights.len());

        for hl in highlights {
            let note = self.highlight_repo.get_note(hl.id, user_id).await?;
            let tags = tags_by_highlight.remove(&hl.id).unwrap_or_default();
            result.push(HighlightWithNote {
                highlight: hl,
                note,
                tags,
            });
        }

        Ok(result)
    }

    pub async fn upsert_highlight_note(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
        body: String,
    ) -> Result<HighlightNote, AppError> {
        let highlight = self.require_highlight(user_id, highlight_id).await?;
        let document_id = highlight.document_id;
        let now = Utc::now();
        let effects = MutationSideEffects {
            events: vec![highlight_noted(user_id, document_id, highlight_id)],
            outbox: vec![search_reindex_document_outbox(document_id, now)],
        };
        self.highlight_repo
            .upsert_note(highlight_id, user_id, &body, effects)
            .await
    }

    pub async fn delete_highlight_note(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
    ) -> Result<(), AppError> {
        let highlight = self.require_highlight(user_id, highlight_id).await?;
        let document_id = highlight.document_id;
        let now = Utc::now();
        // No `highlight.note_deleted` exists in the webhook catalog; deleting a note still mutates
        // indexed highlight content, so reindex without emitting a domain event.
        let effects =
            MutationSideEffects::with_outbox(search_reindex_document_outbox(document_id, now));
        self.highlight_repo
            .delete_note(highlight_id, user_id, effects)
            .await
    }

    pub async fn list_highlight_tags(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
    ) -> Result<Vec<Tag>, AppError> {
        self.highlight_repo
            .get_by_id(highlight_id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "highlight",
                    id: highlight_id.to_string(),
                })
            })?;
        self.highlight_repo.list_tags(highlight_id, user_id).await
    }

    pub async fn set_highlight_tags(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
        tag_names: Vec<String>,
    ) -> Result<Vec<Tag>, AppError> {
        self.require_highlight(user_id, highlight_id).await?;

        let mut tag_ids = Vec::with_capacity(tag_names.len());
        for name in &tag_names {
            let tag = self.tag_repo.find_or_create_by_name(user_id, name).await?;
            tag_ids.push(tag.id);
        }

        self.tag_repo
            .replace_for_highlight(user_id, highlight_id, &tag_ids, MutationSideEffects::none())
            .await?;

        self.highlight_repo.list_tags(highlight_id, user_id).await
    }

    async fn require_highlight(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
    ) -> Result<Highlight, AppError> {
        self.highlight_repo
            .get_by_id(highlight_id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "highlight",
                    id: highlight_id.to_string(),
                })
            })
    }
}

impl HighlightOperations for HighlightService {
    fn update_highlight_color(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
        color: String,
    ) -> BoxFuture<'_, Result<Highlight, AppError>> {
        Box::pin(self.update_highlight_color(user_id, highlight_id, color))
    }

    fn delete_highlight(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.delete_highlight(user_id, highlight_id))
    }

    fn upsert_note(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
        body: String,
    ) -> BoxFuture<'_, Result<HighlightNote, AppError>> {
        Box::pin(self.upsert_highlight_note(user_id, highlight_id, body))
    }

    fn delete_note(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.delete_highlight_note(user_id, highlight_id))
    }

    fn list_recent_highlights(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> BoxFuture<'_, Result<Vec<HighlightWithNote>, AppError>> {
        Box::pin(self.list_recent_highlights(user_id, limit))
    }

    fn list_highlight_tags(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
    ) -> BoxFuture<'_, Result<Vec<Tag>, AppError>> {
        Box::pin(self.list_highlight_tags(user_id, highlight_id))
    }

    fn set_highlight_tags(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
        tag_names: Vec<String>,
    ) -> BoxFuture<'_, Result<Vec<Tag>, AppError>> {
        Box::pin(self.set_highlight_tags(user_id, highlight_id, tag_names))
    }
}
