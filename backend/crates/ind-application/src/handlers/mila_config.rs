use std::sync::Arc;

use chrono::{DateTime, Utc};
use ind_domain::ai::MILA_EMBEDDING_DIM;
use ind_domain::{DomainError, MilaConfig, MilaPlatformDefaults, UserId};

use crate::AppError;
use crate::repos::mila_config::{ApiKeyUpdate, MilaConfigRepository, UpsertMilaConfigInput};

pub struct MilaConfigService {
    repo: Arc<dyn MilaConfigRepository>,
    defaults: MilaPlatformDefaults,
}

impl MilaConfigService {
    pub fn new(repo: Arc<dyn MilaConfigRepository>, defaults: MilaPlatformDefaults) -> Self {
        Self { repo, defaults }
    }

    pub fn platform_defaults(&self) -> &MilaPlatformDefaults {
        &self.defaults
    }

    pub async fn get_config(&self, user_id: UserId) -> Result<Option<MilaConfig>, AppError> {
        let current = self.repo.get_by_user(user_id).await?;
        Ok(Some(current.unwrap_or_else(|| {
            self.defaults.materialize(user_id, Utc::now())
        })))
    }

    pub async fn upsert_config(
        &self,
        user_id: UserId,
        input: UpsertMilaConfigInput,
    ) -> Result<MilaConfig, AppError> {
        let current = self.repo.get_by_user(user_id).await?;
        let next = merge_config(user_id, current.as_ref(), &self.defaults, input, Utc::now())?;
        self.repo.upsert(&next).await
    }
}

fn merge_config(
    user_id: UserId,
    current: Option<&MilaConfig>,
    defaults: &MilaPlatformDefaults,
    input: UpsertMilaConfigInput,
    now: DateTime<Utc>,
) -> Result<MilaConfig, AppError> {
    let chat_api_base = resolve_string(
        "chat_api_base",
        input.chat_api_base,
        current.map(|config| config.chat_api_base.as_str()),
    )?;
    let chat_model = resolve_string(
        "chat_model",
        input.chat_model,
        current.map(|config| config.chat_model.as_str()),
    )?;
    let embedding_api_base = resolve_string(
        "embedding_api_base",
        input.embedding_api_base,
        current.map(|config| config.embedding_api_base.as_str()),
    )?;
    let embedding_model = resolve_string(
        "embedding_model",
        input.embedding_model,
        current.map(|config| config.embedding_model.as_str()),
    )?;
    let embedding_dim = validate_positive(
        "embedding_dim",
        resolve_i32(
            "embedding_dim",
            input.embedding_dim,
            current.map(|config| config.embedding_dim),
        )?,
    )?;
    if embedding_dim != MILA_EMBEDDING_DIM {
        return Err(validation_owned(
            "embedding_dim",
            format!("must be exactly {MILA_EMBEDDING_DIM} for the current vector index"),
        ));
    }
    let model_context_window = validate_positive(
        "model_context_window",
        input
            .model_context_window
            .or(current.map(|config| config.model_context_window))
            .unwrap_or(defaults.model_context_window),
    )?;
    let chat_context_pct = validate_percent(
        "chat_context_pct",
        input
            .chat_context_pct
            .or(current.map(|config| config.chat_context_pct))
            .unwrap_or(defaults.chat_context_pct),
    )?;
    let chunk_size = validate_positive(
        "chunk_size",
        input
            .chunk_size
            .or(current.map(|config| config.chunk_size))
            .unwrap_or(defaults.chunk_size),
    )?;
    let chunk_overlap = validate_non_negative(
        "chunk_overlap",
        input
            .chunk_overlap
            .or(current.map(|config| config.chunk_overlap))
            .unwrap_or(defaults.chunk_overlap),
    )?;
    let top_k = validate_positive(
        "top_k",
        input
            .top_k
            .or(current.map(|config| config.top_k))
            .unwrap_or(defaults.top_k),
    )?;
    let cross_item_top_k = validate_positive(
        "cross_item_top_k",
        input
            .cross_item_top_k
            .or(current.map(|config| config.cross_item_top_k))
            .unwrap_or(defaults.cross_item_top_k),
    )?;
    let cross_item_max_per_item = validate_positive(
        "cross_item_max_per_item",
        input
            .cross_item_max_per_item
            .or(current.map(|config| config.cross_item_max_per_item))
            .unwrap_or(defaults.cross_item_max_per_item),
    )?;

    if chunk_overlap >= chunk_size {
        return Err(validation(
            "chunk_overlap",
            "must be smaller than chunk_size",
        ));
    }

    if cross_item_max_per_item > cross_item_top_k {
        return Err(validation(
            "cross_item_max_per_item",
            "must be less than or equal to cross_item_top_k",
        ));
    }

    validate_base_change_key_update(
        "chat_api_key",
        current.map(|config| config.chat_api_base.as_str()),
        &chat_api_base,
        current.and_then(|config| config.chat_api_key_enc.as_deref()),
        &input.chat_api_key,
    )?;
    validate_base_change_key_update(
        "embedding_api_key",
        current.map(|config| config.embedding_api_base.as_str()),
        &embedding_api_base,
        current.and_then(|config| config.embedding_api_key_enc.as_deref()),
        &input.embedding_api_key,
    )?;

    let chat_cipher_version = next_key_cipher_version(
        current.map(|config| config.chat_cipher_version),
        &input.chat_api_key,
    );
    let embedding_cipher_version = next_key_cipher_version(
        current.map(|config| config.embedding_cipher_version),
        &input.embedding_api_key,
    );
    let chat_api_key_enc = next_api_key(
        current.and_then(|config| config.chat_api_key_enc.as_deref()),
        input.chat_api_key,
    );
    let embedding_api_key_enc = next_api_key(
        current.and_then(|config| config.embedding_api_key_enc.as_deref()),
        input.embedding_api_key,
    );

    Ok(MilaConfig {
        user_id,
        chat_api_base,
        chat_api_key_enc,
        chat_model,
        embedding_api_base,
        embedding_api_key_enc,
        embedding_model,
        embedding_dim,
        model_context_window,
        chat_context_pct,
        chunk_size,
        chunk_overlap,
        top_k,
        cross_item_top_k,
        cross_item_max_per_item,
        enabled: input.enabled.unwrap_or_else(|| {
            current
                .map(|config| config.enabled)
                .unwrap_or(defaults.enabled)
        }),
        byo_enabled: input
            .byo_enabled
            .or(current.map(|config| config.byo_enabled))
            .unwrap_or(false),
        supports_structured_output: input.supports_structured_output.unwrap_or_else(|| {
            current
                .map(|config| config.supports_structured_output)
                .unwrap_or(defaults.supports_structured_output)
        }),
        supports_reasoning_effort: input.supports_reasoning_effort.unwrap_or_else(|| {
            current
                .map(|config| config.supports_reasoning_effort)
                .unwrap_or(defaults.supports_reasoning_effort)
        }),
        chat_cipher_version,
        embedding_cipher_version,
        created_at: current.map(|config| config.created_at).unwrap_or(now),
        updated_at: now,
    })
}

fn resolve_string(
    field: &'static str,
    incoming: Option<String>,
    current: Option<&str>,
) -> Result<String, AppError> {
    match incoming {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(validation(field, "is required"))
            } else {
                Ok(trimmed.to_string())
            }
        }
        None => current
            .map(ToOwned::to_owned)
            .ok_or_else(|| validation(field, "is required")),
    }
}

fn resolve_i32(
    field: &'static str,
    incoming: Option<i32>,
    current: Option<i32>,
) -> Result<i32, AppError> {
    incoming
        .or(current)
        .ok_or_else(|| validation(field, "is required"))
}

fn validate_positive(field: &'static str, value: i32) -> Result<i32, AppError> {
    if value > 0 {
        Ok(value)
    } else {
        Err(validation(field, "must be greater than 0"))
    }
}

fn validate_non_negative(field: &'static str, value: i32) -> Result<i32, AppError> {
    if value >= 0 {
        Ok(value)
    } else {
        Err(validation(field, "must be 0 or greater"))
    }
}

fn validate_percent(field: &'static str, value: i32) -> Result<i32, AppError> {
    if (1..=100).contains(&value) {
        Ok(value)
    } else {
        Err(validation(field, "must be between 1 and 100"))
    }
}

fn next_api_key(current: Option<&[u8]>, update: ApiKeyUpdate) -> Option<Vec<u8>> {
    match update {
        ApiKeyUpdate::Preserve => current.map(<[u8]>::to_vec),
        ApiKeyUpdate::Clear => None,
        ApiKeyUpdate::Replace(bytes) => Some(bytes),
    }
}

fn next_key_cipher_version(current: Option<i16>, update: &ApiKeyUpdate) -> i16 {
    if matches!(update, ApiKeyUpdate::Replace(_)) {
        1
    } else {
        current.unwrap_or(1)
    }
}

fn validate_base_change_key_update(
    key_field: &'static str,
    current_base: Option<&str>,
    next_base: &str,
    current_key: Option<&[u8]>,
    key_update: &ApiKeyUpdate,
) -> Result<(), AppError> {
    if current_base.is_some_and(|base| base != next_base)
        && current_key.is_some_and(|key| !key.is_empty())
        && matches!(key_update, ApiKeyUpdate::Preserve)
    {
        return Err(validation(
            key_field,
            "must replace or clear the saved key when changing the provider base",
        ));
    }
    Ok(())
}

fn validation(field: &'static str, message: &'static str) -> AppError {
    AppError::Domain(DomainError::Validation {
        field: field.to_string(),
        message: message.to_string(),
    })
}

fn validation_owned(field: &'static str, message: String) -> AppError {
    AppError::Domain(DomainError::Validation {
        field: field.to_string(),
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> MilaPlatformDefaults {
        MilaPlatformDefaults {
            chat_api_base: "https://managed-chat.example/v1".into(),
            chat_model: "managed-chat".into(),
            embedding_api_base: "https://managed-embedding.example/v1".into(),
            embedding_model: "managed-embedding".into(),
            embedding_dim: MILA_EMBEDDING_DIM,
            model_context_window: 8_192,
            chat_context_pct: 70,
            chunk_size: 1_000,
            chunk_overlap: 100,
            top_k: 6,
            cross_item_top_k: 20,
            cross_item_max_per_item: 3,
            enabled: true,
            supports_structured_output: true,
            supports_reasoning_effort: false,
        }
    }

    fn current_config(user_id: UserId) -> MilaConfig {
        let now = Utc::now();
        MilaConfig {
            user_id,
            chat_api_base: "https://saved-chat.example/v1".into(),
            chat_api_key_enc: Some(b"saved-chat-ciphertext".to_vec()),
            chat_model: "saved-chat".into(),
            embedding_api_base: "https://saved-embedding.example/v1".into(),
            embedding_api_key_enc: Some(b"saved-embedding-ciphertext".to_vec()),
            embedding_model: "saved-embedding".into(),
            embedding_dim: MILA_EMBEDDING_DIM,
            model_context_window: 8_192,
            chat_context_pct: 70,
            chunk_size: 1_000,
            chunk_overlap: 100,
            top_k: 6,
            cross_item_top_k: 20,
            cross_item_max_per_item: 3,
            enabled: true,
            byo_enabled: true,
            supports_structured_output: true,
            supports_reasoning_effort: false,
            chat_cipher_version: 1,
            embedding_cipher_version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    fn assert_validation_field(error: AppError, expected_field: &str) {
        let AppError::Domain(DomainError::Validation { field, message }) = error else {
            panic!("expected validation error, got {error:?}");
        };
        assert_eq!(field, expected_field);
        assert!(message.contains("replace or clear"));
    }

    #[test]
    fn changing_a_provider_base_cannot_preserve_its_saved_key() {
        let user_id = UserId::new();
        let current = current_config(user_id);

        let chat_error = merge_config(
            user_id,
            Some(&current),
            &defaults(),
            UpsertMilaConfigInput {
                chat_api_base: Some("https://new-chat.example/v1".into()),
                ..Default::default()
            },
            Utc::now(),
        )
        .expect_err("the saved chat key must not move to a different provider base");
        assert_validation_field(chat_error, "chat_api_key");

        let embedding_error = merge_config(
            user_id,
            Some(&current),
            &defaults(),
            UpsertMilaConfigInput {
                embedding_api_base: Some("https://new-embedding.example/v1".into()),
                ..Default::default()
            },
            Utc::now(),
        )
        .expect_err("the saved embedding key must not move to a different provider base");
        assert_validation_field(embedding_error, "embedding_api_key");
    }

    #[test]
    fn changing_a_provider_base_accepts_a_replaced_or_cleared_key() {
        let user_id = UserId::new();
        let current = current_config(user_id);

        let replaced = merge_config(
            user_id,
            Some(&current),
            &defaults(),
            UpsertMilaConfigInput {
                chat_api_base: Some("https://new-chat.example/v1".into()),
                chat_api_key: ApiKeyUpdate::Replace(b"new-chat-ciphertext".to_vec()),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        assert_eq!(replaced.chat_api_base, "https://new-chat.example/v1");
        assert_eq!(
            replaced.chat_api_key_enc,
            Some(b"new-chat-ciphertext".to_vec())
        );

        let cleared = merge_config(
            user_id,
            Some(&current),
            &defaults(),
            UpsertMilaConfigInput {
                embedding_api_base: Some("https://new-embedding.example/v1".into()),
                embedding_api_key: ApiKeyUpdate::Clear,
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();
        assert_eq!(
            cleared.embedding_api_base,
            "https://new-embedding.example/v1"
        );
        assert_eq!(cleared.embedding_api_key_enc, None);
    }

    #[test]
    fn changing_a_provider_base_can_preserve_an_empty_legacy_key() {
        let user_id = UserId::new();
        let mut current = current_config(user_id);
        current.chat_api_key_enc = Some(Vec::new());
        current.embedding_api_key_enc = Some(Vec::new());

        let merged = merge_config(
            user_id,
            Some(&current),
            &defaults(),
            UpsertMilaConfigInput {
                chat_api_base: Some("http://localhost:11434/v1".into()),
                embedding_api_base: Some("http://localhost:11435/v1".into()),
                ..Default::default()
            },
            Utc::now(),
        )
        .unwrap();

        assert_eq!(merged.chat_api_base, "http://localhost:11434/v1");
        assert_eq!(merged.embedding_api_base, "http://localhost:11435/v1");
    }
}
