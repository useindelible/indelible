use ind_domain::{DocumentId, PlaybackKind, PlaybackState, UserId};

use crate::AppError;

#[async_trait::async_trait]
pub trait PlaybackStateRepository: Send + Sync {
    async fn upsert(&self, state: &PlaybackState) -> Result<PlaybackState, AppError>;

    async fn get(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        kind: PlaybackKind,
    ) -> Result<Option<PlaybackState>, AppError>;
}
