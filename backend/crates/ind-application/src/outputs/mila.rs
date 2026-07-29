use chrono::{DateTime, Utc};
use ind_domain::{
    AiPromptAction, AiPromptPresetId, CollectionId, DocumentId, DocumentProvenance, MessageRole,
    MilaMessageId, MilaSessionId, MilaSessionType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaConfigOutput {
    pub chat_api_base: String,
    pub chat_model: String,
    pub has_chat_api_key: bool,
    pub embedding_api_base: String,
    pub embedding_model: String,
    pub has_embedding_api_key: bool,
    pub embedding_dim: i32,
    pub byo_enabled: bool,
    pub model_context_window: i32,
    pub chat_context_pct: i32,
    pub top_k: i32,
    pub cross_item_top_k: i32,
    pub cross_item_max_per_item: i32,
    pub enabled: bool,
    pub supports_structured_output: bool,
    pub supports_reasoning_effort: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaStatusOutput {
    pub enabled: bool,
    pub eligible_items: i64,
    pub indexed_items: i64,
    pub stale_items: i64,
    pub progress_percent: i32,
    pub is_indexing: bool,
    pub reindex_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaPromptPresetOutput {
    pub id: Option<AiPromptPresetId>,
    pub action: AiPromptAction,
    pub name: String,
    pub system_prompt: String,
    pub is_default: bool,
    pub is_built_in: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaPromptPresetGroupOutput {
    pub action: AiPromptAction,
    pub presets: Vec<MilaPromptPresetOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaSessionOutput {
    pub id: MilaSessionId,
    pub session_type: MilaSessionType,
    pub document_id: Option<DocumentId>,
    pub collection_id: Option<CollectionId>,
    /// TASK-234 AC#6: composed provenance for single-document sessions (Library-backed vs
    /// Feed-prepared-unsaved). `None` for non-document session types.
    pub provenance: Option<DocumentProvenance>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaSourceRefOutput {
    pub source_label: String,
    /// Durable content identity of the cited source.
    pub document_id: DocumentId,
    pub item_title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaMessageOutput {
    pub id: MilaMessageId,
    pub role: MessageRole,
    pub content: String,
    pub source_refs: Vec<MilaSourceRefOutput>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaSessionWithPreviewOutput {
    pub id: MilaSessionId,
    pub session_type: MilaSessionType,
    pub document_id: Option<DocumentId>,
    pub collection_id: Option<CollectionId>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub preview_content: Option<String>,
    pub preview_role: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaConversationOutput {
    pub session: MilaSessionOutput,
    pub messages: Vec<MilaMessageOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaStreamDeltaOutput {
    pub delta: String,
    pub retrieval_degraded: Option<String>,
}
