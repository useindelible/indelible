use futures::future::BoxFuture;
use ind_domain::{
    EmailAlias, EmailAliasId, EmailDestination, EmailSender, EmailSenderId,
    EmailSenderRenderDefault, ImportJob, ImportJobId, IntegrationConnection,
    IntegrationConnectionId, IntegrationOAuthProvider, LibraryEntryId, NotionExportSettings,
    ObsidianExportSettings, User, UserId, WebhookDelivery, WebhookEndpoint, WebhookEndpointId,
};

use crate::AppError;
use crate::outputs::export::{
    ObsidianArtifactDownload, ObsidianExportPreview, ObsidianRefreshResult, ObsidianRunStatus,
};
use crate::outputs::import::ImportStatusOutput;
use crate::repos::integration_connection::NotionExportItemsPage;

pub trait WebhookOperations: Send + Sync {
    fn list_endpoints(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<Vec<WebhookEndpoint>, AppError>>;

    fn create_endpoint(
        &self,
        user_id: UserId,
        name: String,
        url: String,
        events: Vec<String>,
        is_active: bool,
    ) -> BoxFuture<'_, Result<(WebhookEndpoint, String), AppError>>;

    fn update_endpoint(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
        name: Option<String>,
        url: Option<String>,
        events: Option<Vec<String>>,
        is_active: Option<bool>,
    ) -> BoxFuture<'_, Result<WebhookEndpoint, AppError>>;

    fn delete_endpoint(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
    ) -> BoxFuture<'_, Result<(), AppError>>;

    fn rotate_secret(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
    ) -> BoxFuture<'_, Result<(WebhookEndpoint, String), AppError>>;

    fn test_endpoint(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
        event_type: String,
    ) -> BoxFuture<'_, Result<WebhookDelivery, AppError>>;

    fn list_deliveries(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
        limit: i64,
    ) -> BoxFuture<'_, Result<Vec<WebhookDelivery>, AppError>>;
}

pub trait EmailIngestOperations: Send + Sync {
    fn claim_and_enqueue(
        &self,
        input: crate::repos::email_ingest::ClaimAndEnqueueInput<'_>,
    ) -> BoxFuture<'_, Result<Option<crate::repos::email_ingest::EmailIngestLogRow>, AppError>>;

    fn resolve_ingest_recipient(
        &self,
        destination: EmailDestination,
        local_part: &str,
    ) -> BoxFuture<'_, Result<Option<User>, AppError>>;
}

pub struct ImportUpload {
    pub bytes: Vec<u8>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

pub struct ReadwiseImportUpload {
    pub library_csv: Option<ImportUpload>,
    pub archive_zip: Option<ImportUpload>,
    pub feeds_opml: Option<ImportUpload>,
}

pub trait ImportOperations: Send + Sync {
    fn upload_readwise(
        &self,
        user_id: UserId,
        upload: ReadwiseImportUpload,
    ) -> BoxFuture<'_, Result<ImportJob, AppError>>;

    fn get_status(
        &self,
        user_id: UserId,
        id: ImportJobId,
    ) -> BoxFuture<'_, Result<ImportStatusOutput, AppError>>;

    fn rollback(&self, user_id: UserId, id: ImportJobId) -> BoxFuture<'_, Result<(), AppError>>;

    fn list_recent(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> BoxFuture<'_, Result<Vec<ImportStatusOutput>, AppError>>;
}

#[derive(Debug, Clone)]
pub struct IntegrationAuthorizeStart {
    pub authorize_url: String,
}

#[derive(Debug, Clone)]
pub struct IntegrationSyncEnqueued {
    pub job_id: String,
}

#[derive(Debug)]
pub struct NotionRefreshEnqueued {
    pub job_id: String,
    pub archived_page_url: Option<String>,
}

pub trait IntegrationOperations: Send + Sync {
    /// OAuth providers this instance holds credentials for. A provider absent
    /// here cannot complete an authorization; clients use this to disable
    /// Connect affordances instead of failing at click time.
    fn configured_oauth_providers(&self) -> Vec<IntegrationOAuthProvider>;

    fn list_connections(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<Vec<IntegrationConnection>, AppError>>;

    fn pending_jobs_per_connection(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<std::collections::HashMap<IntegrationConnectionId, u32>, AppError>>;

    fn authorize(
        &self,
        user_id: UserId,
        provider: IntegrationOAuthProvider,
        redirect_after: Option<String>,
    ) -> BoxFuture<'_, Result<IntegrationAuthorizeStart, AppError>>;

    fn callback(
        &self,
        provider: IntegrationOAuthProvider,
        code: &str,
        state: &str,
    ) -> BoxFuture<'_, Result<IntegrationConnection, AppError>>;

    fn delete_connection(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
    ) -> BoxFuture<'_, Result<(), AppError>>;

    fn sync_now(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
    ) -> BoxFuture<'_, Result<IntegrationSyncEnqueued, AppError>>;

    fn get_notion_settings(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
    ) -> BoxFuture<'_, Result<NotionExportSettings, AppError>>;

    fn update_notion_settings(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        settings: NotionExportSettings,
    ) -> BoxFuture<'_, Result<NotionExportSettings, AppError>>;

    fn list_notion_export_items(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        query: Option<String>,
        limit: i64,
        offset: i64,
    ) -> BoxFuture<'_, Result<NotionExportItemsPage, AppError>>;

    fn update_notion_export_items(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        selections: Vec<(LibraryEntryId, bool)>,
    ) -> BoxFuture<'_, Result<(), AppError>>;

    fn refresh_notion_export_item(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        library_entry_id: LibraryEntryId,
    ) -> BoxFuture<'_, Result<NotionRefreshEnqueued, AppError>>;

    fn get_obsidian_settings(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
    ) -> BoxFuture<'_, Result<ObsidianExportSettings, AppError>>;

    fn update_obsidian_settings(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        settings: ObsidianExportSettings,
    ) -> BoxFuture<'_, Result<ObsidianExportSettings, AppError>>;

    fn preview_obsidian_export(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        library_entry_id: Option<LibraryEntryId>,
        settings: Option<ObsidianExportSettings>,
    ) -> BoxFuture<'_, Result<ObsidianExportPreview, AppError>>;

    fn setup_obsidian_connection(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<IntegrationConnection, AppError>>;
}

pub trait ExportOperations: Send + Sync {
    fn create_obsidian_run(
        &self,
        user_id: UserId,
        input: ObsidianRunCreate,
    ) -> BoxFuture<'_, Result<ObsidianRunStatus, AppError>>;

    fn get_obsidian_run(
        &self,
        user_id: UserId,
        run_id: uuid::Uuid,
    ) -> BoxFuture<'_, Result<ObsidianRunStatus, AppError>>;

    fn get_obsidian_artifact(
        &self,
        user_id: UserId,
        artifact_id: uuid::Uuid,
    ) -> BoxFuture<'_, Result<ObsidianArtifactDownload, AppError>>;

    fn ack_obsidian_run(
        &self,
        user_id: UserId,
        run_id: uuid::Uuid,
        input: ObsidianRunAck,
    ) -> BoxFuture<'_, Result<ObsidianRunStatus, AppError>>;

    fn refresh_obsidian_subjects(
        &self,
        user_id: UserId,
        input: ObsidianRefreshRequest,
    ) -> BoxFuture<'_, Result<ObsidianRefreshResult, AppError>>;

    fn record_obsidian_path_rename(
        &self,
        user_id: UserId,
        subject_id: LibraryEntryId,
        new_path: String,
    ) -> BoxFuture<'_, Result<(), AppError>>;
}

#[derive(Debug, Clone)]
pub struct ObsidianRunCreate {
    pub parent_folder_deleted: bool,
    pub auto: bool,
    pub force_subject_ids: Vec<LibraryEntryId>,
}

#[derive(Debug, Clone, Default)]
pub struct ObsidianRunAck {
    pub artifact_ids: Vec<uuid::Uuid>,
    pub subjects: Vec<ObsidianAckSubject>,
}

#[derive(Debug, Clone)]
pub struct ObsidianAckSubject {
    pub subject_id: LibraryEntryId,
    pub status: String,
    pub error: Option<String>,
    pub last_content_hash: Option<String>,
    pub last_full_document_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ObsidianRefreshRequest {
    pub subject_ids: Vec<LibraryEntryId>,
    pub reason: String,
}

pub trait EmailSenderOperations: Send + Sync {
    fn list(
        &self,
        user_id: UserId,
        offset: i64,
        limit: i64,
    ) -> BoxFuture<'_, Result<(Vec<EmailSender>, i64), AppError>>;

    fn get(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> BoxFuture<'_, Result<EmailSender, AppError>>;

    fn list_by_ids(
        &self,
        user_id: UserId,
        ids: Vec<EmailSenderId>,
    ) -> BoxFuture<'_, Result<Vec<EmailSender>, AppError>>;

    fn block(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> BoxFuture<'_, Result<EmailSender, AppError>>;

    fn unblock(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> BoxFuture<'_, Result<EmailSender, AppError>>;

    fn set_render_default(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
        value: EmailSenderRenderDefault,
    ) -> BoxFuture<'_, Result<EmailSender, AppError>>;

    fn set_routing_default(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
        value: Option<EmailDestination>,
    ) -> BoxFuture<'_, Result<EmailSender, AppError>>;

    fn unsubscribe(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> BoxFuture<'_, Result<EmailSenderUnsubscribeOutcome, AppError>>;
}

#[derive(Debug, Clone)]
pub struct EmailSenderUnsubscribeOutcome {
    pub sender: EmailSender,
    pub job_id: ind_domain::JobOutboxId,
}

pub trait EmailAliasOperations: Send + Sync {
    fn list(&self, user_id: UserId) -> BoxFuture<'_, Result<Vec<EmailAlias>, AppError>>;

    fn create(
        &self,
        user_id: UserId,
        destination: EmailDestination,
        local_part: String,
        is_default: bool,
    ) -> BoxFuture<'_, Result<EmailAlias, EmailAliasCreateError>>;

    fn delete(
        &self,
        user_id: UserId,
        alias_id: EmailAliasId,
    ) -> BoxFuture<'_, Result<(), AppError>>;
}

#[derive(Debug, thiserror::Error)]
pub enum EmailAliasCreateError {
    #[error("invalid local part: {0}")]
    InvalidLocalPart(ind_domain::AliasLocalPartError),
    #[error("local part collides with another account's seed token")]
    SeedTokenCollision,
    #[error(transparent)]
    Application(#[from] AppError),
}
