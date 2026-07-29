use super::*;

pub trait MilaSessionPort: Send + Sync {
    fn list_sessions(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> BoxFuture<'_, Result<Vec<MilaSessionWithPreviewOutput>, AppError>>;

    fn create_session(
        &self,
        user_id: UserId,
        request: CreateMilaSessionRequest,
    ) -> BoxFuture<'_, Result<MilaSessionOutput, AppError>>;

    fn get_session_messages(
        &self,
        user_id: UserId,
        session_id: MilaSessionId,
    ) -> BoxFuture<'_, Result<MilaConversationOutput, AppError>>;

    fn delete_session(
        &self,
        user_id: UserId,
        session_id: MilaSessionId,
    ) -> BoxFuture<'_, Result<(), AppError>>;
}
