use chrono::{Duration, NaiveDate, Utc};
use ind_application::repos::search::SearchFtsQuery;
use ind_domain::{
    CanonicalAddress, SearchCursor, SearchFilter, SearchHasFilter, SearchSourceFilter,
    SearchStatusFilter, UserId,
};

pub(crate) const FILTER_HINTS: &[&str] = &[
    "tag:",
    "collection:",
    "type:",
    "author:",
    "before:",
    "after:",
    "is:",
    "has:",
    "url:",
    "entity:",
    "pinned:",
    "sender:",
    "sender_domain:",
    "list:",
    "subject:",
];

pub(crate) fn build_fts_query(
    user_id: UserId,
    parsed: &ind_domain::ParsedSearchQuery,
    cursor: Option<&SearchCursor>,
    limit: i64,
) -> SearchFtsQuery {
    let mut query = SearchFtsQuery {
        user_id,
        text_query: parsed.text_query.clone(),
        tag_values: Vec::new(),
        negated_tag_values: Vec::new(),
        collection_values: Vec::new(),
        negated_collection_values: Vec::new(),
        type_values: Vec::new(),
        negated_type_values: Vec::new(),
        author_values: Vec::new(),
        negated_author_values: Vec::new(),
        url_values: Vec::new(),
        negated_url_values: Vec::new(),
        entity_values: Vec::new(),
        negated_entity_values: Vec::new(),
        sender_values: Vec::new(),
        negated_sender_values: Vec::new(),
        sender_domain_values: Vec::new(),
        negated_sender_domain_values: Vec::new(),
        list_values: Vec::new(),
        negated_list_values: Vec::new(),
        subject_values: Vec::new(),
        negated_subject_values: Vec::new(),
        before_saved_at: None,
        after_saved_at: None,
        require_read: false,
        exclude_read: false,
        require_unread: false,
        exclude_unread: false,
        require_archived: false,
        exclude_archived: false,
        require_favorited: false,
        exclude_favorited: false,
        require_has_highlights: false,
        exclude_has_highlights: false,
        require_has_notes: false,
        exclude_has_notes: false,
        require_has_unsubscribe: false,
        exclude_has_unsubscribe: false,
        require_pinned: false,
        exclude_pinned: false,
        require_sender_blocked: false,
        exclude_sender_blocked: false,
        require_feed_only: false,
        exclude_feed_only: false,
        score_reference_at: cursor
            .map(|cursor| cursor.score_reference_at)
            .unwrap_or_else(Utc::now),
        cursor_score: cursor.map(|cursor| cursor.score),
        cursor_saved_at: cursor.map(|cursor| cursor.saved_at),
        cursor_result_id: cursor.map(|cursor| cursor.result_id),
        cursor_section_key: cursor.map(|cursor| cursor.section_key.clone()),
        limit,
    };

    for filter in &parsed.filters {
        match filter {
            SearchFilter::Tag { value, negated } => push_filter(
                value,
                *negated,
                &mut query.tag_values,
                &mut query.negated_tag_values,
            ),
            SearchFilter::Collection { value, negated } => push_filter(
                value,
                *negated,
                &mut query.collection_values,
                &mut query.negated_collection_values,
            ),
            SearchFilter::ContentType { value, negated } => push_filter(
                value,
                *negated,
                &mut query.type_values,
                &mut query.negated_type_values,
            ),
            SearchFilter::Author { value, negated } => push_filter(
                value,
                *negated,
                &mut query.author_values,
                &mut query.negated_author_values,
            ),
            SearchFilter::Url { value, negated } => push_filter(
                value,
                *negated,
                &mut query.url_values,
                &mut query.negated_url_values,
            ),
            SearchFilter::Entity { value, negated } => push_filter(
                value,
                *negated,
                &mut query.entity_values,
                &mut query.negated_entity_values,
            ),
            SearchFilter::Before { value } => {
                #[expect(
                    clippy::expect_used,
                    reason = "midnight is always a valid wall-clock time"
                )]
                let day_start = value
                    .and_hms_opt(0, 0, 0)
                    .expect("midnight is valid")
                    .and_utc();
                query.before_saved_at = Some(day_start + Duration::days(1));
            }
            SearchFilter::After { value } => {
                #[expect(
                    clippy::expect_used,
                    reason = "midnight is always a valid wall-clock time"
                )]
                let day_start = value
                    .and_hms_opt(0, 0, 0)
                    .expect("midnight is valid")
                    .and_utc();
                query.after_saved_at = Some(day_start);
            }
            SearchFilter::Status { value, negated } => match (value, negated) {
                (SearchStatusFilter::Read, false) => query.require_read = true,
                (SearchStatusFilter::Read, true) => query.exclude_read = true,
                (SearchStatusFilter::Unread, false) => query.require_unread = true,
                (SearchStatusFilter::Unread, true) => query.exclude_unread = true,
                (SearchStatusFilter::Archived, false) => query.require_archived = true,
                (SearchStatusFilter::Archived, true) => query.exclude_archived = true,
                (SearchStatusFilter::Favorited, false) => query.require_favorited = true,
                (SearchStatusFilter::Favorited, true) => query.exclude_favorited = true,
            },
            SearchFilter::Has { value, negated } => match (value, negated) {
                (SearchHasFilter::Highlights, false) => query.require_has_highlights = true,
                (SearchHasFilter::Highlights, true) => query.exclude_has_highlights = true,
                (SearchHasFilter::Notes, false) => query.require_has_notes = true,
                (SearchHasFilter::Notes, true) => query.exclude_has_notes = true,
                (SearchHasFilter::Unsubscribe, false) => query.require_has_unsubscribe = true,
                (SearchHasFilter::Unsubscribe, true) => query.exclude_has_unsubscribe = true,
            },
            SearchFilter::Pinned { value, negated } => match (*value, *negated) {
                (true, false) => query.require_pinned = true,
                (true, true) => query.exclude_pinned = true,
                (false, false) => query.exclude_pinned = true,
                (false, true) => query.require_pinned = true,
            },
            SearchFilter::Sender { value, negated } => push_sender_filter(
                value,
                *negated,
                &mut query.sender_values,
                &mut query.negated_sender_values,
            ),
            SearchFilter::SenderDomain { value, negated } => push_filter(
                value,
                *negated,
                &mut query.sender_domain_values,
                &mut query.negated_sender_domain_values,
            ),
            SearchFilter::ListId { value, negated } => push_filter(
                value,
                *negated,
                &mut query.list_values,
                &mut query.negated_list_values,
            ),
            SearchFilter::Subject { value, negated } => push_filter(
                value,
                *negated,
                &mut query.subject_values,
                &mut query.negated_subject_values,
            ),
            SearchFilter::SenderBlocked { negated } => {
                if *negated {
                    query.exclude_sender_blocked = true;
                } else {
                    query.require_sender_blocked = true;
                }
            }
            SearchFilter::Source { value, negated } => match (value, negated) {
                (SearchSourceFilter::Feed, false) => query.require_feed_only = true,
                (SearchSourceFilter::Feed, true) => query.exclude_feed_only = true,
                (SearchSourceFilter::Library, false) => query.exclude_feed_only = true,
                (SearchSourceFilter::Library, true) => query.require_feed_only = true,
            },
        }
    }

    query
}

fn push_filter(value: &str, negated: bool, positive: &mut Vec<String>, negative: &mut Vec<String>) {
    if negated {
        negative.push(value.to_lowercase());
    } else {
        positive.push(value.to_lowercase());
    }
}

fn push_sender_filter(
    value: &str,
    negated: bool,
    positive: &mut Vec<String>,
    negative: &mut Vec<String>,
) {
    let normalized = if value.contains('@') {
        CanonicalAddress::new(value).into_string()
    } else {
        value.to_lowercase()
    };
    if negated {
        negative.push(normalized);
    } else {
        positive.push(normalized);
    }
}

pub(crate) fn normalize_query(raw_query: &str) -> String {
    raw_query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub(crate) fn extract_last_token(raw_query: &str) -> String {
    tokenize(raw_query)
        .last()
        .cloned()
        .unwrap_or_else(|| raw_query.trim().to_string())
}

pub(crate) fn parse_query(raw_query: &str) -> ind_domain::ParsedSearchQuery {
    let tokens = tokenize(raw_query);
    let mut text_tokens = Vec::new();
    let mut filters = Vec::new();

    for token in tokens {
        if let Some(filter) = parse_filter(&token) {
            filters.push(filter);
        } else {
            text_tokens.push(strip_matching_quotes(&token).to_string());
        }
    }

    let text_query = {
        let joined = text_tokens.join(" ").trim().to_string();
        (!joined.is_empty()).then_some(joined)
    };

    ind_domain::ParsedSearchQuery {
        raw_query: raw_query.trim().to_string(),
        text_query,
        filters,
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in input.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            c if c.is_whitespace() && !in_quotes => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    tokens
}

fn parse_filter(token: &str) -> Option<SearchFilter> {
    let (negated, body) = match token.chars().next() {
        Some('!') | Some('-') => (true, &token[1..]),
        _ => (false, token),
    };

    let (key, raw_value) = body.split_once(':')?;
    if key.is_empty() || raw_value.is_empty() {
        return None;
    }

    let value = strip_matching_quotes(raw_value).trim();
    if value.is_empty() {
        return None;
    }

    match key.to_ascii_lowercase().as_str() {
        "tag" => Some(SearchFilter::Tag {
            value: value.to_string(),
            negated,
        }),
        "collection" => Some(SearchFilter::Collection {
            value: value.to_string(),
            negated,
        }),
        "type" => Some(SearchFilter::ContentType {
            value: value.to_string(),
            negated,
        }),
        "author" => Some(SearchFilter::Author {
            value: value.to_string(),
            negated,
        }),
        "before" => parse_date(value).and_then(|value| {
            if negated {
                value
                    .checked_add_signed(Duration::days(1))
                    .map(|value| SearchFilter::After { value })
            } else {
                Some(SearchFilter::Before { value })
            }
        }),
        "after" => parse_date(value).and_then(|value| {
            if negated {
                value
                    .checked_sub_signed(Duration::days(1))
                    .map(|value| SearchFilter::Before { value })
            } else {
                Some(SearchFilter::After { value })
            }
        }),
        "is" => {
            if let Some(status) = parse_status_filter(value) {
                Some(SearchFilter::Status {
                    value: status,
                    negated,
                })
            } else if value.eq_ignore_ascii_case("blocked") {
                Some(SearchFilter::SenderBlocked { negated })
            } else {
                parse_source_filter(value).map(|value| SearchFilter::Source { value, negated })
            }
        }
        "has" => parse_has_filter(value).map(|value| SearchFilter::Has { value, negated }),
        "sender" => Some(SearchFilter::Sender {
            value: value.to_string(),
            negated,
        }),
        "sender_domain" => Some(SearchFilter::SenderDomain {
            value: value.to_string(),
            negated,
        }),
        "list" => Some(SearchFilter::ListId {
            value: value.to_string(),
            negated,
        }),
        "subject" => Some(SearchFilter::Subject {
            value: value.to_string(),
            negated,
        }),
        "url" => Some(SearchFilter::Url {
            value: value.to_string(),
            negated,
        }),
        "entity" => Some(SearchFilter::Entity {
            value: value.to_string(),
            negated,
        }),
        "pinned" => parse_bool(value).map(|value| SearchFilter::Pinned { value, negated }),
        _ => None,
    }
}

fn strip_matching_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" => Some(true),
        "false" | "no" | "0" => Some(false),
        _ => None,
    }
}

fn parse_status_filter(value: &str) -> Option<SearchStatusFilter> {
    match value.to_ascii_lowercase().as_str() {
        "read" => Some(SearchStatusFilter::Read),
        "unread" => Some(SearchStatusFilter::Unread),
        "archived" => Some(SearchStatusFilter::Archived),
        "favorited" => Some(SearchStatusFilter::Favorited),
        _ => None,
    }
}

fn parse_has_filter(value: &str) -> Option<SearchHasFilter> {
    match value.to_ascii_lowercase().as_str() {
        "highlights" => Some(SearchHasFilter::Highlights),
        "notes" => Some(SearchHasFilter::Notes),
        "unsubscribe" => Some(SearchHasFilter::Unsubscribe),
        _ => None,
    }
}

fn parse_source_filter(value: &str) -> Option<SearchSourceFilter> {
    match value.to_ascii_lowercase().as_str() {
        "feed" => Some(SearchSourceFilter::Feed),
        "library" => Some(SearchSourceFilter::Library),
        _ => None,
    }
}
