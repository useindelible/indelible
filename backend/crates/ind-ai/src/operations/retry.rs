use chrono::Utc;
use futures::FutureExt;
use ind_application::repos::lifecycle_outbox::{
    document_ai_entities_outbox, document_ai_summarize_outbox, document_ai_tags_outbox,
};

use super::helpers::ensure_mila_enabled;
use super::*;

impl MilaActionRetryPort for MilaOperationsService {
    fn retry_document_action(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        action: RetryMilaDocumentAction,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        async move {
            self.document_repo
                .find_by_id(user_id, document_id)
                .await?
                .ok_or_else(|| {
                    AppError::Domain(DomainError::NotFound {
                        entity: "Document",
                        id: document_id.to_string(),
                    })
                })?;

            ensure_mila_enabled(&self.service, user_id).await?;

            let entry = match action {
                RetryMilaDocumentAction::Summary => {
                    document_ai_summarize_outbox(document_id, Utc::now())
                }
                RetryMilaDocumentAction::Tags => document_ai_tags_outbox(document_id, Utc::now()),
                RetryMilaDocumentAction::Entities => {
                    document_ai_entities_outbox(document_id, Utc::now())
                }
            };
            self.outbox_repo
                .enqueue(
                    &entry.job_type,
                    entry.payload,
                    entry.dedupe_key,
                    entry.available_at,
                )
                .await?;
            Ok(())
        }
        .boxed()
    }
}
