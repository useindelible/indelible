use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::routing::{get, patch, post};
use chrono::{DateTime, Utc};
use http::{HeaderMap, StatusCode};
use ind_application::ports::WebhookOperations;
use ind_application::webhooks::is_known_webhook_event;
use ind_domain::{WebhookDelivery, WebhookEndpoint, WebhookEndpointId};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use utoipa::ToSchema;

use ind_application::repos::email_ingest::ClaimAndEnqueueInput;
use ind_domain::ops::{EmailIngestJob, job_types};
use ind_integrations::email::parse_ingest_address;

use crate::error::{ApiError, FieldError};
use crate::extract::Json;
use crate::middleware::AccountAccess;
use crate::response::{ApiResponse, EmptyResponse};
use crate::state::AppState;

mod dto;
pub(crate) mod handlers;
mod helpers;
mod providers;

pub use dto::*;
use handlers::*;
use providers::*;

pub fn webhook_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/webhooks",
            get(list_webhook_endpoints).post(create_webhook_endpoint),
        )
        .route(
            "/api/v1/webhooks/{webhook_id}",
            patch(update_webhook_endpoint).delete(delete_webhook_endpoint),
        )
        .route(
            "/api/v1/webhooks/{webhook_id}/rotate-secret",
            post(rotate_webhook_secret),
        )
        .route(
            "/api/v1/webhooks/{webhook_id}/test",
            post(test_webhook_endpoint),
        )
        .route(
            "/api/v1/webhooks/{webhook_id}/deliveries",
            get(list_webhook_deliveries),
        )
        .route(
            "/api/v1/integrations/webhooks/email-ingest/resend",
            post(resend_webhook_handler),
        )
}
