use super::*;

fn build_library_context_response(
    config: &crate::state::AppConfig,
    joined: ind_domain::LibraryEntryWithDocument,
    tags: Vec<ind_domain::Tag>,
    note: Option<ind_domain::DocumentNote>,
) -> ExtensionSavedEntryResponse {
    let entry = joined.entry;
    let document = joined.document;
    let url = document
        .original_url
        .clone()
        .or_else(|| document.canonical_url.clone());
    let tags = tags
        .into_iter()
        .map(|t| TagResponse {
            id: t.id.to_string(),
            name: t.name,
            color: t.color,
        })
        .collect();
    let note = note.map(|n| ExtensionNoteResponse {
        id: n.id.to_string(),
        body: n.body,
        created_at: n.created_at,
        updated_at: n.updated_at,
    });

    ExtensionSavedEntryResponse {
        library_entry_id: entry.id.to_string(),
        document_id: document.id.to_string(),
        reader_url: document_reader_url(config, &document.id),
        title: document.title,
        url,
        triage_state: entry.triage_state.as_str().to_string(),
        is_favorite: entry.is_favorite,
        saved_at: entry.saved_at,
        tags,
        note,
    }
}

pub(super) async fn library_context_response(
    state: &AppState,
    user_id: ind_domain::UserId,
    entry_id: ind_domain::LibraryEntryId,
) -> Result<ExtensionSavedEntryResponse, ApiError> {
    let joined = library_entry_for_alias(state, user_id, entry_id).await?;
    let tags = if let Some(tag_ops) = state.tag_ops.as_ref() {
        tag_ops
            .list_library_entry_tags(user_id, entry_id)
            .await
            .map_err(ApiError::from)?
    } else {
        Vec::new()
    };
    let note = if let Some(document_reader_ops) = state.document_reader_ops.as_ref() {
        document_reader_ops
            .get_note(user_id, joined.document.id)
            .await
            .map_err(ApiError::from)?
    } else {
        None
    };

    Ok(build_library_context_response(
        &state.config,
        joined,
        tags,
        note,
    ))
}

pub(super) async fn library_entry_for_alias(
    state: &AppState,
    user_id: ind_domain::UserId,
    entry_id: ind_domain::LibraryEntryId,
) -> Result<ind_domain::LibraryEntryWithDocument, ApiError> {
    let library_ops = state
        .library_ops
        .as_ref()
        .ok_or_else(|| ApiError::NotFound {
            entity: "SavedEntry",
            id: entry_id.to_string(),
        })?;
    library_ops
        .get(user_id, entry_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound {
            entity: "SavedEntry",
            id: entry_id.to_string(),
        })
}

pub(super) async fn patch_library_context_response(
    state: &AppState,
    user_id: ind_domain::UserId,
    entry_id: ind_domain::LibraryEntryId,
    req: PatchExtensionEntryRequest,
) -> Result<ExtensionSavedEntryResponse, ApiError> {
    let library_ops = state
        .library_ops
        .as_ref()
        .ok_or_else(|| ApiError::NotFound {
            entity: "SavedEntry",
            id: entry_id.to_string(),
        })?;
    let current = library_ops
        .get(user_id, entry_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound {
            entity: "SavedEntry",
            id: entry_id.to_string(),
        })?;

    if let Some(triage_state) = req.triage_state {
        library_ops
            .set_triage(user_id, entry_id, triage_state)
            .await
            .map_err(ApiError::from)?;
    }
    if let Some(is_favorite) = req.is_favorite
        && current.entry.is_favorite != is_favorite
    {
        library_ops
            .toggle_favorite(user_id, entry_id)
            .await
            .map_err(ApiError::from)?;
    }

    library_context_response(state, user_id, entry_id).await
}
