use chrono::Utc;

use super::*;
use crate::indexer::{MAX_INDEXED_TEXT_BYTES, cap_search_document};
use crate::query::{build_fts_query, parse_query};
use ind_domain::{DocumentId, SearchCursor, SearchDocument, UserId};

#[test]
fn parser_table_covers_text_filters_negation_and_malformed_fallback() {
    for (raw, text, filters) in [
        (
            "rust async tag:research author:\"Jane Doe\"",
            Some("rust async"),
            2,
        ),
        ("mars entity:\"Elon Musk\" -entity:OpenAI", Some("mars"), 2),
        ("gpu sender:alice@example.com", Some("gpu"), 1),
        ("newsletter is:feed", Some("newsletter"), 1),
        ("tag: rust", Some("tag: rust"), 0),
    ] {
        let parsed = parse_query(raw);
        assert_eq!(parsed.text_query.as_deref(), text, "{raw}");
        assert_eq!(parsed.filters.len(), filters, "{raw}");
    }
}

#[test]
fn fts_routing_preserves_email_entity_and_source_filters() {
    let parsed = parse_query(
        "tag:Research -tag:Old collection:Inbox -collection:Archive \
         type:Article -type:Video author:Jane -author:John url:example.com -url:evil \
         entity:\"Elon Musk\" -entity:OpenAI sender:NEWS@Example.COM \
         -sender:other@example.com sender_domain:Example.COM -list:other \
         subject:Brief has:unsubscribe -is:blocked is:feed before:2026-07-01 \
         after:2026-06-01 is:read -is:read is:unread -is:unread \
         is:archived -is:archived is:favorited -is:favorited \
         has:highlights -has:highlights has:notes -has:notes \
         pinned:true -pinned:true pinned:false -pinned:false is:library -is:library",
    );
    let query = build_fts_query(UserId::new(), &parsed, None, 20);
    assert_eq!(query.tag_values, ["research"]);
    assert_eq!(query.negated_tag_values, ["old"]);
    assert_eq!(query.collection_values, ["inbox"]);
    assert_eq!(query.negated_collection_values, ["archive"]);
    assert_eq!(query.type_values, ["article"]);
    assert_eq!(query.negated_type_values, ["video"]);
    assert_eq!(query.author_values, ["jane"]);
    assert_eq!(query.negated_author_values, ["john"]);
    assert_eq!(query.url_values, ["example.com"]);
    assert_eq!(query.negated_url_values, ["evil"]);
    assert_eq!(query.entity_values, ["elon musk"]);
    assert_eq!(query.negated_entity_values, ["openai"]);
    assert_eq!(query.sender_values, ["news@example.com"]);
    assert_eq!(query.negated_sender_values, ["other@example.com"]);
    assert_eq!(query.sender_domain_values, ["example.com"]);
    assert_eq!(query.negated_list_values, ["other"]);
    assert_eq!(query.subject_values, ["brief"]);
    assert!(query.require_has_unsubscribe && query.exclude_sender_blocked);
    assert!(query.before_saved_at.is_some() && query.after_saved_at.is_some());
    assert!(query.require_read && query.exclude_read);
    assert!(query.require_unread && query.exclude_unread);
    assert!(query.require_archived && query.exclude_archived);
    assert!(query.require_favorited && query.exclude_favorited);
    assert!(query.require_has_highlights && query.exclude_has_highlights);
    assert!(query.require_has_notes && query.exclude_has_notes);
    assert!(query.require_pinned && query.exclude_pinned);
    assert!(query.require_feed_only && query.exclude_feed_only);
}

#[test]
fn cursor_round_trip_preserves_stable_boundary_fields() {
    let cursor = SearchCursor {
        score: 0.42,
        score_reference_at: Utc::now(),
        saved_at: Utc::now(),
        result_id: uuid::Uuid::now_v7(),
        section_key: "chapter-1".into(),
    };
    let decoded =
        SearchEngine::decode_cursor(&SearchEngine::encode_cursor(&cursor).unwrap()).unwrap();
    assert_eq!(
        (decoded.result_id, decoded.section_key.as_str()),
        (cursor.result_id, "chapter-1")
    );
    assert_eq!(decoded.score_reference_at, cursor.score_reference_at);
}

#[test]
fn indexed_text_cap_is_utf8_safe_and_preserves_short_documents() {
    let short = document(
        "short",
        "body".into(),
        "highlight".into(),
        "metadata".into(),
    );
    let capped = cap_search_document(short.clone());
    assert_eq!(
        (
            capped.title,
            capped.body_text,
            capped.highlight_text,
            capped.metadata_text
        ),
        (
            short.title,
            short.body_text,
            short.highlight_text,
            short.metadata_text
        )
    );

    let long = document(
        "durable title",
        "界".repeat(MAX_INDEXED_TEXT_BYTES),
        "h".repeat(MAX_INDEXED_TEXT_BYTES),
        "m".repeat(MAX_INDEXED_TEXT_BYTES),
    );
    let capped = cap_search_document(long);
    let total = capped.title.len()
        + capped.body_text.len()
        + capped.highlight_text.len()
        + capped.metadata_text.len();
    assert!(total <= MAX_INDEXED_TEXT_BYTES);
    assert_eq!(capped.title, "durable title");
    assert!(capped.body_text.is_char_boundary(capped.body_text.len()));
}

fn document(title: &str, body: String, highlights: String, metadata: String) -> SearchDocument {
    SearchDocument {
        id: ind_domain::SearchDocumentId::new(),
        source: ind_domain::SearchDocumentSource::Document {
            document_id: DocumentId::new(),
        },
        user_id: UserId::new(),
        document_kind: ind_domain::SearchDocumentKind::Item,
        section_key: String::new(),
        section_title: None,
        title: title.into(),
        body_text: body,
        highlight_text: highlights,
        metadata_text: metadata,
        search_config: "simple".into(),
        saved_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
