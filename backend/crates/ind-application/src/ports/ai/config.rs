use super::*;

pub trait MilaConfigPort: Send + Sync {
    fn get_config(&self, user_id: UserId) -> BoxFuture<'_, Result<MilaConfigOutput, AppError>>;

    fn get_status(&self, user_id: UserId) -> BoxFuture<'_, Result<MilaStatusOutput, AppError>>;

    fn upsert_config(
        &self,
        user_id: UserId,
        request: UpdateMilaConfigRequest,
    ) -> BoxFuture<'_, Result<MilaConfigOutput, AppError>>;

    fn reindex_config(
        &self,
        user_id: UserId,
        request: UpdateMilaConfigRequest,
    ) -> BoxFuture<'_, Result<MilaConfigOutput, AppError>>;

    fn test_config(
        &self,
        user_id: UserId,
        request: TestMilaConfigRequest,
    ) -> BoxFuture<'_, Result<MilaProviderTestResult, AppError>>;
}
