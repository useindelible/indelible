use std::sync::Arc;

use crate::error::AppError;
use chrono::Utc;
use ind_domain::{MilaConfig, MilaPlatformDefaults, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ApiKeyUpdate {
    #[default]
    Preserve,
    Clear,
    Replace(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpsertMilaConfigInput {
    pub chat_api_base: Option<String>,
    pub chat_api_key: ApiKeyUpdate,
    pub chat_model: Option<String>,
    pub embedding_api_base: Option<String>,
    pub embedding_api_key: ApiKeyUpdate,
    pub embedding_model: Option<String>,
    pub embedding_dim: Option<i32>,
    pub model_context_window: Option<i32>,
    pub chat_context_pct: Option<i32>,
    pub chunk_size: Option<i32>,
    pub chunk_overlap: Option<i32>,
    pub top_k: Option<i32>,
    pub cross_item_top_k: Option<i32>,
    pub cross_item_max_per_item: Option<i32>,
    pub enabled: Option<bool>,
    pub byo_enabled: Option<bool>,
    pub supports_structured_output: Option<bool>,
    pub supports_reasoning_effort: Option<bool>,
}

#[async_trait::async_trait]
pub trait MilaConfigRepository: Send + Sync {
    async fn get_by_user(&self, user_id: UserId) -> Result<Option<MilaConfig>, AppError>;
    async fn upsert(&self, config: &MilaConfig) -> Result<MilaConfig, AppError>;
}

pub struct DefaultingMilaConfigRepository {
    inner: Arc<dyn MilaConfigRepository>,
    defaults: MilaPlatformDefaults,
}

impl DefaultingMilaConfigRepository {
    pub fn new(inner: Arc<dyn MilaConfigRepository>, defaults: MilaPlatformDefaults) -> Self {
        Self { inner, defaults }
    }
}

#[async_trait::async_trait]
impl MilaConfigRepository for DefaultingMilaConfigRepository {
    async fn get_by_user(&self, user_id: UserId) -> Result<Option<MilaConfig>, AppError> {
        let stored = self.inner.get_by_user(user_id).await?;
        Ok(Some(match stored {
            // Runtime consumers resolve the effective provider here: BYO when opted
            // in, the managed platform default otherwise (stored endpoints untouched).
            Some(config) => config.resolve_effective(&self.defaults),
            None => self.defaults.materialize(user_id, Utc::now()),
        }))
    }

    async fn upsert(&self, config: &MilaConfig) -> Result<MilaConfig, AppError> {
        self.inner.upsert(config).await
    }
}
