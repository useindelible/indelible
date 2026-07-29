use std::sync::Arc;

use ind_persistence::repos::{
    PgDocumentRepository, PgExportCursorRepository, PgHighlightRepository,
    PgIntegrationConnectionRepository, PgIntegrationOAuthTokenRepository, PgJobOutboxRepository,
    PgLibraryRepository, PgTagRepository,
};
use sqlx::PgPool;

pub struct Repositories {
    pub document: Arc<PgDocumentRepository>,
    pub export_cursor: Arc<PgExportCursorRepository>,
    pub highlight: Arc<PgHighlightRepository>,
    pub integration_connection: Arc<PgIntegrationConnectionRepository>,
    pub integration_oauth_token: Arc<PgIntegrationOAuthTokenRepository>,
    pub job_outbox: Arc<PgJobOutboxRepository>,
    pub library: Arc<PgLibraryRepository>,
    pub tag: Arc<PgTagRepository>,
}

impl Repositories {
    pub fn new(pool: &PgPool) -> Self {
        Self {
            document: Arc::new(PgDocumentRepository::new(pool.clone())),
            export_cursor: Arc::new(PgExportCursorRepository::new(pool.clone())),
            highlight: Arc::new(PgHighlightRepository::new(pool.clone())),
            integration_connection: Arc::new(PgIntegrationConnectionRepository::new(pool.clone())),
            integration_oauth_token: Arc::new(PgIntegrationOAuthTokenRepository::new(pool.clone())),
            job_outbox: Arc::new(PgJobOutboxRepository::new(pool.clone())),
            library: Arc::new(PgLibraryRepository::new(pool.clone())),
            tag: Arc::new(PgTagRepository::new(pool.clone())),
        }
    }
}
