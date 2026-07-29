use super::*;

pub trait ExtensionSaveOperations: Send + Sync {
    fn quick_save(
        &self,
        user_id: UserId,
        input: QuickSaveInput,
    ) -> BoxFuture<'_, Result<SaveResult, AppError>>;

    fn reader_save(
        &self,
        user_id: UserId,
        input: ReaderSaveInput,
    ) -> BoxFuture<'_, Result<SaveResult, AppError>>;

    fn full_archive(
        &self,
        user_id: UserId,
        input: FullArchiveInput,
    ) -> BoxFuture<'_, Result<SaveResult, AppError>>;
}

#[derive(Clone, Copy)]
pub struct PatchExtensionEntryRequest {
    pub triage_state: Option<TriageState>,
    pub is_favorite: Option<bool>,
}
