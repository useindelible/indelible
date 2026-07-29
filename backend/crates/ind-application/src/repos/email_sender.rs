use crate::error::AppError;
use ind_domain::{
    CanonicalAddress, EmailDestination, EmailSender, EmailSenderId, EmailSenderRenderDefault,
    UserId,
};

#[async_trait::async_trait]
pub trait EmailSenderRepository: Send + Sync {
    async fn upsert_for_user(
        &self,
        user_id: UserId,
        canonical_addr: &CanonicalAddress,
        list_id: Option<&str>,
        display_name: Option<&str>,
    ) -> Result<EmailSender, AppError>;

    async fn find_by_user_and_canonical(
        &self,
        user_id: UserId,
        canonical_addr: &CanonicalAddress,
    ) -> Result<Option<EmailSender>, AppError>;

    async fn find_by_id_and_user(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> Result<Option<EmailSender>, AppError>;

    /// Paginated listing scoped to a single user. Returns (page, total).
    async fn list_for_user(
        &self,
        user_id: UserId,
        offset: i64,
        limit: i64,
    ) -> Result<(Vec<EmailSender>, i64), AppError>;

    /// Batch-fetch senders by id, filtered to those owned by `user_id`.
    /// Senders not owned by the user or not found are silently skipped.
    /// Used for enriching list views (e.g. search results) without N+1.
    async fn list_by_ids_for_user(
        &self,
        user_id: UserId,
        ids: &[EmailSenderId],
    ) -> Result<Vec<EmailSender>, AppError>;

    async fn block(&self, sender_id: EmailSenderId) -> Result<(), AppError>;

    async fn unblock(&self, sender_id: EmailSenderId) -> Result<(), AppError>;

    /// User-scoped variants that error with `NotFound` if the sender does not
    /// belong to `user_id`. Routes and services must use these.
    async fn block_for_user(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> Result<(), AppError>;

    async fn unblock_for_user(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> Result<(), AppError>;

    async fn set_render_default(
        &self,
        sender_id: EmailSenderId,
        value: EmailSenderRenderDefault,
    ) -> Result<(), AppError>;

    async fn set_render_default_for_user(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
        value: EmailSenderRenderDefault,
    ) -> Result<(), AppError>;

    async fn set_routing_default(
        &self,
        sender_id: EmailSenderId,
        value: Option<EmailDestination>,
    ) -> Result<(), AppError>;

    async fn set_routing_default_for_user(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
        value: Option<EmailDestination>,
    ) -> Result<(), AppError>;

    async fn increment_delivery(&self, sender_id: EmailSenderId) -> Result<(), AppError>;
}
