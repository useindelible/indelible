use crate::error::AppError;
use ind_domain::{TtsVoicePersona, TtsVoicePersonaId, UserId};

#[async_trait::async_trait]
pub trait TtsVoicePersonaRepository: Send + Sync {
    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<TtsVoicePersona>, AppError>;
    async fn get(
        &self,
        id: TtsVoicePersonaId,
        user_id: UserId,
    ) -> Result<Option<TtsVoicePersona>, AppError>;
    async fn insert(&self, persona: &TtsVoicePersona) -> Result<TtsVoicePersona, AppError>;
    async fn update_fields(&self, persona: &TtsVoicePersona) -> Result<TtsVoicePersona, AppError>;
    async fn delete(&self, id: TtsVoicePersonaId, user_id: UserId) -> Result<bool, AppError>;
}
