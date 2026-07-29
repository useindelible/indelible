use chrono::Utc;
use ind_application::error::AppError;
use ind_domain::{
    ContentSource, HighlightLocator, HighlightSourceLocator, NotionExportDocumentJob,
};
use ind_integrations::notion::{HighlightLocation, NotionError};

use crate::context::NotionJobDeps;

pub(super) async fn requeue_after(
    deps: &NotionJobDeps,
    job: &NotionExportDocumentJob,
    retry_after_secs: u64,
) -> Result<(), AppError> {
    deps.export_cursor_repo
        .mark_attempted(
            job.connection_id,
            job.library_entry_id,
            Utc::now(),
            Some(format!("rate limited; requeued after {retry_after_secs}s")),
        )
        .await?;
    deps.connection_repo
        .set_last_error(
            job.connection_id,
            job.user_id,
            Some(format!(
                "Notion rate limited; requeued after {retry_after_secs}s"
            )),
        )
        .await?;
    let available_at = Utc::now() + chrono::Duration::seconds(retry_after_secs as i64);
    let payload = serde_json::to_value(job).map_err(|e| AppError::ExternalService {
        service: "notion".into(),
        message: format!("failed to serialize requeue payload: {e}"),
    })?;
    deps.outbox_repo
        .enqueue(
            "integration.notion.export_document",
            payload,
            Some(format!(
                "export:{}:{}",
                job.connection_id.into_uuid(),
                job.library_entry_id.into_uuid()
            )),
            available_at,
        )
        .await?;
    Ok(())
}

pub(super) fn map_notion_error(e: NotionError) -> AppError {
    match e {
        NotionError::RateLimited { .. } => AppError::RateLimited,
        NotionError::Api { status: 401, .. } | NotionError::Api { status: 403, .. } => {
            AppError::Auth
        }
        NotionError::Api { status, body } => AppError::ExternalService {
            service: "notion".into(),
            message: format!("HTTP {status}: {body}"),
        },
        NotionError::Http(e) => AppError::ExternalService {
            service: "notion".into(),
            message: e.to_string(),
        },
        NotionError::Json(e) => AppError::ExternalService {
            service: "notion".into(),
            message: e.to_string(),
        },
        NotionError::State(message) => AppError::ExternalService {
            service: "notion".into(),
            message,
        },
    }
}

pub(super) fn triage_state_to_str(state: ind_domain::TriageState) -> String {
    match state {
        ind_domain::TriageState::Inbox => "inbox",
        ind_domain::TriageState::Later => "later",
        ind_domain::TriageState::Archive => "archive",
    }
    .to_string()
}

pub(super) fn content_source_to_str(source: ContentSource) -> String {
    match source {
        ContentSource::Manual => "manual",
        ContentSource::Extension => "extension",
        ContentSource::ShareSheet => "share_sheet",
        ContentSource::Feed => "feed",
        ContentSource::Email => "email",
        ContentSource::Api => "api",
        ContentSource::Cli => "cli",
        ContentSource::Import => "import",
    }
    .to_string()
}

pub(super) fn highlight_location(
    locator: Option<&HighlightLocator>,
    source_locator: Option<&HighlightSourceLocator>,
) -> Option<HighlightLocation> {
    if let Some(HighlightSourceLocator::WebPageDomRange { url, location, .. }) = source_locator {
        return Some(HighlightLocation {
            label: location.clone(),
            href: Some(url.clone()),
        });
    }
    match locator {
        Some(HighlightLocator::Pdf { page, .. }) => Some(HighlightLocation {
            label: format!("Page {page}"),
            href: None,
        }),
        Some(HighlightLocator::Epub { chapter, .. }) => Some(HighlightLocation {
            label: format!("Chapter {chapter}"),
            href: None,
        }),
        Some(HighlightLocator::Html { start_offset, .. }) => Some(HighlightLocation {
            label: format!("Offset {start_offset}"),
            href: None,
        }),
        None => None,
    }
}
