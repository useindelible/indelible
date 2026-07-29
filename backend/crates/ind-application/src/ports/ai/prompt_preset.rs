use super::*;

pub trait MilaPromptPresetPort: Send + Sync {
    fn list_prompt_presets(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<Vec<MilaPromptPresetGroupOutput>, AppError>>;

    fn create_prompt_preset(
        &self,
        user_id: UserId,
        request: CreateMilaPromptPresetRequest,
    ) -> BoxFuture<'_, Result<MilaPromptPresetOutput, AppError>>;

    fn update_prompt_preset(
        &self,
        user_id: UserId,
        preset_id: AiPromptPresetId,
        request: UpdateMilaPromptPresetRequest,
    ) -> BoxFuture<'_, Result<MilaPromptPresetOutput, AppError>>;

    fn delete_prompt_preset(
        &self,
        user_id: UserId,
        preset_id: AiPromptPresetId,
    ) -> BoxFuture<'_, Result<(), AppError>>;
}
