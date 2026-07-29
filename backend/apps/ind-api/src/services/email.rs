use std::sync::Arc;

use crate::config::ServerConfig;
use crate::services::repositories::Repositories;
use ind_application::ports::{EmailAliasOperations, EmailIngestOperations, EmailSenderOperations};
use ind_persistence::repos::{
    PgEmailIngestLogRepository, PgEmailSenderRepository, PgEmailUnsubscribeCommit,
};
use secrecy::ExposeSecret;

type EmailIngestServices = (
    Option<Arc<dyn EmailIngestOperations>>,
    Option<Arc<dyn ind_integrations::email::InboundEmailProvider>>,
);

pub(super) struct EmailServices {
    pub ingest_ops: Option<Arc<dyn EmailIngestOperations>>,
    pub ingest_provider: Option<Arc<dyn ind_integrations::email::InboundEmailProvider>>,
    pub sender_ops: Option<Arc<dyn EmailSenderOperations>>,
    pub alias_ops: Option<Arc<dyn EmailAliasOperations>>,
}

pub(super) fn build_email_services(
    config: &ServerConfig,
    pool: &sqlx::PgPool,
    repos: &Repositories,
) -> EmailServices {
    let (ingest_ops, ingest_provider) = build_email_ingest(config, pool, repos);
    EmailServices {
        ingest_ops,
        ingest_provider,
        sender_ops: Some(build_email_sender_ops(pool)),
        alias_ops: Some(build_email_alias_ops(repos)),
    }
}

fn build_email_ingest(
    config: &ServerConfig,
    pool: &sqlx::PgPool,
    repos: &Repositories,
) -> EmailIngestServices {
    let ingest_log_repo = Arc::new(PgEmailIngestLogRepository::new(pool.clone()));
    let ops: Arc<dyn EmailIngestOperations> = Arc::new(
        ind_application::services::email_ingest::EmailIngestOperationsService::new(
            repos.user.clone(),
            ingest_log_repo,
            repos.email_alias.clone(),
        ),
    );

    let provider = match config.email_ingest.provider.as_deref() {
        Some("resend") => {
            let webhook_secret = config
                .email_ingest
                .webhook_secret
                .as_ref()
                .map(|s| s.expose_secret().to_string())
                .unwrap_or_default();
            let api_key = config
                .email_ingest
                .resend_api_key
                .as_ref()
                .map(|s| s.expose_secret().to_string())
                .unwrap_or_default();
            match ind_integrations::email::ResendProvider::new(&webhook_secret, api_key) {
                Ok(p) => {
                    Some(Arc::new(p) as Arc<dyn ind_integrations::email::InboundEmailProvider>)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "failed to initialize Resend provider");
                    None
                }
            }
        }
        _ => None,
    };

    (Some(ops), provider)
}

fn build_email_sender_ops(pool: &sqlx::PgPool) -> Arc<dyn EmailSenderOperations> {
    let repo = Arc::new(PgEmailSenderRepository::new(pool.clone()));
    let commit = Arc::new(PgEmailUnsubscribeCommit::new(pool.clone()));
    let service = ind_application::services::email_sender::EmailSenderService::new(repo, commit);
    Arc::new(service)
}

fn build_email_alias_ops(repos: &Repositories) -> Arc<dyn EmailAliasOperations> {
    let service = ind_application::services::email_alias::EmailAliasService::new(
        repos.email_alias.clone(),
        repos.user.clone(),
    );
    Arc::new(service)
}
