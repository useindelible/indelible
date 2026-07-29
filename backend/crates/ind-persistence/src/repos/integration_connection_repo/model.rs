use chrono::{DateTime, Utc};
use uuid::Uuid;

use ind_application::AppError;
use ind_domain::{
    DomainError, IntegrationConnection, IntegrationConnectionId, IntegrationProvider, UserId,
};

#[derive(sqlx::FromRow)]
pub(super) struct ConnectionRow {
    pub(super) id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) provider: String,
    pub(super) config: serde_json::Value,
    pub(super) status: String,
    pub(super) last_sync_at: Option<DateTime<Utc>>,
    pub(super) last_error: Option<String>,
    pub(super) created_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
    pub(super) version: i64,
}

impl TryFrom<ConnectionRow> for IntegrationConnection {
    type Error = AppError;

    fn try_from(row: ConnectionRow) -> Result<Self, Self::Error> {
        Ok(IntegrationConnection {
            id: IntegrationConnectionId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            provider: parse_provider(&row.provider)?,
            config: row.config,
            status: row.status,
            last_sync_at: row.last_sync_at,
            last_error: row.last_error,
            created_at: row.created_at,
            updated_at: row.updated_at,
            version: row.version,
        })
    }
}

pub(super) fn parse_provider(s: &str) -> Result<IntegrationProvider, AppError> {
    match s {
        "obsidian" => Ok(IntegrationProvider::Obsidian),
        "notion" => Ok(IntegrationProvider::Notion),
        "logseq" => Ok(IntegrationProvider::Logseq),
        "browser_extension" => Ok(IntegrationProvider::BrowserExtension),
        "email_ingest" => Ok(IntegrationProvider::EmailIngest),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid integration provider: {other}"),
        })),
    }
}

pub(super) fn provider_to_str(provider: IntegrationProvider) -> &'static str {
    match provider {
        IntegrationProvider::Obsidian => "obsidian",
        IntegrationProvider::Notion => "notion",
        IntegrationProvider::Logseq => "logseq",
        IntegrationProvider::BrowserExtension => "browser_extension",
        IntegrationProvider::EmailIngest => "email_ingest",
    }
}
