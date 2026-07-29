use std::sync::Arc;

use chrono::Utc;
use futures::future::BoxFuture;
use ind_domain::ops::{EmailUnsubscribeJob, job_types};
use ind_domain::{
    DomainError, EmailDestination, EmailSender, EmailSenderId, EmailSenderRenderDefault,
    JobOutboxId, UserId,
};

use crate::error::AppError;
use crate::ports::{EmailSenderOperations, EmailSenderUnsubscribeOutcome};
use crate::repos::email_sender::EmailSenderRepository;
use crate::repos::email_unsubscribe_commit::EmailUnsubscribeCommit;

pub struct UnsubscribeOutcome {
    pub sender: EmailSender,
    pub job_id: JobOutboxId,
}

pub struct EmailSenderService {
    repo: Arc<dyn EmailSenderRepository>,
    commit: Arc<dyn EmailUnsubscribeCommit>,
}

impl EmailSenderService {
    pub fn new(
        repo: Arc<dyn EmailSenderRepository>,
        commit: Arc<dyn EmailUnsubscribeCommit>,
    ) -> Self {
        Self { repo, commit }
    }

    pub async fn list(
        &self,
        user_id: UserId,
        offset: i64,
        limit: i64,
    ) -> Result<(Vec<EmailSender>, i64), AppError> {
        self.repo.list_for_user(user_id, offset, limit).await
    }

    pub async fn get(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> Result<EmailSender, AppError> {
        self.repo
            .find_by_id_and_user(user_id, sender_id)
            .await?
            .ok_or_else(|| not_found(sender_id))
    }

    pub async fn list_by_ids(
        &self,
        user_id: UserId,
        ids: &[EmailSenderId],
    ) -> Result<Vec<EmailSender>, AppError> {
        self.repo.list_by_ids_for_user(user_id, ids).await
    }

    pub async fn block(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> Result<EmailSender, AppError> {
        self.repo.block_for_user(user_id, sender_id).await?;
        self.get(user_id, sender_id).await
    }

    pub async fn unblock(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> Result<EmailSender, AppError> {
        self.repo.unblock_for_user(user_id, sender_id).await?;
        self.get(user_id, sender_id).await
    }

    pub async fn set_render_default(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
        value: EmailSenderRenderDefault,
    ) -> Result<EmailSender, AppError> {
        self.repo
            .set_render_default_for_user(user_id, sender_id, value)
            .await?;
        self.get(user_id, sender_id).await
    }

    pub async fn set_routing_default(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
        value: Option<EmailDestination>,
    ) -> Result<EmailSender, AppError> {
        self.repo
            .set_routing_default_for_user(user_id, sender_id, value)
            .await?;
        self.get(user_id, sender_id).await
    }

    pub async fn unsubscribe(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> Result<UnsubscribeOutcome, AppError> {
        let payload = serde_json::to_value(EmailUnsubscribeJob { user_id, sender_id })
            .map_err(|e| AppError::Repository(Box::new(e)))?;
        let dedupe_key = format!("{}:{user_id}:{sender_id}", job_types::EMAIL_UNSUBSCRIBE);

        let outcome = self
            .commit
            .commit_unsubscribe(user_id, sender_id, payload, dedupe_key, Utc::now())
            .await?;

        Ok(UnsubscribeOutcome {
            sender: outcome.sender,
            job_id: outcome.outbox.id,
        })
    }
}

impl EmailSenderOperations for EmailSenderService {
    fn list(
        &self,
        user_id: UserId,
        offset: i64,
        limit: i64,
    ) -> BoxFuture<'_, Result<(Vec<EmailSender>, i64), AppError>> {
        Box::pin(self.list(user_id, offset, limit))
    }

    fn get(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> BoxFuture<'_, Result<EmailSender, AppError>> {
        Box::pin(self.get(user_id, sender_id))
    }

    fn list_by_ids(
        &self,
        user_id: UserId,
        ids: Vec<EmailSenderId>,
    ) -> BoxFuture<'_, Result<Vec<EmailSender>, AppError>> {
        Box::pin(async move { self.list_by_ids(user_id, &ids).await })
    }

    fn block(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> BoxFuture<'_, Result<EmailSender, AppError>> {
        Box::pin(self.block(user_id, sender_id))
    }

    fn unblock(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> BoxFuture<'_, Result<EmailSender, AppError>> {
        Box::pin(self.unblock(user_id, sender_id))
    }

    fn set_render_default(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
        value: EmailSenderRenderDefault,
    ) -> BoxFuture<'_, Result<EmailSender, AppError>> {
        Box::pin(self.set_render_default(user_id, sender_id, value))
    }

    fn set_routing_default(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
        value: Option<EmailDestination>,
    ) -> BoxFuture<'_, Result<EmailSender, AppError>> {
        Box::pin(self.set_routing_default(user_id, sender_id, value))
    }

    fn unsubscribe(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> BoxFuture<'_, Result<EmailSenderUnsubscribeOutcome, AppError>> {
        Box::pin(async move {
            let outcome = self.unsubscribe(user_id, sender_id).await?;
            Ok(EmailSenderUnsubscribeOutcome {
                sender: outcome.sender,
                job_id: outcome.job_id,
            })
        })
    }
}

fn not_found(sender_id: EmailSenderId) -> AppError {
    AppError::Domain(DomainError::NotFound {
        entity: "EmailSender",
        id: sender_id.to_string(),
    })
}
