use ind_application::AppError;
use ind_domain::ops::EmailIngestJob;
use ind_integrations::email::{InboundEmail, gmail_confirm};
use tracing::{info, warn};

pub(super) fn extract_gmail_confirmation_url(email: &InboundEmail) -> Option<String> {
    email
        .text_body
        .as_deref()
        .and_then(gmail_confirm::extract_confirmation_url)
        .or_else(|| {
            email
                .html_body
                .as_deref()
                .and_then(gmail_confirm::extract_confirmation_url)
        })
}

pub(super) fn gmail_confirmation_submission_succeeded(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.is_redirection()
}

/// Auto-completes a Gmail "Forwarding Confirmation" handshake: extracts the
/// `mail-settings.google.com` URL embedded in the message, submits it server-side,
/// and marks the ingest log row so the email never lands as a regular item.
/// Failed submissions return an error so the job can retry or land in failed
/// state instead of being incorrectly marked handled.
pub(super) async fn handle_gmail_confirmation(
    job: &EmailIngestJob,
    email: &InboundEmail,
    ingest_log_repo: &dyn ind_application::repos::email_ingest::EmailIngestLogRepository,
) -> Result<(), AppError> {
    let Some(url) = extract_gmail_confirmation_url(email) else {
        let message = "gmail confirmation detected but no mail-settings.google.com URL found";
        warn!(
            ingest_log_id = ?job.ingest_log_id,
            error = message,
            "gmail confirmation cannot be submitted"
        );
        if let Some(log_id) = job.ingest_log_id {
            ingest_log_repo.mark_failed(log_id, message).await?;
        }
        return Err(AppError::Domain(
            ind_domain::DomainError::InvariantViolation {
                message: message.to_string(),
            },
        ));
    };

    let Some(submit_url) = gmail_confirm::confirmation_submit_url(&url) else {
        let message = "gmail confirmation URL could not be converted to submit endpoint";
        warn!(
            ingest_log_id = ?job.ingest_log_id,
            error = message,
            "gmail confirmation cannot be submitted"
        );
        if let Some(log_id) = job.ingest_log_id {
            ingest_log_repo.mark_failed(log_id, message).await?;
        }
        return Err(AppError::Domain(
            ind_domain::DomainError::InvariantViolation {
                message: message.to_string(),
            },
        ));
    };

    // Do not follow redirects: the token endpoint is the only URL we
    // intentionally submit to, and the response status is enough for logs.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| AppError::ExternalService {
            service: "gmail_confirmation".to_string(),
            message: format!("failed to build HTTP client: {e}"),
        })?;

    let response =
        client
            .post(&submit_url)
            .send()
            .await
            .map_err(|e| AppError::ExternalService {
                service: "gmail_confirmation".to_string(),
                message: format!("submission request failed: {e}"),
            })?;
    let status = response.status();

    if !gmail_confirmation_submission_succeeded(status) {
        warn!(
            ingest_log_id = ?job.ingest_log_id,
            status = %status,
            "gmail forwarding confirmation submission failed"
        );
        return Err(AppError::ExternalService {
            service: "gmail_confirmation".to_string(),
            message: format!("submission returned {status}"),
        });
    }

    info!(
        ingest_log_id = ?job.ingest_log_id,
        status = %status,
        "gmail forwarding confirmation submitted"
    );

    if let Some(log_id) = job.ingest_log_id {
        ingest_log_repo.mark_gmail_confirmation(log_id).await?;
    }
    Ok(())
}
