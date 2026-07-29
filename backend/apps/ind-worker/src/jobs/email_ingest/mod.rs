mod assets;
mod gmail;
mod library;
mod sender;

use ind_application::AppError;
use ind_application::services::email_ingest::{
    EmailIngestService, FeedRouteInput, generate_excerpt,
};
use ind_domain::GenericJobEnvelope;
use ind_domain::ops::EmailIngestJob;
use ind_integrations::email::{
    ContentMode, WebhookMetadata, detect_content_mode, extract_primary_url, gmail_confirm,
};
use tracing::{info, warn};

use gmail::handle_gmail_confirmation;
use library::{
    EmailLibraryInput, LibraryEmailIngestAction, apply_no_link_found_tag,
    decide_library_email_ingest_action, email_origin_key, enqueue_url_save, save_email_as_document,
};
use sender::resolve_sender_id;
pub use sender::{bump_sender_delivery_count, persist_unsubscribe_targets};

use crate::context::EmailJobDeps;

pub async fn dispatch_generic_job(
    ctx: &EmailJobDeps,
    envelope: GenericJobEnvelope,
) -> Result<Option<()>, AppError> {
    match envelope.job_type.as_str() {
        "email.ingest" => {
            let job: EmailIngestJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::Repository(Box::new(e)))?;
            handle_email_ingest(ctx, job).await?;
            Ok(Some(()))
        }
        _ => Ok(None),
    }
}

async fn handle_email_ingest(ctx: &EmailJobDeps, job: EmailIngestJob) -> Result<(), AppError> {
    let provider = ctx.email_ingest_provider.as_ref().ok_or_else(|| {
        AppError::Domain(ind_domain::DomainError::InvariantViolation {
            message: "email ingest provider not configured".to_string(),
        })
    })?;

    let ingest_log_repo = ctx.email_ingest_log_repo.as_ref().ok_or_else(|| {
        AppError::Domain(ind_domain::DomainError::InvariantViolation {
            message: "email ingest log repo not configured".to_string(),
        })
    })?;

    let feed_repo = ctx.feed_repo.clone();
    let user_repo = ctx.user_repo.as_ref().ok_or_else(|| {
        AppError::Domain(ind_domain::DomainError::InvariantViolation {
            message: "user repo not configured".to_string(),
        })
    })?;

    let metadata = WebhookMetadata {
        provider_email_id: job.provider_email_id.clone(),
        to_addresses: Vec::new(),
        from_address: String::new(),
        list_id: None,
    };

    let email = match provider
        .resolve_full_email(&metadata, &job.raw_payload)
        .await
    {
        Ok(e) => e,
        Err(e) => {
            let error_msg = format!("failed to resolve full email: {e}");
            let is_transient =
                matches!(e, ind_integrations::email::EmailIngestError::ProviderApi(_));
            warn!(
                ingest_log_id = ?job.ingest_log_id,
                error = %e,
                transient = is_transient,
                "email resolution failed"
            );
            if !is_transient && let Some(log_id) = job.ingest_log_id {
                ingest_log_repo.mark_failed(log_id, &error_msg).await?;
            }
            return Err(if is_transient {
                AppError::ExternalService {
                    service: "email_provider".to_string(),
                    message: error_msg,
                }
            } else {
                AppError::Domain(ind_domain::DomainError::InvariantViolation { message: error_msg })
            });
        }
    };

    if gmail_confirm::is_gmail_confirmation(&email.from_address, &email.subject) {
        handle_gmail_confirmation(&job, &email, ingest_log_repo.as_ref()).await?;
        return Ok(());
    }

    let service = EmailIngestService::new(
        user_repo.clone(),
        feed_repo,
        ctx.feed_delivery_repo.clone(),
        ingest_log_repo.clone(),
    );

    persist_unsubscribe_targets(ctx, job.user_id, &email).await?;

    let sender_id = resolve_sender_id(ctx, job.user_id, &email.from_address).await?;

    let mode = detect_content_mode(email.text_body.as_deref(), email.html_body.as_deref());
    let guid = email
        .message_id
        .as_deref()
        .unwrap_or(&job.provider_email_id);

    let origin_key = email_origin_key(
        email.message_id.as_deref(),
        &job.provider,
        &job.provider_email_id,
    );
    let html_text = match email.text_body.as_deref() {
        Some(_) => None,
        None => email.html_body.as_deref().map(ind_html::html_to_text),
    };
    let language_body = email
        .text_body
        .as_deref()
        .or(html_text.as_deref())
        .unwrap_or_default();
    let language =
        ind_application::classify_search_language(None, &[email.subject.as_str(), language_body])
            .language;

    match job.destination.as_str() {
        "feed" => {
            let excerpt = email
                .text_body
                .as_deref()
                .and_then(|t| generate_excerpt(Some(t), 300));

            let feed_input = FeedRouteInput {
                user_id: job.user_id,
                from_address: &email.from_address,
                from_display_name: email.from_display_name.as_deref(),
                subject: &email.subject,
                guid,
                content_html: email.html_body.as_deref(),
                excerpt: excerpt.as_deref(),
                language: language.as_deref(),
            };

            match mode {
                ContentMode::ModeA => {
                    service.route_to_feed(&feed_input).await?;
                }
                ContentMode::ModeB => {
                    if let Some(url) =
                        extract_primary_url(email.html_body.as_deref(), email.text_body.as_deref())
                    {
                        enqueue_url_save(ctx, job.user_id, &url, &email.subject).await?;
                    }
                    service.route_to_feed(&feed_input).await?;
                }
            }
        }
        "library" => {
            let excerpt = email
                .text_body
                .as_deref()
                .and_then(|t| generate_excerpt(Some(t), 300));
            let input = EmailLibraryInput {
                user_id: job.user_id,
                subject: &email.subject,
                author: email.from_display_name.as_deref(),
                from_address: &email.from_address,
                content_html: email.html_body.as_deref(),
                text_body: email.text_body.as_deref(),
                excerpt: excerpt.as_deref(),
                language: language.as_deref(),
                origin_key: &origin_key,
                sender_id,
            };
            match decide_library_email_ingest_action(
                mode,
                email.html_body.as_deref(),
                email.text_body.as_deref(),
            ) {
                LibraryEmailIngestAction::EmailBody => {
                    save_email_as_document(ctx, input).await?;
                }
                LibraryEmailIngestAction::ExtractedUrl { url } => {
                    enqueue_url_save(ctx, job.user_id, &url, &email.subject).await?;
                }
                LibraryEmailIngestAction::EmailBodyNoLinkFound => {
                    warn!(
                        ingest_log_id = ?job.ingest_log_id,
                        "Mode B library email but no URL found, falling back to Mode A"
                    );
                    let outcome = save_email_as_document(ctx, input).await?;
                    apply_no_link_found_tag(ctx, job.user_id, outcome.entry.id).await?;
                }
            }
        }
        other => {
            warn!(destination = other, "unknown email destination");
        }
    }

    if let Some(log_id) = job.ingest_log_id {
        ingest_log_repo.mark_processed(log_id).await?;
    }
    bump_sender_delivery_count(ctx, sender_id).await?;
    info!(
        ingest_log_id = ?job.ingest_log_id,
        user_id = %job.user_id,
        destination = %job.destination,
        "email ingest job processed successfully"
    );

    Ok(())
}
