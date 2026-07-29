use super::*;

pub trait MilaChatPort: Send + Sync {
    fn stream_chat(
        &self,
        user_id: UserId,
        request: MilaStreamRequest,
    ) -> BoxFuture<'_, Result<MilaStreamOutputStream, AppError>>;
}
