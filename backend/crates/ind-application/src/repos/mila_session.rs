use chrono::{DateTime, Utc};

use crate::error::AppError;
use ind_domain::{MessageRole, MilaMessage, MilaSession, MilaSessionId, UserId};

pub struct MilaSessionWithPreview {
    pub session: MilaSession,
    pub preview_content: Option<String>,
    pub preview_role: Option<MessageRole>,
}

#[async_trait::async_trait]
pub trait MilaSessionRepository: Send + Sync {
    async fn create_session(&self, session: &MilaSession) -> Result<MilaSession, AppError>;

    async fn find_session_for_user(
        &self,
        session_id: MilaSessionId,
        user_id: UserId,
    ) -> Result<Option<MilaSession>, AppError>;

    async fn list_sessions_for_user(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<MilaSessionWithPreview>, AppError>;

    async fn insert_message(
        &self,
        user_id: UserId,
        message: &MilaMessage,
    ) -> Result<MilaMessage, AppError>;

    async fn list_messages(
        &self,
        session_id: MilaSessionId,
        user_id: UserId,
    ) -> Result<Vec<MilaMessage>, AppError>;

    async fn touch_session(
        &self,
        session_id: MilaSessionId,
        user_id: UserId,
        last_active: DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn delete_session(
        &self,
        session_id: MilaSessionId,
        user_id: UserId,
    ) -> Result<(), AppError>;
}
