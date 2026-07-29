use crate::error::AppError;
use ind_domain::{AiPromptAction, AiPromptPreset, AiPromptPresetId, UserId};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdateAiPromptPresetInput {
    pub name: Option<String>,
    pub system_prompt: Option<String>,
    pub is_default: Option<bool>,
}

#[async_trait::async_trait]
pub trait AiPromptPresetRepository: Send + Sync {
    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<AiPromptPreset>, AppError>;

    async fn find_by_id_for_user(
        &self,
        preset_id: AiPromptPresetId,
        user_id: UserId,
    ) -> Result<Option<AiPromptPreset>, AppError>;

    async fn create(&self, preset: &AiPromptPreset) -> Result<AiPromptPreset, AppError>;

    async fn update(
        &self,
        preset_id: AiPromptPresetId,
        user_id: UserId,
        input: UpdateAiPromptPresetInput,
    ) -> Result<AiPromptPreset, AppError>;

    async fn delete(&self, preset_id: AiPromptPresetId, user_id: UserId) -> Result<(), AppError>;

    async fn find_default_for_action(
        &self,
        user_id: UserId,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError>;

    async fn find_system_preset_for_action(
        &self,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError>;
}
