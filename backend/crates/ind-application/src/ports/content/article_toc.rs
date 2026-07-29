use ind_domain::{DocumentId, UserId};

use crate::error::AppError;
use crate::handlers::article_toc::StoredArticleToc;

/// Read model for the article ToC endpoint. `Pending` covers missing, stale,
/// and not-yet-readable alike — the client contract is "poll until terminal".
#[derive(Debug)]
pub enum ArticleTocReadOutput {
    /// A stored outline whose source version matches the current readable
    /// asset. Its own `status` field distinguishes `ready` from `none`.
    Available(StoredArticleToc),
    Pending,
}

/// HTTP-facing port for the article table of contents. On a miss or stale
/// outline the operation enqueues one deduped `document.toc.ensure` job before
/// returning `Pending`; the transport layer never touches repositories,
/// storage, or the outbox directly.
#[async_trait::async_trait]
pub trait ArticleTocOperations: Send + Sync {
    async fn get_or_request(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<ArticleTocReadOutput, AppError>;
}
