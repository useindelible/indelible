use super::*;

pub(super) async fn enqueue_document_ai_processing(
    outbox_repo: &dyn JobOutboxRepository,
    document_id: DocumentId,
) -> Result<(), AppError> {
    for entry in document_ai_processing_outbox(document_id, chrono::Utc::now()) {
        outbox_repo
            .enqueue(
                &entry.job_type,
                entry.payload,
                entry.dedupe_key,
                entry.available_at,
            )
            .await?;
    }
    Ok(())
}

pub(super) async fn enqueue_embedding_backfill(
    outbox_repo: &dyn JobOutboxRepository,
    embedding_backfill_repo: &dyn EmbeddingBackfillRepository,
    user_id: UserId,
    embedding_model: &str,
    embedding_dim: i32,
) -> Result<(), AppError> {
    let document_ids = embedding_backfill_repo
        .eligible_document_ids_for_backfill(user_id, embedding_model, embedding_dim)
        .await?;
    for document_id in document_ids {
        enqueue_document_ai_processing(outbox_repo, document_id).await?;
    }
    Ok(())
}

pub(super) fn mila_status_view(
    enabled: bool,
    eligible_items: i64,
    indexed_items: i64,
    stale_items: i64,
    has_pending_jobs: bool,
) -> MilaStatusOutput {
    let clamped_indexed = indexed_items.min(eligible_items).max(0);
    let progress_percent = if eligible_items <= 0 {
        0
    } else {
        ((clamped_indexed * 100) / eligible_items) as i32
    };
    MilaStatusOutput {
        enabled,
        eligible_items,
        indexed_items,
        stale_items,
        progress_percent,
        is_indexing: enabled && (indexed_items < eligible_items || has_pending_jobs),
        reindex_required: stale_items > 0,
    }
}

pub(super) fn mila_enable_requires_backfill(
    previous: Option<&MilaConfig>,
    next_enabled: bool,
) -> bool {
    next_enabled && !previous.map(|config| config.enabled).unwrap_or(false)
}

pub(super) fn mila_config_output(config: ind_domain::MilaConfig) -> MilaConfigOutput {
    MilaConfigOutput {
        chat_api_base: config.chat_api_base,
        chat_model: config.chat_model,
        has_chat_api_key: config.chat_api_key_enc.is_some(),
        embedding_api_base: config.embedding_api_base,
        embedding_model: config.embedding_model,
        has_embedding_api_key: config.embedding_api_key_enc.is_some(),
        embedding_dim: config.embedding_dim,
        byo_enabled: config.byo_enabled,
        model_context_window: config.model_context_window,
        chat_context_pct: config.chat_context_pct,
        top_k: config.top_k,
        cross_item_top_k: config.cross_item_top_k,
        cross_item_max_per_item: config.cross_item_max_per_item,
        enabled: config.enabled,
        supports_structured_output: config.supports_structured_output,
        supports_reasoning_effort: config.supports_reasoning_effort,
    }
}

pub(super) fn api_key_update(
    api_key: Option<String>,
    clear_provider_api_key: bool,
    cipher: Option<&ind_auth::CredentialCipher>,
) -> Result<ind_application::ApiKeyUpdate, AppError> {
    if clear_provider_api_key {
        Ok(ind_application::ApiKeyUpdate::Clear)
    } else if let Some(api_key) = api_key {
        let cipher = cipher.ok_or_else(|| AppError::ExternalService {
            service: "credential_cipher".into(),
            message: "AUTH_CREDENTIAL_KEY is required before saving Mila API keys".into(),
        })?;
        Ok(ind_application::ApiKeyUpdate::Replace(
            cipher.seal(api_key.as_bytes()),
        ))
    } else {
        Ok(ind_application::ApiKeyUpdate::Preserve)
    }
}

pub(super) fn chat_target_from_request(
    request: &CreateMilaSessionRequest,
) -> Result<ind_application::ChatTarget, AppError> {
    if let Some(document_id) = request.document_id {
        Ok(ind_application::ChatTarget::ExistingDocument(document_id))
    } else if let Some(delivery_id) = request.delivery_id {
        Ok(ind_application::ChatTarget::Delivery(delivery_id))
    } else {
        Err(AppError::Domain(ind_domain::DomainError::Validation {
            field: "document_id".into(),
            message: "single_document sessions require document_id or delivery_id".into(),
        }))
    }
}

pub(super) async fn ensure_collection_owned(
    collection_repo: &dyn CollectionRepository,
    user_id: UserId,
    collection_id: CollectionId,
) -> Result<(), AppError> {
    collection_repo
        .find_by_id(user_id, collection_id)
        .await?
        .ok_or_else(|| {
            AppError::Domain(ind_domain::DomainError::NotFound {
                entity: "Collection",
                id: collection_id.to_string(),
            })
        })?;
    Ok(())
}

pub(super) async fn ensure_mila_enabled(
    service: &ind_application::MilaConfigService,
    user_id: UserId,
) -> Result<(), AppError> {
    match service.get_config(user_id).await? {
        Some(config) if config.enabled => Ok(()),
        _ => Err(AppError::Domain(ind_domain::DomainError::Validation {
            field: "mila".into(),
            message: "Mila is not enabled for this user".into(),
        })),
    }
}

pub(super) fn mila_session_output(session: MilaSession) -> MilaSessionOutput {
    MilaSessionOutput {
        id: session.id,
        session_type: session.session_type,
        document_id: session.document_id,
        collection_id: session.collection_id,
        provenance: None,
        created_at: session.created_at,
        last_active: session.last_active,
    }
}

pub(super) async fn resolve_source_refs(
    content_vector_repo: &dyn ContentVectorRepository,
    chunk_ids: &[uuid::Uuid],
) -> Result<std::collections::HashMap<uuid::Uuid, (DocumentId, String)>, AppError> {
    if chunk_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let refs = content_vector_repo
        .source_refs_for_chunks(chunk_ids)
        .await?;

    Ok(refs
        .into_iter()
        .map(|source_ref| {
            (
                source_ref.chunk_id,
                (source_ref.document_id, source_ref.title),
            )
        })
        .collect())
}

pub(super) fn mila_message_output(
    message: MilaMessage,
    ref_map: &std::collections::HashMap<uuid::Uuid, (DocumentId, String)>,
) -> MilaMessageOutput {
    let source_refs_by_number = source_refs_by_source_number(&message.source_chunks, ref_map);
    let source_refs = cited_source_numbers(&message.content)
        .into_iter()
        .filter_map(|source_number| source_refs_by_number.get(&source_number).cloned())
        .collect();

    MilaMessageOutput {
        id: message.id,
        role: message.role,
        content: message.content,
        source_refs,
        created_at: message.created_at,
    }
}

fn source_refs_by_source_number(
    source_chunks: &[uuid::Uuid],
    ref_map: &std::collections::HashMap<uuid::Uuid, (DocumentId, String)>,
) -> std::collections::HashMap<usize, MilaSourceRefOutput> {
    let mut seen_documents = std::collections::HashSet::new();
    let mut refs = std::collections::HashMap::new();

    for chunk_id in source_chunks {
        let Some((document_id, item_title)) = ref_map.get(chunk_id) else {
            continue;
        };
        if !seen_documents.insert(*document_id) {
            continue;
        }

        let source_number = seen_documents.len();
        refs.insert(
            source_number,
            MilaSourceRefOutput {
                source_label: format!("S{source_number}"),
                document_id: *document_id,
                item_title: item_title.clone(),
            },
        );
    }

    refs
}

pub(super) fn cited_source_numbers(content: &str) -> Vec<usize> {
    let bytes = content.as_bytes();
    let mut source_numbers = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut cursor = 0;

    while cursor + 3 < bytes.len() {
        if bytes[cursor] != b'[' || bytes[cursor + 1] != b'S' {
            cursor += 1;
            continue;
        }

        let digit_start = cursor + 2;
        let mut digit_end = digit_start;
        while digit_end < bytes.len() && bytes[digit_end].is_ascii_digit() {
            digit_end += 1;
        }

        if digit_end == digit_start || digit_end >= bytes.len() || bytes[digit_end] != b']' {
            cursor += 1;
            continue;
        }

        if let Ok(source_number) = content[digit_start..digit_end].parse::<usize>()
            && source_number > 0
            && seen.insert(source_number)
        {
            source_numbers.push(source_number);
        }

        cursor = digit_end + 1;
    }

    source_numbers
}

pub(super) fn mila_prompt_preset_groups(
    presets: Vec<AiPromptPreset>,
) -> Vec<MilaPromptPresetGroupOutput> {
    // Actions where the user has explicitly set a default (system presets excluded).
    let user_default_actions: HashSet<_> = presets
        .iter()
        .filter(|p| !p.is_system && p.is_default)
        .map(|p| p.action)
        .collect();

    // Group while preserving the ORDER BY from the query (system first per action).
    let mut groups: Vec<(AiPromptAction, Vec<MilaPromptPresetOutput>)> = Vec::new();

    for preset in presets {
        let effective_default = if preset.is_system {
            !user_default_actions.contains(&preset.action)
        } else {
            preset.is_default
        };

        let output = MilaPromptPresetOutput {
            id: Some(preset.id),
            action: preset.action,
            name: preset.name,
            system_prompt: preset.system_prompt,
            is_default: effective_default,
            is_built_in: preset.is_system,
        };

        if let Some(group) = groups.iter_mut().find(|(a, _)| *a == preset.action) {
            group.1.push(output);
        } else {
            groups.push((preset.action, vec![output]));
        }
    }

    groups
        .into_iter()
        .map(|(action, presets)| MilaPromptPresetGroupOutput { action, presets })
        .collect()
}
