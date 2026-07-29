use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AiOutputId, AiPromptPresetId, AiRunId, CollectionId, DocumentId, EntityType, MilaMessageId,
    MilaSessionId, UserId,
};

pub const MILA_EMBEDDING_DIM: i32 = 768;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiPromptAction {
    Summary,
    Tags,
    Entities,
    Chat,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiOutputType {
    Summary,
    Tags,
    Entities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MilaSessionType {
    SingleDocument,
    CrossItem,
    Collection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilaConfig {
    pub user_id: UserId,
    pub chat_api_base: String,
    pub chat_api_key_enc: Option<Vec<u8>>,
    pub chat_model: String,
    pub embedding_api_base: String,
    pub embedding_api_key_enc: Option<Vec<u8>>,
    pub embedding_model: String,
    pub embedding_dim: i32,
    /// Whether the user opts into their own AI provider. When false, the stored
    /// provider fields are retained but the user is on the managed/platform default.
    pub byo_enabled: bool,
    /// Total token window of the chat model (input + output). Source of truth for the
    /// per-action input cap; the chat RAG switchover is `chat_context_pct` of this.
    pub model_context_window: i32,
    /// Percent (1-100) of `model_context_window` the chat sends inline before switching to RAG.
    pub chat_context_pct: i32,
    pub chunk_size: i32,
    pub chunk_overlap: i32,
    pub top_k: i32,
    pub cross_item_top_k: i32,
    pub cross_item_max_per_item: i32,
    pub enabled: bool,
    /// OpenAI-style response_format json_schema support. Default true; off falls back
    /// to prompt-only JSON with tolerant parsing.
    pub supports_structured_output: bool,
    pub supports_reasoning_effort: bool,
    /// 0 = legacy plaintext bytes; 1 = AES-256-GCM sealed by CredentialCipher.
    pub chat_cipher_version: i16,
    /// 0 = legacy plaintext bytes; 1 = AES-256-GCM sealed by CredentialCipher.
    pub embedding_cipher_version: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MilaConfig {
    /// Resolve the provider Mila actually calls. When the user opts out of BYO,
    /// the managed platform defaults are used while their stored endpoints are
    /// retained on the row for when they opt back in. The master `enabled` flag
    /// and identity/timestamps are preserved.
    pub fn resolve_effective(&self, defaults: &MilaPlatformDefaults) -> MilaConfig {
        if self.byo_enabled {
            return self.clone();
        }
        let mut managed = defaults.materialize(self.user_id, self.updated_at);
        managed.enabled = self.enabled;
        managed.created_at = self.created_at;
        managed
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MilaPlatformDefaults {
    pub chat_api_base: String,
    pub chat_model: String,
    pub embedding_api_base: String,
    pub embedding_model: String,
    pub embedding_dim: i32,
    /// Total token window of the chat model. Defaults to 12000; override with
    /// `mila.model_context_window` (or `MILA_MODEL_CONTEXT_WINDOW`) to match your model.
    pub model_context_window: i32,
    #[serde(default = "default_chat_context_pct")]
    pub chat_context_pct: i32,
    pub chunk_size: i32,
    pub chunk_overlap: i32,
    pub top_k: i32,
    pub cross_item_top_k: i32,
    pub cross_item_max_per_item: i32,
    pub enabled: bool,
    #[serde(default = "default_supports_structured_output")]
    pub supports_structured_output: bool,
    #[serde(default)]
    pub supports_reasoning_effort: bool,
}

const fn default_supports_structured_output() -> bool {
    true
}

const fn default_chat_context_pct() -> i32 {
    70
}

impl MilaPlatformDefaults {
    /// Validate the budgeting fields that the HTTP write path checks per-user but that the
    /// platform-default path (env/config -> `materialize` for users without a row) would
    /// otherwise accept unchecked. Call at startup so a bad deployment config fails fast.
    pub fn validate(&self) -> Result<(), String> {
        if self.chat_api_base.trim().is_empty() {
            return Err("mila.chat_api_base is required".into());
        }
        if self.embedding_api_base.trim().is_empty() {
            return Err("mila.embedding_api_base is required".into());
        }
        if self.model_context_window <= 0 {
            return Err(format!(
                "mila.model_context_window must be greater than 0, got {}",
                self.model_context_window
            ));
        }
        if !(1..=100).contains(&self.chat_context_pct) {
            return Err(format!(
                "mila.chat_context_pct must be between 1 and 100, got {}",
                self.chat_context_pct
            ));
        }
        Ok(())
    }

    pub fn materialize(&self, user_id: UserId, now: DateTime<Utc>) -> MilaConfig {
        MilaConfig {
            user_id,
            chat_api_base: self.chat_api_base.clone(),
            chat_api_key_enc: None,
            chat_model: self.chat_model.clone(),
            embedding_api_base: self.embedding_api_base.clone(),
            embedding_api_key_enc: None,
            embedding_model: self.embedding_model.clone(),
            embedding_dim: self.embedding_dim,
            byo_enabled: false,
            model_context_window: self.model_context_window,
            chat_context_pct: self.chat_context_pct,
            chunk_size: self.chunk_size,
            chunk_overlap: self.chunk_overlap,
            top_k: self.top_k,
            cross_item_top_k: self.cross_item_top_k,
            cross_item_max_per_item: self.cross_item_max_per_item,
            enabled: self.enabled,
            supports_structured_output: self.supports_structured_output,
            supports_reasoning_effort: self.supports_reasoning_effort,
            chat_cipher_version: 1,
            embedding_cipher_version: 1,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPromptPreset {
    pub id: AiPromptPresetId,
    pub user_id: Option<UserId>,
    pub name: String,
    pub action: AiPromptAction,
    pub system_prompt: String,
    pub is_default: bool,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRun {
    pub id: AiRunId,
    pub user_id: UserId,
    pub document_id: Option<DocumentId>,
    pub action: AiPromptAction,
    pub provider: String,
    pub model: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub is_byok: bool,
    pub status: String,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiOutput {
    pub id: AiOutputId,
    pub document_id: Option<DocumentId>,
    pub output_type: AiOutputType,
    pub content: serde_json::Value,
    pub ai_run_id: Option<AiRunId>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub name: String,
    pub entity_type: EntityType,
    pub description: Option<String>,
    pub mention_count: i32,
    /// Well-known alternate names the model emits (synonyms and acronyms, e.g. Facebook for Meta).
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilaSession {
    pub id: MilaSessionId,
    pub user_id: UserId,
    pub document_id: Option<DocumentId>,
    pub collection_id: Option<CollectionId>,
    pub session_type: MilaSessionType,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilaMessage {
    pub id: MilaMessageId,
    pub session_id: MilaSessionId,
    pub role: MessageRole,
    pub content: String,
    pub source_chunks: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}
