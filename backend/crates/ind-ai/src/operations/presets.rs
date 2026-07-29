use super::helpers::*;
use super::*;

impl MilaPromptPresetPort for MilaOperationsService {
    fn list_prompt_presets(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<Vec<MilaPromptPresetGroupOutput>, AppError>> {
        Box::pin(async move {
            let presets = self.ai_preset_repo.list_by_user(user_id).await?;
            Ok(mila_prompt_preset_groups(presets))
        })
    }

    fn create_prompt_preset(
        &self,
        user_id: UserId,
        request: CreateMilaPromptPresetRequest,
    ) -> BoxFuture<'_, Result<MilaPromptPresetOutput, AppError>> {
        Box::pin(async move {
            let preset = AiPromptPreset {
                id: AiPromptPresetId::new(),
                user_id: Some(user_id),
                name: request.name,
                action: request.action,
                system_prompt: request.system_prompt,
                is_default: request.is_default,
                is_system: false,
                created_at: chrono::Utc::now(),
            };

            let created = self.ai_preset_repo.create(&preset).await?;
            Ok(MilaPromptPresetOutput {
                id: Some(created.id),
                action: created.action,
                name: created.name,
                system_prompt: created.system_prompt,
                is_default: created.is_default,
                is_built_in: false,
            })
        })
    }

    fn update_prompt_preset(
        &self,
        user_id: UserId,
        preset_id: AiPromptPresetId,
        request: UpdateMilaPromptPresetRequest,
    ) -> BoxFuture<'_, Result<MilaPromptPresetOutput, AppError>> {
        Box::pin(async move {
            let updated = self
                .ai_preset_repo
                .update(
                    preset_id,
                    user_id,
                    UpdateAiPromptPresetInput {
                        name: request.name,
                        system_prompt: request.system_prompt,
                        is_default: request.is_default,
                    },
                )
                .await?;
            Ok(MilaPromptPresetOutput {
                id: Some(updated.id),
                action: updated.action,
                name: updated.name,
                system_prompt: updated.system_prompt,
                is_default: updated.is_default,
                is_built_in: updated.is_system,
            })
        })
    }

    fn delete_prompt_preset(
        &self,
        user_id: UserId,
        preset_id: AiPromptPresetId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(async move { self.ai_preset_repo.delete(preset_id, user_id).await })
    }
}
