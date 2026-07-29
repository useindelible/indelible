use ind_domain::{
    AiOutputType, AiPromptAction, AiRunId, ContentSource, DocumentId, HighlightId, LibraryEntry,
    LibraryEntryId, NewDomainEvent, Tag, TriageState, UserId, build_domain_event,
};

pub fn ai_output_completed(
    user_id: UserId,
    document_id: DocumentId,
    output_type: AiOutputType,
    run_id: AiRunId,
) -> NewDomainEvent {
    document_event(
        "ai.output.completed",
        user_id,
        document_id,
        serde_json::json!({
            "document_id": document_id.to_string(),
            "action": ai_output_type(output_type),
            "ai_run_id": run_id.to_string()
        }),
    )
}

pub fn ai_output_failed(
    user_id: UserId,
    document_id: DocumentId,
    action: AiPromptAction,
    run_id: AiRunId,
    message: &str,
) -> NewDomainEvent {
    document_event(
        "ai.output.failed",
        user_id,
        document_id,
        serde_json::json!({
            "document_id": document_id.to_string(),
            "action": ai_prompt_action(action),
            "ai_run_id": run_id.to_string(),
            "message": message
        }),
    )
}

fn ai_output_type(value: AiOutputType) -> &'static str {
    match value {
        AiOutputType::Summary => "summary",
        AiOutputType::Tags => "tags",
        AiOutputType::Entities => "entities",
    }
}

fn ai_prompt_action(value: AiPromptAction) -> &'static str {
    match value {
        AiPromptAction::Summary => "summary",
        AiPromptAction::Tags => "tags",
        AiPromptAction::Entities => "entities",
        AiPromptAction::Chat => "chat",
        AiPromptAction::Custom => "custom",
    }
}

pub fn document_highlighted(
    user_id: UserId,
    document_id: DocumentId,
    highlight_id: HighlightId,
) -> NewDomainEvent {
    highlight_payload_event(
        "document.highlighted",
        "document",
        user_id,
        document_id,
        highlight_id,
    )
}

pub fn highlight_updated(
    user_id: UserId,
    document_id: DocumentId,
    highlight_id: HighlightId,
) -> NewDomainEvent {
    highlight_payload_event(
        "highlight.updated",
        "highlight",
        user_id,
        document_id,
        highlight_id,
    )
}

pub fn highlight_deleted(
    user_id: UserId,
    document_id: DocumentId,
    highlight_id: HighlightId,
) -> NewDomainEvent {
    highlight_payload_event(
        "highlight.deleted",
        "highlight",
        user_id,
        document_id,
        highlight_id,
    )
}

pub fn highlight_noted(
    user_id: UserId,
    document_id: DocumentId,
    highlight_id: HighlightId,
) -> NewDomainEvent {
    highlight_payload_event(
        "highlight.noted",
        "highlight",
        user_id,
        document_id,
        highlight_id,
    )
}

fn highlight_payload_event(
    event_type: &str,
    aggregate_type: &str,
    user_id: UserId,
    document_id: DocumentId,
    highlight_id: HighlightId,
) -> NewDomainEvent {
    let aggregate_id = if aggregate_type == "highlight" {
        *highlight_id.as_uuid()
    } else {
        *document_id.as_uuid()
    };
    build_domain_event(
        event_type,
        aggregate_type,
        aggregate_id,
        user_id,
        serde_json::json!({
            "document_id": document_id.to_string(),
            "highlight_id": highlight_id.to_string()
        }),
    )
}

fn content_source(source: ContentSource) -> &'static str {
    match source {
        ContentSource::Manual => "manual",
        ContentSource::Extension => "extension",
        ContentSource::ShareSheet => "share_sheet",
        ContentSource::Feed => "feed",
        ContentSource::Email => "email",
        ContentSource::Api => "api",
        ContentSource::Cli => "cli",
        ContentSource::Import => "import",
    }
}

fn triage_state(state: TriageState) -> &'static str {
    match state {
        TriageState::Inbox => "inbox",
        TriageState::Later => "later",
        TriageState::Archive => "archive",
    }
}

pub fn document_materialized(
    user_id: UserId,
    document_id: DocumentId,
    source: ContentSource,
) -> NewDomainEvent {
    document_event(
        "document.materialized",
        user_id,
        document_id,
        serde_json::json!({
            "document_id": document_id.to_string(),
            "source": content_source(source)
        }),
    )
}

pub fn document_engaged(
    user_id: UserId,
    document_id: DocumentId,
    engagement_kind: &'static str,
) -> NewDomainEvent {
    document_event(
        "document.engaged",
        user_id,
        document_id,
        serde_json::json!({
            "document_id": document_id.to_string(),
            "engagement_kind": engagement_kind
        }),
    )
}

fn document_event(
    event_type: &str,
    user_id: UserId,
    document_id: DocumentId,
    payload: serde_json::Value,
) -> NewDomainEvent {
    build_domain_event(
        event_type,
        "document",
        *document_id.as_uuid(),
        user_id,
        payload,
    )
}

pub fn library_entry_saved(
    user_id: UserId,
    library_entry_id: LibraryEntryId,
    document_id: DocumentId,
    source: ContentSource,
) -> NewDomainEvent {
    build_domain_event(
        "library_entry.saved",
        "library_entry",
        *library_entry_id.as_uuid(),
        user_id,
        serde_json::json!({
            "library_entry_id": library_entry_id.to_string(),
            "document_id": document_id.to_string(),
            "source": content_source(source)
        }),
    )
}

fn library_entry_event(
    event_type: &str,
    user_id: UserId,
    library_entry_id: LibraryEntryId,
    payload: serde_json::Value,
) -> NewDomainEvent {
    build_domain_event(
        event_type,
        "library_entry",
        *library_entry_id.as_uuid(),
        user_id,
        payload,
    )
}

/// `library_entry.archived` when the entry landed in Archive, otherwise `library_entry.triaged`.
pub fn library_entry_triaged(user_id: UserId, entry: &LibraryEntry) -> NewDomainEvent {
    let event_type = if entry.triage_state == TriageState::Archive {
        "library_entry.archived"
    } else {
        "library_entry.triaged"
    };
    library_entry_event(
        event_type,
        user_id,
        entry.id,
        serde_json::json!({
            "library_entry_id": entry.id.to_string(),
            "document_id": entry.document_id.to_string(),
            "triage_state": triage_state(entry.triage_state)
        }),
    )
}

/// Always `library_entry.favorited`; `is_favorite` carries the resulting state, including the
/// un-favorite case (`false`). The catalog has no separate `unfavorited` event.
pub fn library_entry_favorite_changed(user_id: UserId, entry: &LibraryEntry) -> NewDomainEvent {
    library_entry_event(
        "library_entry.favorited",
        user_id,
        entry.id,
        serde_json::json!({
            "library_entry_id": entry.id.to_string(),
            "document_id": entry.document_id.to_string(),
            "is_favorite": entry.is_favorite
        }),
    )
}

pub fn library_entry_trashed(
    user_id: UserId,
    library_entry_id: LibraryEntryId,
    document_id: DocumentId,
) -> NewDomainEvent {
    library_entry_event(
        "library_entry.trashed",
        user_id,
        library_entry_id,
        serde_json::json!({
            "library_entry_id": library_entry_id.to_string(),
            "document_id": document_id.to_string()
        }),
    )
}

pub fn library_entry_restored(user_id: UserId, entry: &LibraryEntry) -> NewDomainEvent {
    library_entry_event(
        "library_entry.restored",
        user_id,
        entry.id,
        serde_json::json!({
            "library_entry_id": entry.id.to_string(),
            "document_id": entry.document_id.to_string()
        }),
    )
}

pub fn library_entry_permanently_deleted(
    user_id: UserId,
    library_entry_id: LibraryEntryId,
    document_id: DocumentId,
) -> NewDomainEvent {
    library_entry_event(
        "library_entry.permanently_deleted",
        user_id,
        library_entry_id,
        serde_json::json!({
            "library_entry_id": library_entry_id.to_string(),
            "document_id": document_id.to_string()
        }),
    )
}

/// Single event carrying the full resulting tag set (replace semantics).
pub fn library_entry_tagged(
    user_id: UserId,
    library_entry_id: LibraryEntryId,
    document_id: DocumentId,
    tags: &[Tag],
) -> NewDomainEvent {
    library_entry_event(
        "library_entry.tagged",
        user_id,
        library_entry_id,
        serde_json::json!({
            "library_entry_id": library_entry_id.to_string(),
            "document_id": document_id.to_string(),
            "tag_ids": tags.iter().map(|tag| tag.id.to_string()).collect::<Vec<_>>(),
            "tags": tags.iter().map(|tag| tag.name.clone()).collect::<Vec<_>>()
        }),
    )
}
