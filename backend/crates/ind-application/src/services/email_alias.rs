use std::sync::Arc;

use futures::future::BoxFuture;
use ind_domain::{
    AliasLocalPartError, DomainError, EmailAlias, EmailAliasId, EmailDestination, UserId,
    validate_local_part,
};

use crate::error::AppError;
use crate::ports::{EmailAliasCreateError, EmailAliasOperations};
use crate::repos::email_alias::{CreateEmailAlias, EmailAliasRepository};
use crate::repos::user::UserRepository;

/// Grace window during which a superseded default alias keeps resolving at the
/// webhook. After this many days have passed since the alias was marked for
/// retirement, `find_active` excludes it and incoming mail falls through to
/// the unknown-token log.
pub const DEFAULT_RETIRE_GRACE_DAYS: i64 = 28;

#[derive(Debug, thiserror::Error)]
pub enum EmailAliasServiceError {
    #[error("invalid local part: {0}")]
    LocalPart(#[from] AliasLocalPartError),
    #[error("local part collides with another account's seed token")]
    SeedTokenCollision,
    #[error(transparent)]
    Application(#[from] AppError),
}

pub struct EmailAliasService {
    repo: Arc<dyn EmailAliasRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl EmailAliasService {
    pub fn new(repo: Arc<dyn EmailAliasRepository>, user_repo: Arc<dyn UserRepository>) -> Self {
        Self { repo, user_repo }
    }

    pub async fn list(&self, user_id: UserId) -> Result<Vec<EmailAlias>, AppError> {
        self.repo.list_for_user(user_id).await
    }

    pub async fn create(
        &self,
        user_id: UserId,
        destination: EmailDestination,
        local_part: &str,
        is_default: bool,
    ) -> Result<EmailAlias, EmailAliasServiceError> {
        let normalized = validate_local_part(local_part)?;

        // Seed tokens (users.email_token) share the address namespace with
        // aliases and resolve from the same lookup path. Reject any alias
        // whose local_part matches another account's seed token, otherwise
        // the alias would intercept that user's inbound mail.
        if let Some(seed_owner) = self.user_repo.find_by_email_token(&normalized).await?
            && seed_owner.id != user_id
        {
            return Err(EmailAliasServiceError::SeedTokenCollision);
        }

        let alias = self
            .repo
            .create_with_default_rotation(
                CreateEmailAlias {
                    user_id,
                    destination,
                    local_part: &normalized,
                    is_default,
                },
                DEFAULT_RETIRE_GRACE_DAYS,
            )
            .await?;
        Ok(alias)
    }

    pub async fn delete(&self, user_id: UserId, alias_id: EmailAliasId) -> Result<(), AppError> {
        let alias = self
            .repo
            .find_by_id_and_user(user_id, alias_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "EmailAlias",
                    id: alias_id.to_string(),
                })
            })?;
        self.repo.retire(alias.id).await?;
        Ok(())
    }
}

impl EmailAliasOperations for EmailAliasService {
    fn list(&self, user_id: UserId) -> BoxFuture<'_, Result<Vec<EmailAlias>, AppError>> {
        Box::pin(self.list(user_id))
    }

    fn create(
        &self,
        user_id: UserId,
        destination: EmailDestination,
        local_part: String,
        is_default: bool,
    ) -> BoxFuture<'_, Result<EmailAlias, EmailAliasCreateError>> {
        Box::pin(async move {
            self.create(user_id, destination, &local_part, is_default)
                .await
                .map_err(|error| match error {
                    EmailAliasServiceError::LocalPart(error) => {
                        EmailAliasCreateError::InvalidLocalPart(error)
                    }
                    EmailAliasServiceError::SeedTokenCollision => {
                        EmailAliasCreateError::SeedTokenCollision
                    }
                    EmailAliasServiceError::Application(error) => {
                        EmailAliasCreateError::Application(error)
                    }
                })
        })
    }

    fn delete(
        &self,
        user_id: UserId,
        alias_id: EmailAliasId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.delete(user_id, alias_id))
    }
}
