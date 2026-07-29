use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::FieldError;
use crate::extract::Validate;
use ind_application::outputs::mila::{MilaConfigOutput, MilaStatusOutput};
use ind_application::ports::{
    MilaProviderTestResult, TestMilaConfigRequest, UpdateMilaConfigRequest,
};
use ind_domain::ai::MILA_EMBEDDING_DIM;

use super::{validate_http_url, validate_positive, validate_required};

pub(super) const DEFAULT_CHAT_CONTEXT_PCT: i32 = 70;
pub(super) const DEFAULT_TOP_K: i32 = 6;
pub(super) const DEFAULT_CROSS_ITEM_TOP_K: i32 = 20;
pub(super) const DEFAULT_CROSS_ITEM_MAX_PER_ITEM: i32 = 3;

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertMilaConfigBody {
    pub chat_api_base: String,
    #[serde(default)]
    #[schema(write_only)]
    pub chat_api_key: Option<String>,
    #[serde(default)]
    pub clear_chat_api_key: bool,
    pub chat_model: String,
    pub embedding_api_base: String,
    #[serde(default)]
    #[schema(write_only)]
    pub embedding_api_key: Option<String>,
    #[serde(default)]
    pub clear_embedding_api_key: bool,
    pub embedding_model: String,
    pub embedding_dim: i32,
    pub model_context_window: i32,
    #[serde(default = "default_chat_context_pct")]
    pub chat_context_pct: i32,
    #[serde(default = "default_top_k")]
    pub top_k: i32,
    #[serde(default = "default_cross_item_top_k")]
    pub cross_item_top_k: i32,
    #[serde(default = "default_cross_item_max_per_item")]
    pub cross_item_max_per_item: i32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    #[schema(value_type = bool)]
    pub byo_enabled: Option<bool>,
    #[serde(default)]
    #[schema(value_type = bool)]
    pub supports_structured_output: Option<bool>,
    #[serde(default)]
    #[schema(value_type = bool)]
    pub supports_reasoning_effort: Option<bool>,
}

impl Validate for UpsertMilaConfigBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();

        validate_required("chat_api_base", &self.chat_api_base, &mut errors);
        validate_http_url("chat_api_base", &self.chat_api_base, &mut errors);
        validate_required("chat_model", &self.chat_model, &mut errors);
        validate_required("embedding_api_base", &self.embedding_api_base, &mut errors);
        validate_http_url("embedding_api_base", &self.embedding_api_base, &mut errors);
        validate_required("embedding_model", &self.embedding_model, &mut errors);
        validate_positive("embedding_dim", self.embedding_dim, &mut errors);
        validate_embedding_dim("embedding_dim", self.embedding_dim, &mut errors);
        validate_positive(
            "model_context_window",
            self.model_context_window,
            &mut errors,
        );
        if !(1..=100).contains(&self.chat_context_pct) {
            errors.push(FieldError {
                field: "chat_context_pct".into(),
                message: "must be between 1 and 100".into(),
            });
        }
        validate_positive("top_k", self.top_k, &mut errors);
        validate_positive("cross_item_top_k", self.cross_item_top_k, &mut errors);
        validate_positive(
            "cross_item_max_per_item",
            self.cross_item_max_per_item,
            &mut errors,
        );

        if self.cross_item_max_per_item > self.cross_item_top_k {
            errors.push(FieldError {
                field: "cross_item_max_per_item".into(),
                message: "must be less than or equal to cross_item_top_k".into(),
            });
        }

        if self.clear_chat_api_key && self.chat_api_key_text().is_some() {
            errors.push(FieldError {
                field: "chat_api_key".into(),
                message: "cannot be provided when clear_chat_api_key is true".into(),
            });
        }

        if self.clear_embedding_api_key && self.embedding_api_key_text().is_some() {
            errors.push(FieldError {
                field: "embedding_api_key".into(),
                message: "cannot be provided when clear_embedding_api_key is true".into(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl UpsertMilaConfigBody {
    pub fn into_state_request(self) -> UpdateMilaConfigRequest {
        UpdateMilaConfigRequest {
            chat_api_base: self.chat_api_base.trim().to_string(),
            chat_api_key: self.chat_api_key_text(),
            clear_chat_api_key: self.clear_chat_api_key,
            chat_model: self.chat_model.trim().to_string(),
            embedding_api_base: self.embedding_api_base.trim().to_string(),
            embedding_api_key: self.embedding_api_key_text(),
            clear_embedding_api_key: self.clear_embedding_api_key,
            embedding_model: self.embedding_model.trim().to_string(),
            embedding_dim: self.embedding_dim,
            model_context_window: self.model_context_window,
            chat_context_pct: self.chat_context_pct,
            top_k: self.top_k,
            cross_item_top_k: self.cross_item_top_k,
            cross_item_max_per_item: self.cross_item_max_per_item,
            enabled: self.enabled,
            byo_enabled: self.byo_enabled,
            supports_structured_output: self.supports_structured_output,
            supports_reasoning_effort: self.supports_reasoning_effort,
        }
    }

    fn chat_api_key_text(&self) -> Option<String> {
        trimmed_text(self.chat_api_key.as_deref())
    }

    fn embedding_api_key_text(&self) -> Option<String> {
        trimmed_text(self.embedding_api_key.as_deref())
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TestMilaConfigBody {
    pub chat_api_base: String,
    #[serde(default)]
    #[schema(write_only)]
    pub chat_api_key: Option<String>,
    pub chat_model: String,
    pub embedding_api_base: String,
    #[serde(default)]
    #[schema(write_only)]
    pub embedding_api_key: Option<String>,
    pub embedding_model: String,
    #[serde(default = "default_embedding_dim")]
    pub embedding_dim: i32,
}

impl Validate for TestMilaConfigBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        validate_required("chat_api_base", &self.chat_api_base, &mut errors);
        validate_http_url("chat_api_base", &self.chat_api_base, &mut errors);
        validate_required("chat_model", &self.chat_model, &mut errors);
        validate_required("embedding_api_base", &self.embedding_api_base, &mut errors);
        validate_http_url("embedding_api_base", &self.embedding_api_base, &mut errors);
        validate_required("embedding_model", &self.embedding_model, &mut errors);
        validate_positive("embedding_dim", self.embedding_dim, &mut errors);
        validate_embedding_dim("embedding_dim", self.embedding_dim, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl TestMilaConfigBody {
    pub fn into_state_request(self) -> TestMilaConfigRequest {
        TestMilaConfigRequest {
            chat_api_base: self.chat_api_base.trim().to_string(),
            chat_api_key: trimmed_text(self.chat_api_key.as_deref()),
            chat_model: self.chat_model.trim().to_string(),
            embedding_api_base: self.embedding_api_base.trim().to_string(),
            embedding_api_key: trimmed_text(self.embedding_api_key.as_deref()),
            embedding_model: self.embedding_model.trim().to_string(),
            embedding_dim: self.embedding_dim,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaConfigResponse {
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

pub fn project_mila_config(output: MilaConfigOutput) -> MilaConfigResponse {
    MilaConfigResponse {
        chat_api_base: output.chat_api_base,
        chat_model: output.chat_model,
        has_chat_api_key: output.has_chat_api_key,
        embedding_api_base: output.embedding_api_base,
        embedding_model: output.embedding_model,
        has_embedding_api_key: output.has_embedding_api_key,
        embedding_dim: output.embedding_dim,
        byo_enabled: output.byo_enabled,
        model_context_window: output.model_context_window,
        chat_context_pct: output.chat_context_pct,
        top_k: output.top_k,
        cross_item_top_k: output.cross_item_top_k,
        cross_item_max_per_item: output.cross_item_max_per_item,
        enabled: output.enabled,
        supports_structured_output: output.supports_structured_output,
        supports_reasoning_effort: output.supports_reasoning_effort,
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TestMilaConfigResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_dim: Option<i32>,
    pub chat_model_ok: bool,
    pub embedding_model_ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn project_mila_provider_test(result: MilaProviderTestResult) -> TestMilaConfigResponse {
    TestMilaConfigResponse {
        success: result.success,
        embedding_dim: result.embedding_dim,
        chat_model_ok: result.chat_model_ok,
        embedding_model_ok: result.embedding_model_ok,
        chat_error: result.chat_error,
        embedding_error: result.embedding_error,
        error: result.error,
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaStatusResponse {
    pub enabled: bool,
    pub eligible_items: i64,
    pub indexed_items: i64,
    pub stale_items: i64,
    pub progress_percent: i32,
    pub is_indexing: bool,
    pub reindex_required: bool,
}

pub fn project_mila_status(output: MilaStatusOutput) -> MilaStatusResponse {
    MilaStatusResponse {
        enabled: output.enabled,
        eligible_items: output.eligible_items,
        indexed_items: output.indexed_items,
        stale_items: output.stale_items,
        progress_percent: output.progress_percent,
        is_indexing: output.is_indexing,
        reindex_required: output.reindex_required,
    }
}

const fn default_chat_context_pct() -> i32 {
    DEFAULT_CHAT_CONTEXT_PCT
}

const fn default_top_k() -> i32 {
    DEFAULT_TOP_K
}

const fn default_cross_item_top_k() -> i32 {
    DEFAULT_CROSS_ITEM_TOP_K
}

const fn default_cross_item_max_per_item() -> i32 {
    DEFAULT_CROSS_ITEM_MAX_PER_ITEM
}

const fn default_embedding_dim() -> i32 {
    MILA_EMBEDDING_DIM
}

fn validate_embedding_dim(field: &str, value: i32, errors: &mut Vec<FieldError>) {
    if value > 0 && value != MILA_EMBEDDING_DIM {
        errors.push(FieldError {
            field: field.into(),
            message: format!("must be exactly {MILA_EMBEDDING_DIM}"),
        });
    }
}

fn trimmed_text(value: Option<&str>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

const fn default_enabled() -> bool {
    true
}
