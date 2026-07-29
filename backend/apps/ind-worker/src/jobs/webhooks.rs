use chrono::Utc;
use ind_application::AppError;
use ind_domain::{GenericJobEnvelope, WebhookDeliverJob};
use ind_integrations::webhook_delivery::WebhookDeliveryService;

use crate::context::WebhookJobDeps;

pub async fn dispatch_generic_job(
    ctx: &WebhookJobDeps,
    envelope: GenericJobEnvelope,
) -> Result<Option<()>, AppError> {
    if envelope.job_type != "webhook.deliver" {
        return Ok(None);
    }
    let job: WebhookDeliverJob =
        serde_json::from_value(envelope.payload).map_err(|e| AppError::Repository(Box::new(e)))?;
    handle_webhook_delivery(ctx, job).await?;
    Ok(Some(()))
}

pub async fn project_due_webhooks(
    ctx: &WebhookJobDeps,
    batch_size: i64,
) -> Result<usize, AppError> {
    let projected = ctx.webhook_repo.project_next_events(batch_size).await?;
    Ok(projected.into_iter().map(|p| p.dispatch_ids.len()).sum())
}

async fn handle_webhook_delivery(
    ctx: &WebhookJobDeps,
    job: WebhookDeliverJob,
) -> Result<(), AppError> {
    let repo = ctx.webhook_repo.clone();
    let Some(dispatch_ctx) = repo.get_dispatch_context(job.dispatch_id).await? else {
        return Ok(());
    };

    if !dispatch_ctx.endpoint.is_active {
        repo.mark_dispatch_exhausted(
            dispatch_ctx.dispatch.id,
            Utc::now(),
            "webhook endpoint is inactive".into(),
        )
        .await?;
        return Ok(());
    }

    let prior_deliveries = repo
        .list_deliveries(dispatch_ctx.event.user_id, dispatch_ctx.endpoint.id, 50)
        .await?;
    let attempt_number = prior_deliveries
        .iter()
        .filter(|d| d.dispatch_id == dispatch_ctx.dispatch.id)
        .count() as i32
        + 1;

    let delivery_service = WebhookDeliveryService::new(
        repo,
        ctx.credential_cipher.clone(),
        ctx.webhook_http.clone(),
    );
    let delivery = delivery_service
        .deliver_dispatch(dispatch_ctx, attempt_number, attempt_number >= 3)
        .await?;

    if delivery.delivered_at.is_some() || attempt_number >= 3 {
        Ok(())
    } else {
        let message = delivery
            .status_code
            .map(|status| format!("webhook target returned HTTP {status}"))
            .or(delivery.response_body)
            .unwrap_or_else(|| "webhook delivery failed".into());
        Err(AppError::ExternalService {
            service: "webhooks".into(),
            message,
        })
    }
}
