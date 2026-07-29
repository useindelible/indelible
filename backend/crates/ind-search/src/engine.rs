use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ind_application::AppError;
use ind_application::repos::search::{RecentSearchRepository, SearchRepository};
use ind_domain::{
    DocumentId, ItemType, RecentSearch, SearchCursor, SearchEntityCard, SearchHit, SearchPage,
    SearchSuggestion, SearchSuggestionKind, UserId,
};

use crate::entity::{chip_matches_query, entity_filter_values, entity_type_label};
use crate::query::{
    FILTER_HINTS, build_fts_query, extract_last_token, normalize_query, parse_query,
};
use crate::rate_limit::SearchRateLimitDefaults;
use crate::suggesters::{HAS_VALUES, IS_VALUES, PINNED_VALUES, push_static, quote_if_needed};

pub const PUBLIC_SEARCH_QUERY_MAX_CHARS: usize = 512;

pub struct SearchEngine {
    search_repo: Arc<dyn SearchRepository>,
    recent_repo: Arc<dyn RecentSearchRepository>,
    defaults: SearchRateLimitDefaults,
}

impl SearchEngine {
    pub fn new(
        search_repo: Arc<dyn SearchRepository>,
        recent_repo: Arc<dyn RecentSearchRepository>,
        defaults: SearchRateLimitDefaults,
    ) -> Self {
        Self {
            search_repo,
            recent_repo,
            defaults,
        }
    }

    pub fn parse_query(&self, raw_query: &str) -> ind_domain::ParsedSearchQuery {
        parse_query(raw_query)
    }

    pub fn encode_cursor(cursor: &SearchCursor) -> Result<String, AppError> {
        serde_json::to_vec(cursor)
            .map(|json| URL_SAFE_NO_PAD.encode(json))
            .map_err(|err| AppError::Repository(Box::new(err)))
    }

    pub fn decode_cursor(value: &str) -> Result<SearchCursor, AppError> {
        let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|err| {
            AppError::Domain(ind_domain::DomainError::Validation {
                field: "cursor".into(),
                message: err.to_string(),
            })
        })?;
        serde_json::from_slice(&bytes).map_err(|err| {
            AppError::Domain(ind_domain::DomainError::Validation {
                field: "cursor".into(),
                message: err.to_string(),
            })
        })
    }

    pub async fn search(
        &self,
        user_id: UserId,
        raw_query: &str,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<SearchPage, AppError> {
        validate_public_query_length(raw_query)?;
        let parsed = parse_query(raw_query);
        if parsed.text_query.is_none() && parsed.filters.is_empty() {
            return Ok(SearchPage {
                query: raw_query.trim().to_string(),
                results: Vec::new(),
                next_cursor: None,
                has_more: false,
                entity_card: None,
            });
        }

        let cursor = match cursor {
            Some(value) if !value.trim().is_empty() => Some(Self::decode_cursor(value)?),
            _ => None,
        };

        let query = build_fts_query(user_id, &parsed, cursor.as_ref(), limit as i64 + 1);
        let mut rows = self.search_repo.search_fts(&query).await?;
        let has_more = rows.len() as u32 > limit;
        if has_more {
            rows.truncate(limit as usize);
        }

        let entity_card = self.resolve_entity_card(user_id, &parsed).await?;
        self.enrich_hits_with_entities(user_id, &parsed, entity_card.as_ref(), &mut rows)
            .await?;

        let next_cursor = if has_more {
            rows.last()
                .map(|hit| cursor_from_hit(hit, query.score_reference_at))
                .transpose()?
                .map(|cursor| Self::encode_cursor(&cursor))
                .transpose()?
        } else {
            None
        };

        let normalized = normalize_query(raw_query);
        if !normalized.is_empty() {
            let _ = self
                .recent_repo
                .upsert_recent_search(
                    user_id,
                    raw_query.trim(),
                    &normalized,
                    self.defaults.recent_search_limit,
                )
                .await?;
        }

        Ok(SearchPage {
            query: raw_query.trim().to_string(),
            results: rows,
            next_cursor,
            has_more,
            entity_card,
        })
    }

    pub async fn suggestions(
        &self,
        user_id: UserId,
        raw_query: &str,
        limit: u32,
    ) -> Result<Vec<SearchSuggestion>, AppError> {
        validate_public_query_length(raw_query)?;
        let trimmed = raw_query.trim();
        let limit = limit.max(1) as usize;
        let last_token = extract_last_token(raw_query);
        let lower_token = last_token.to_lowercase();
        let mut suggestions = Vec::new();

        if let Some(prefix) = lower_token.strip_prefix("tag:") {
            for name in self
                .search_repo
                .suggest_tags(user_id, prefix, limit as i64)
                .await?
            {
                suggestions.push(SearchSuggestion {
                    kind: SearchSuggestionKind::Tag,
                    label: format!("tag:{name}"),
                    insert_text: format!("tag:{name}"),
                    description: Some("Filter by tag".into()),
                });
            }
        } else if let Some(prefix) = lower_token.strip_prefix("collection:") {
            for name in self
                .search_repo
                .suggest_collections(user_id, prefix, limit as i64)
                .await?
            {
                suggestions.push(SearchSuggestion {
                    kind: SearchSuggestionKind::Collection,
                    label: format!("collection:{name}"),
                    insert_text: format!("collection:{name}"),
                    description: Some("Filter by collection".into()),
                });
            }
        } else if let Some(prefix) = lower_token.strip_prefix("entity:") {
            for entity in self
                .search_repo
                .suggest_entities(user_id, prefix, limit as i64)
                .await?
            {
                suggestions.push(SearchSuggestion {
                    kind: SearchSuggestionKind::Entity,
                    label: format!(
                        "{} ({})",
                        entity.name,
                        entity_type_label(entity.entity_type)
                    ),
                    insert_text: format!("entity:\"{}\"", entity.name),
                    description: Some("Filter by entity".into()),
                });
            }
        } else if let Some(prefix) = lower_token.strip_prefix("sender_domain:") {
            for value in self
                .search_repo
                .suggest_sender_domains(user_id, prefix, limit as i64)
                .await?
            {
                suggestions.push(SearchSuggestion {
                    kind: SearchSuggestionKind::Sender,
                    label: format!("sender_domain:{value}"),
                    insert_text: quote_if_needed("sender_domain", &value),
                    description: Some("Filter by sender domain".into()),
                });
            }
            suggestions.truncate(limit);
            return Ok(suggestions);
        } else if let Some(prefix) = lower_token.strip_prefix("sender:") {
            for value in self
                .search_repo
                .suggest_senders(user_id, prefix, limit as i64)
                .await?
            {
                suggestions.push(SearchSuggestion {
                    kind: SearchSuggestionKind::Sender,
                    label: format!("sender:{value}"),
                    insert_text: quote_if_needed("sender", &value),
                    description: Some("Filter by sender".into()),
                });
            }
            suggestions.truncate(limit);
            return Ok(suggestions);
        } else if let Some(prefix) = lower_token.strip_prefix("list:") {
            for value in self
                .search_repo
                .suggest_list_ids(user_id, prefix, limit as i64)
                .await?
            {
                suggestions.push(SearchSuggestion {
                    kind: SearchSuggestionKind::List,
                    label: format!("list:{value}"),
                    insert_text: quote_if_needed("list", &value),
                    description: Some("Filter by mailing list".into()),
                });
            }
            suggestions.truncate(limit);
            return Ok(suggestions);
        } else if let Some(prefix) = lower_token.strip_prefix("author:") {
            for value in self
                .search_repo
                .suggest_authors(user_id, prefix, limit as i64)
                .await?
            {
                suggestions.push(SearchSuggestion {
                    kind: SearchSuggestionKind::Author,
                    label: format!("author:{value}"),
                    insert_text: quote_if_needed("author", &value),
                    description: Some("Filter by author".into()),
                });
            }
            suggestions.truncate(limit);
            return Ok(suggestions);
        } else if let Some(prefix) = lower_token.strip_prefix("type:") {
            push_static(
                &mut suggestions,
                "type",
                ItemType::NAMES,
                prefix,
                "Filter by content type",
            );
            suggestions.truncate(limit);
            return Ok(suggestions);
        } else if let Some(prefix) = lower_token.strip_prefix("is:") {
            push_static(
                &mut suggestions,
                "is",
                IS_VALUES,
                prefix,
                "Filter by status",
            );
            suggestions.truncate(limit);
            return Ok(suggestions);
        } else if let Some(prefix) = lower_token.strip_prefix("has:") {
            push_static(
                &mut suggestions,
                "has",
                HAS_VALUES,
                prefix,
                "Filter by attribute",
            );
            suggestions.truncate(limit);
            return Ok(suggestions);
        } else if let Some(prefix) = lower_token.strip_prefix("pinned:") {
            push_static(
                &mut suggestions,
                "pinned",
                PINNED_VALUES,
                prefix,
                "Filter by pin state",
            );
            suggestions.truncate(limit);
            return Ok(suggestions);
        } else {
            for filter_name in FILTER_HINTS {
                if trimmed.is_empty() || filter_name.starts_with(&lower_token) {
                    suggestions.push(SearchSuggestion {
                        kind: SearchSuggestionKind::Filter,
                        label: (*filter_name).to_string(),
                        insert_text: (*filter_name).to_string(),
                        description: Some("Search filter".into()),
                    });
                }
            }

            let prefix = lower_token.trim_matches('"');
            // For recent searches and entity suggestions, match against the full
            // normalized query rather than just the last whitespace-delimited token.
            // This ensures that typing "Elon Musk" returns the "Elon Musk" recent
            // search and entity rather than trying to prefix-match on "Musk" alone.
            // Tags and collections are single-word identifiers, so last-token is correct for those.
            let full_prefix = normalize_query(trimmed);
            if !prefix.is_empty() {
                for query in self
                    .recent_repo
                    .suggest_recent_searches(user_id, &full_prefix, limit as i64)
                    .await?
                {
                    suggestions.push(SearchSuggestion {
                        kind: SearchSuggestionKind::Recent,
                        label: query.raw_query.clone(),
                        insert_text: query.raw_query,
                        description: Some("Recent search".into()),
                    });
                }

                for name in self
                    .search_repo
                    .suggest_tags(user_id, prefix, limit as i64)
                    .await?
                {
                    suggestions.push(SearchSuggestion {
                        kind: SearchSuggestionKind::Tag,
                        label: format!("tag:{name}"),
                        insert_text: format!("tag:{name}"),
                        description: Some("Filter by tag".into()),
                    });
                }

                for name in self
                    .search_repo
                    .suggest_collections(user_id, prefix, limit as i64)
                    .await?
                {
                    suggestions.push(SearchSuggestion {
                        kind: SearchSuggestionKind::Collection,
                        label: format!("collection:{name}"),
                        insert_text: format!("collection:{name}"),
                        description: Some("Filter by collection".into()),
                    });
                }

                for entity in self
                    .search_repo
                    .suggest_entities(user_id, &full_prefix, limit as i64)
                    .await?
                {
                    suggestions.push(SearchSuggestion {
                        kind: SearchSuggestionKind::Entity,
                        label: format!(
                            "{} ({})",
                            entity.name,
                            entity_type_label(entity.entity_type)
                        ),
                        insert_text: format!("entity:\"{}\"", entity.name),
                        description: Some("Filter by entity".into()),
                    });
                }
            }
        }

        suggestions.sort_by(|left, right| left.label.cmp(&right.label));
        suggestions.dedup_by(|left, right| left.insert_text == right.insert_text);
        suggestions.truncate(limit);
        Ok(suggestions)
    }

    pub async fn list_recent_searches(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<RecentSearch>, AppError> {
        self.recent_repo.list_recent_searches(user_id, limit).await
    }

    pub async fn clear_recent_searches(&self, user_id: UserId) -> Result<(), AppError> {
        self.recent_repo.clear_recent_searches(user_id).await
    }

    pub async fn delete_recent_search(
        &self,
        user_id: UserId,
        recent_search_id: ind_domain::RecentSearchId,
    ) -> Result<(), AppError> {
        self.recent_repo
            .delete_recent_search(user_id, recent_search_id)
            .await
    }

    async fn resolve_entity_card(
        &self,
        user_id: UserId,
        parsed: &ind_domain::ParsedSearchQuery,
    ) -> Result<Option<SearchEntityCard>, AppError> {
        if let [value] = entity_filter_values(parsed).as_slice() {
            return self.search_repo.find_entity_card(user_id, value).await;
        }

        let Some(text_query) = parsed.text_query.as_deref() else {
            return Ok(None);
        };

        if text_query.trim().len() < 3 {
            return Ok(None);
        }

        self.search_repo.find_entity_card(user_id, text_query).await
    }

    async fn enrich_hits_with_entities(
        &self,
        user_id: UserId,
        parsed: &ind_domain::ParsedSearchQuery,
        entity_card: Option<&SearchEntityCard>,
        hits: &mut [SearchHit],
    ) -> Result<(), AppError> {
        if hits.is_empty() {
            return Ok(());
        }

        let document_ids: Vec<DocumentId> = hits.iter().filter_map(|hit| hit.document_id).collect();
        if document_ids.is_empty() {
            return Ok(());
        }
        let chips_by_document = self
            .search_repo
            .list_entity_chips_for_documents(user_id, &document_ids)
            .await?;
        let entity_filters = entity_filter_values(parsed);
        let normalized_text_query = parsed.text_query.as_deref().map(normalize_query);

        for hit in hits {
            let Some(document_id) = hit.document_id else {
                continue;
            };
            let Some(mut chips) = chips_by_document.get(&document_id).cloned() else {
                continue;
            };

            chips.retain(|chip| {
                entity_card
                    .map(|card| chip.entity_id == card.entity_id)
                    .unwrap_or(false)
                    || chip_matches_query(chip, &entity_filters, normalized_text_query.as_deref())
            });
            chips.sort_by(|left, right| {
                right
                    .mention_count
                    .cmp(&left.mention_count)
                    .then_with(|| left.name.cmp(&right.name))
            });
            chips.truncate(5);
            hit.entity_chips = chips;
        }

        Ok(())
    }
}

fn validate_public_query_length(raw_query: &str) -> Result<(), AppError> {
    if raw_query.trim().chars().count() > PUBLIC_SEARCH_QUERY_MAX_CHARS {
        return Err(AppError::Domain(ind_domain::DomainError::Validation {
            field: "q".into(),
            message: format!("must be at most {PUBLIC_SEARCH_QUERY_MAX_CHARS} characters"),
        }));
    }

    Ok(())
}

fn cursor_from_hit(
    hit: &SearchHit,
    score_reference_at: chrono::DateTime<chrono::Utc>,
) -> Result<SearchCursor, AppError> {
    Ok(SearchCursor {
        score: hit.score,
        score_reference_at,
        saved_at: hit.saved_at,
        result_id: hit.result_id_uuid(),
        section_key: hit
            .section
            .as_ref()
            .map(|section| section.key.clone())
            .unwrap_or_default(),
    })
}
