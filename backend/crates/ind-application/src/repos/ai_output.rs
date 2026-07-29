use super::event::MutationSideEffects;
use crate::error::AppError;
use ind_domain::{AiOutput, AiOutputType, DocumentId};

#[async_trait::async_trait]
pub trait AiOutputRepository: Send + Sync {
    async fn upsert(
        &self,
        output: &AiOutput,
        effects: MutationSideEffects,
    ) -> Result<AiOutput, AppError>;

    async fn list_for_document(
        &self,
        document_id: DocumentId,
        output_type: Option<AiOutputType>,
    ) -> Result<Vec<AiOutput>, AppError>;

    async fn list_for_documents(
        &self,
        document_ids: &[DocumentId],
        output_type: Option<AiOutputType>,
    ) -> Result<Vec<AiOutput>, AppError>;

    async fn delete_by_document_and_type(
        &self,
        document_id: DocumentId,
        output_type: AiOutputType,
    ) -> Result<(), AppError>;
}
