use ind_domain::UserId;

use crate::error::AppError;

/// Result of a completed account purge, for the operator audit log.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AccountPurgeOutcome {
    pub documents_deleted: u64,
}

/// Permanent, transactional erasure of an account and everything it owns.
#[async_trait::async_trait]
pub trait AccountPurgeRepository: Send + Sync {
    /// Remove the user and every row the account owns in ONE transaction, and
    /// enqueue a durable job that removes the account's object-storage keys.
    ///
    /// Returns `DomainError::NotFound` if the user does not exist. Concurrent
    /// callers are serialized on the user row, so exactly one purge succeeds.
    async fn purge_account(&self, user_id: UserId) -> Result<AccountPurgeOutcome, AppError>;
}
