use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use ind_application::AppError;
use ind_application::ports::WebhookOperations;
use ind_application::repos::webhook::{
    UpdateWebhookEndpointInput, WebhookDeliveryInput, WebhookDispatchContext, WebhookRepository,
};
use ind_application::webhooks::payload_for_event;
use ind_auth::webhooks::sign_webhook;
use ind_auth::{
    CredentialCipher, generate_webhook_secret, open_webhook_secret, seal_webhook_secret,
};
use ind_domain::{DomainError, UserId, WebhookDelivery, WebhookEndpoint, WebhookEndpointId};
use ind_egress::{
    EgressPolicy, GuardedClientOptions, GuardedHttpClient, UrlRules, build_guarded_client,
};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Build the guarded HTTP client used for webhook delivery: https-only, no
/// redirects (receivers must not redirect us), with the webhook-surface egress
/// policy so SSRF re-validation happens at delivery time against the resolved
/// IP — closing the DNS-rebinding window between creation and delivery.
pub fn build_webhook_http_client(
    policy: EgressPolicy,
) -> Result<GuardedHttpClient, ind_egress::EgressError> {
    build_guarded_client(
        GuardedClientOptions::new(UrlRules::webhook(), policy)
            .request_timeout(Duration::from_secs(30))
            .max_redirects(0),
    )
}

pub struct WebhookDeliveryService {
    repo: Arc<dyn WebhookRepository>,
    credential_cipher: Option<Arc<CredentialCipher>>,
    http: GuardedHttpClient,
}

impl WebhookDeliveryService {
    pub fn new(
        repo: Arc<dyn WebhookRepository>,
        credential_cipher: Option<Arc<CredentialCipher>>,
        http: GuardedHttpClient,
    ) -> Self {
        Self {
            repo,
            credential_cipher,
            http,
        }
    }

    fn require_cipher(&self) -> Result<&CredentialCipher, AppError> {
        self.credential_cipher
            .as_deref()
            .ok_or_else(|| AppError::ExternalService {
                service: "webhooks".into(),
                message: "auth.credential_key is required for outgoing webhooks".into(),
            })
    }

    fn seal_secret(&self, raw_secret: &str) -> Result<(String, Vec<u8>, String), AppError> {
        Ok(seal_webhook_secret(self.require_cipher()?, raw_secret))
    }

    pub async fn deliver_dispatch(
        &self,
        ctx: WebhookDispatchContext,
        attempt_number: i32,
        terminal_on_failure: bool,
    ) -> Result<WebhookDelivery, AppError> {
        let secret_ciphertext =
            ctx.endpoint
                .secret_ciphertext
                .as_deref()
                .ok_or_else(|| AppError::ExternalService {
                    service: "webhooks".into(),
                    message: "webhook endpoint is missing an encrypted signing secret".into(),
                })?;
        let secret =
            open_webhook_secret(self.require_cipher()?, secret_ciphertext).map_err(|e| {
                AppError::ExternalService {
                    service: "webhooks".into(),
                    message: format!("failed to decrypt webhook secret: {e}"),
                }
            })?;

        let payload = payload_for_event(&ctx.event);
        let payload_value =
            serde_json::to_value(&payload).map_err(|e| AppError::Repository(Box::new(e)))?;
        let body = serde_json::to_vec(&payload).map_err(|e| AppError::Repository(Box::new(e)))?;
        let delivery_id = ind_domain::WebhookDeliveryId::new();
        let timestamp = Utc::now().to_rfc3339();
        let signature = sign_webhook(&secret, &timestamp, &body);
        let started = Instant::now();
        // A guard rejection (private/internal target, e.g. via DNS rebinding) is
        // recorded as a failed delivery rather than crashing the job.
        let (status_code, response_body, last_error) = match self.http.post(&ctx.endpoint.url) {
            Err(egress_err) => {
                let message = format!("delivery blocked: {}", egress_err.client_message());
                (None, Some(message.clone()), message)
            }
            Ok(builder) => {
                let result = builder
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .header("X-Indelible-Event", &ctx.event.event_type)
                    .header("X-Indelible-Delivery", delivery_id.to_string())
                    .header("X-Indelible-Timestamp", &timestamp)
                    .header("X-Indelible-Signature", signature)
                    .body(body)
                    .send()
                    .await;
                match result {
                    Ok(response) => {
                        let status = response.status().as_u16() as i32;
                        let body = response.text().await.unwrap_or_default();
                        let snippet = if body.len() > 4096 {
                            body.chars().take(4096).collect()
                        } else {
                            body
                        };
                        (
                            Some(status),
                            Some(snippet),
                            format!("webhook target returned HTTP {status}"),
                        )
                    }
                    Err(err) => (None, Some(err.to_string()), err.to_string()),
                }
            }
        };
        let latency_ms = started.elapsed().as_millis().min(i32::MAX as u128) as i32;
        let now = Utc::now();

        let delivered_at = if matches!(status_code, Some(200..=299)) {
            Some(now)
        } else {
            None
        };
        let delivery = self
            .repo
            .record_delivery(WebhookDeliveryInput {
                id: delivery_id,
                dispatch_id: ctx.dispatch.id,
                domain_event_id: ctx.event.id,
                endpoint_id: ctx.endpoint.id,
                event_type: ctx.event.event_type.clone(),
                payload: payload_value,
                status_code,
                response_body,
                attempt_number,
                latency_ms: Some(latency_ms),
                delivered_at,
                next_retry_at: None,
            })
            .await?;

        if delivered_at.is_some() {
            self.repo
                .mark_dispatch_delivered(ctx.dispatch.id, now)
                .await?;
        } else if terminal_on_failure {
            self.repo
                .mark_dispatch_exhausted(ctx.dispatch.id, now, last_error)
                .await?;
        }

        Ok(delivery)
    }
}

impl WebhookOperations for WebhookDeliveryService {
    fn list_endpoints(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<Vec<WebhookEndpoint>, AppError>> {
        Box::pin(self.repo.list_endpoints(user_id))
    }

    fn create_endpoint(
        &self,
        user_id: UserId,
        name: String,
        url: String,
        events: Vec<String>,
        is_active: bool,
    ) -> BoxFuture<'_, Result<(WebhookEndpoint, String), AppError>> {
        Box::pin(async move {
            let raw_secret = generate_webhook_secret();
            let (secret_hash, secret_ciphertext, secret_preview) = self.seal_secret(&raw_secret)?;
            let now = Utc::now();
            let endpoint = WebhookEndpoint {
                id: WebhookEndpointId::new(),
                user_id,
                name,
                url,
                secret_hash,
                secret_ciphertext: Some(secret_ciphertext),
                secret_preview,
                events,
                is_active,
                created_at: now,
                updated_at: now,
            };
            let saved = self.repo.create_endpoint(endpoint).await?;
            Ok((saved, raw_secret))
        })
    }

    fn update_endpoint(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
        name: Option<String>,
        url: Option<String>,
        events: Option<Vec<String>>,
        is_active: Option<bool>,
    ) -> BoxFuture<'_, Result<WebhookEndpoint, AppError>> {
        Box::pin(self.repo.update_endpoint(
            user_id,
            endpoint_id,
            UpdateWebhookEndpointInput {
                name,
                url,
                events,
                is_active,
            },
        ))
    }

    fn delete_endpoint(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.repo.delete_endpoint(user_id, endpoint_id))
    }

    fn rotate_secret(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
    ) -> BoxFuture<'_, Result<(WebhookEndpoint, String), AppError>> {
        Box::pin(async move {
            let raw_secret = generate_webhook_secret();
            let (secret_hash, secret_ciphertext, secret_preview) = self.seal_secret(&raw_secret)?;
            let endpoint = self
                .repo
                .update_endpoint_secret(
                    user_id,
                    endpoint_id,
                    secret_hash,
                    secret_ciphertext,
                    secret_preview,
                )
                .await?;
            Ok((endpoint, raw_secret))
        })
    }

    fn test_endpoint(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
        event_type: String,
    ) -> BoxFuture<'_, Result<WebhookDelivery, AppError>> {
        Box::pin(async move {
            let endpoint = self
                .repo
                .find_endpoint(user_id, endpoint_id)
                .await?
                .ok_or_else(|| {
                    AppError::Domain(DomainError::NotFound {
                        entity: "webhook",
                        id: endpoint_id.to_string(),
                    })
                })?;
            let ctx = self
                .repo
                .create_test_dispatch(
                    &endpoint,
                    &event_type,
                    serde_json::json!({
                        "test": true,
                        "webhook_id": endpoint.id.to_string(),
                        "event": event_type,
                    }),
                )
                .await?;
            self.deliver_dispatch(ctx, 1, true).await
        })
    }

    fn list_deliveries(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
        limit: i64,
    ) -> BoxFuture<'_, Result<Vec<WebhookDelivery>, AppError>> {
        Box::pin(self.repo.list_deliveries(user_id, endpoint_id, limit))
    }
}
