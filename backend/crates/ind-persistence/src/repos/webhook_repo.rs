use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::webhook::{
    UpdateWebhookEndpointInput, WebhookDeliveryInput, WebhookDispatchContext,
    WebhookProjectionEvent, WebhookRepository,
};
use ind_domain::{
    DomainError, DomainEvent, DomainEventId, UserId, WebhookDelivery, WebhookDeliveryId,
    WebhookDispatch, WebhookDispatchId, WebhookDispatchStatus, WebhookEndpoint, WebhookEndpointId,
};

pub struct PgWebhookRepository {
    pool: PgPool,
}

impl PgWebhookRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct EndpointRow {
    id: Uuid,
    user_id: Uuid,
    name: String,
    url: String,
    secret_hash: String,
    secret_ciphertext: Option<Vec<u8>>,
    secret_preview: String,
    events: Vec<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<EndpointRow> for WebhookEndpoint {
    fn from(row: EndpointRow) -> Self {
        Self {
            id: WebhookEndpointId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            name: row.name,
            url: row.url,
            secret_hash: row.secret_hash,
            secret_ciphertext: row.secret_ciphertext,
            secret_preview: row.secret_preview,
            events: row.events,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

struct DispatchRow {
    id: Uuid,
    domain_event_id: Uuid,
    endpoint_id: Uuid,
    event_type: String,
    status: String,
    first_enqueued_at: Option<DateTime<Utc>>,
    delivered_at: Option<DateTime<Utc>>,
    exhausted_at: Option<DateTime<Utc>>,
    last_error: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<DispatchRow> for WebhookDispatch {
    type Error = AppError;

    fn try_from(row: DispatchRow) -> Result<Self, Self::Error> {
        let status = match row.status.as_str() {
            "pending" => WebhookDispatchStatus::Pending,
            "delivered" => WebhookDispatchStatus::Delivered,
            "exhausted" => WebhookDispatchStatus::Exhausted,
            other => {
                return Err(AppError::Domain(DomainError::InvariantViolation {
                    message: format!("unknown webhook dispatch status {other}"),
                }));
            }
        };

        Ok(Self {
            id: WebhookDispatchId::from_uuid(row.id),
            domain_event_id: DomainEventId::from_uuid(row.domain_event_id),
            endpoint_id: WebhookEndpointId::from_uuid(row.endpoint_id),
            event_type: row.event_type,
            status,
            first_enqueued_at: row.first_enqueued_at,
            delivered_at: row.delivered_at,
            exhausted_at: row.exhausted_at,
            last_error: row.last_error,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

struct DeliveryRow {
    id: Uuid,
    dispatch_id: Uuid,
    domain_event_id: Uuid,
    endpoint_id: Uuid,
    event_type: String,
    payload: serde_json::Value,
    status_code: Option<i32>,
    response_body: Option<String>,
    attempt_number: i32,
    latency_ms: Option<i32>,
    delivered_at: Option<DateTime<Utc>>,
    next_retry_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<DeliveryRow> for WebhookDelivery {
    fn from(row: DeliveryRow) -> Self {
        Self {
            id: WebhookDeliveryId::from_uuid(row.id),
            dispatch_id: WebhookDispatchId::from_uuid(row.dispatch_id),
            domain_event_id: DomainEventId::from_uuid(row.domain_event_id),
            endpoint_id: WebhookEndpointId::from_uuid(row.endpoint_id),
            event_type: row.event_type,
            payload: row.payload,
            status_code: row.status_code,
            response_body: row.response_body,
            attempt_number: row.attempt_number,
            latency_ms: row.latency_ms,
            delivered_at: row.delivered_at,
            next_retry_at: row.next_retry_at,
            created_at: row.created_at,
        }
    }
}

struct EventRow {
    id: Uuid,
    event_type: String,
    aggregate_type: String,
    aggregate_id: Uuid,
    user_id: Uuid,
    payload: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<EventRow> for DomainEvent {
    fn from(row: EventRow) -> Self {
        Self {
            id: DomainEventId::from_uuid(row.id),
            event_type: row.event_type,
            aggregate_type: row.aggregate_type,
            aggregate_id: row.aggregate_id,
            user_id: UserId::from_uuid(row.user_id),
            payload: row.payload,
            created_at: row.created_at,
        }
    }
}

fn map_sqlx(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("webhook", "webhook repository error", err)
}

#[async_trait::async_trait]
impl WebhookRepository for PgWebhookRepository {
    async fn create_endpoint(
        &self,
        endpoint: WebhookEndpoint,
    ) -> Result<WebhookEndpoint, AppError> {
        let row = sqlx::query_as!(
            EndpointRow,
            "INSERT INTO webhook_endpoints (id, user_id, name, url, secret_hash, secret_ciphertext, secret_preview, events, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             RETURNING id, user_id, name, url, secret_hash, secret_ciphertext, secret_preview, events, is_active, created_at, updated_at",
            endpoint.id.into_uuid(),
            endpoint.user_id.into_uuid(),
            endpoint.name,
            endpoint.url,
            endpoint.secret_hash,
            endpoint.secret_ciphertext,
            endpoint.secret_preview,
            &endpoint.events,
            endpoint.is_active,
            endpoint.created_at,
            endpoint.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx)?;

        Ok(row.into())
    }

    async fn list_endpoints(&self, user_id: UserId) -> Result<Vec<WebhookEndpoint>, AppError> {
        let rows = sqlx::query_as!(
            EndpointRow,
            "SELECT id, user_id, name, url, secret_hash, secret_ciphertext, secret_preview, events, is_active, created_at, updated_at \
             FROM webhook_endpoints WHERE user_id = $1 ORDER BY created_at DESC",
            user_id.into_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx)?;

        Ok(rows.into_iter().map(WebhookEndpoint::from).collect())
    }

    async fn find_endpoint(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
    ) -> Result<Option<WebhookEndpoint>, AppError> {
        let row = sqlx::query_as!(
            EndpointRow,
            "SELECT id, user_id, name, url, secret_hash, secret_ciphertext, secret_preview, events, is_active, created_at, updated_at \
             FROM webhook_endpoints WHERE id = $1 AND user_id = $2",
            endpoint_id.into_uuid(),
            user_id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;

        Ok(row.map(WebhookEndpoint::from))
    }

    async fn update_endpoint(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
        input: UpdateWebhookEndpointInput,
    ) -> Result<WebhookEndpoint, AppError> {
        let row = sqlx::query_as!(
            EndpointRow,
            "UPDATE webhook_endpoints SET \
                name = COALESCE($3::text, name), \
                url = COALESCE($4::text, url), \
                events = COALESCE($5::text[], events), \
                is_active = COALESCE($6::bool, is_active), \
                updated_at = $7 \
             WHERE id = $1 AND user_id = $2 \
             RETURNING id, user_id, name, url, secret_hash, secret_ciphertext, secret_preview, events, is_active, created_at, updated_at",
            endpoint_id.into_uuid(),
            user_id.into_uuid(),
            input.name,
            input.url,
            input.events.as_deref(),
            input.is_active,
            Utc::now(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;

        row.map(WebhookEndpoint::from).ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "webhook",
                id: endpoint_id.to_string(),
            })
        })
    }

    async fn update_endpoint_secret(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
        secret_hash: String,
        secret_ciphertext: Vec<u8>,
        secret_preview: String,
    ) -> Result<WebhookEndpoint, AppError> {
        let row = sqlx::query_as!(
            EndpointRow,
            "UPDATE webhook_endpoints SET secret_hash = $3, secret_ciphertext = $4, secret_preview = $5, updated_at = $6 \
             WHERE id = $1 AND user_id = $2 \
             RETURNING id, user_id, name, url, secret_hash, secret_ciphertext, secret_preview, events, is_active, created_at, updated_at",
            endpoint_id.into_uuid(),
            user_id.into_uuid(),
            secret_hash,
            secret_ciphertext,
            secret_preview,
            Utc::now(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;

        row.map(WebhookEndpoint::from).ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "webhook",
                id: endpoint_id.to_string(),
            })
        })
    }

    async fn delete_endpoint(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            "DELETE FROM webhook_endpoints WHERE id = $1 AND user_id = $2",
            endpoint_id.into_uuid(),
            user_id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "webhook",
                id: endpoint_id.to_string(),
            }));
        }

        Ok(())
    }

    async fn list_deliveries(
        &self,
        user_id: UserId,
        endpoint_id: WebhookEndpointId,
        limit: i64,
    ) -> Result<Vec<WebhookDelivery>, AppError> {
        let rows = sqlx::query_as!(
            DeliveryRow,
            "SELECT d.id AS \"id!\", d.dispatch_id AS \"dispatch_id!\", \
                    d.domain_event_id AS \"domain_event_id!\", d.endpoint_id AS \"endpoint_id!\", \
                    d.event_type AS \"event_type!\", d.payload AS \"payload!: serde_json::Value\", \
                    d.status_code, d.response_body, d.attempt_number AS \"attempt_number!\", \
                    d.latency_ms, d.delivered_at, d.next_retry_at, d.created_at AS \"created_at!\" \
             FROM webhook_deliveries d \
             JOIN webhook_endpoints e ON e.id = d.endpoint_id \
             WHERE d.endpoint_id = $1 AND e.user_id = $2 \
             ORDER BY d.created_at DESC LIMIT $3",
            endpoint_id.into_uuid(),
            user_id.into_uuid(),
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx)?;

        Ok(rows.into_iter().map(WebhookDelivery::from).collect())
    }

    async fn create_test_dispatch(
        &self,
        endpoint: &WebhookEndpoint,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<WebhookDispatchContext, AppError> {
        let now = Utc::now();
        let event_id = DomainEventId::new();
        let dispatch_id = WebhookDispatchId::new();

        let mut tx = self.pool.begin().await.map_err(map_sqlx)?;

        let event_row = sqlx::query_as!(
            EventRow,
            "INSERT INTO domain_events (id, event_type, aggregate_type, aggregate_id, user_id, payload, created_at) \
             VALUES ($1, $2, 'webhook_test', $3, $4, $5, $6) \
             RETURNING id, event_type, aggregate_type, aggregate_id, user_id, payload AS \"payload!: serde_json::Value\", created_at",
            event_id.into_uuid(),
            event_type,
            endpoint.id.into_uuid(),
            endpoint.user_id.into_uuid(),
            payload,
            now,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx)?;

        let dispatch_row = sqlx::query_as!(
            DispatchRow,
            "INSERT INTO webhook_dispatches (id, domain_event_id, endpoint_id, event_type, status, first_enqueued_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, 'pending', $5, $5, $5) \
             RETURNING id, domain_event_id, endpoint_id, event_type, status, first_enqueued_at, delivered_at, exhausted_at, last_error, created_at, updated_at",
            dispatch_id.into_uuid(),
            event_id.into_uuid(),
            endpoint.id.into_uuid(),
            event_type,
            now,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx)?;

        tx.commit().await.map_err(map_sqlx)?;

        Ok(WebhookDispatchContext {
            dispatch: dispatch_row.try_into()?,
            endpoint: endpoint.clone(),
            event: event_row.into(),
        })
    }

    async fn get_dispatch_context(
        &self,
        dispatch_id: WebhookDispatchId,
    ) -> Result<Option<WebhookDispatchContext>, AppError> {
        let Some(dispatch_row) = sqlx::query_as!(
            DispatchRow,
            "SELECT id, domain_event_id, endpoint_id, event_type, status, first_enqueued_at, delivered_at, exhausted_at, last_error, created_at, updated_at \
             FROM webhook_dispatches WHERE id = $1",
            dispatch_id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?
        else {
            return Ok(None);
        };

        let endpoint_row = sqlx::query_as!(
            EndpointRow,
            "SELECT id, user_id, name, url, secret_hash, secret_ciphertext, secret_preview, events, is_active, created_at, updated_at \
             FROM webhook_endpoints WHERE id = $1",
            dispatch_row.endpoint_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx)?;

        let event_row = sqlx::query_as!(
            EventRow,
            "SELECT id, event_type, aggregate_type, aggregate_id, user_id, payload AS \"payload!: serde_json::Value\", created_at \
             FROM domain_events WHERE id = $1",
            dispatch_row.domain_event_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx)?;

        Ok(Some(WebhookDispatchContext {
            dispatch: dispatch_row.try_into()?,
            endpoint: endpoint_row.into(),
            event: event_row.into(),
        }))
    }

    async fn record_delivery(
        &self,
        input: WebhookDeliveryInput,
    ) -> Result<WebhookDelivery, AppError> {
        let row = sqlx::query_as!(
            DeliveryRow,
            "INSERT INTO webhook_deliveries (id, dispatch_id, domain_event_id, endpoint_id, event_type, payload, status_code, response_body, attempt_number, latency_ms, delivered_at, next_retry_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             RETURNING id, dispatch_id, domain_event_id, endpoint_id, event_type, payload AS \"payload!: serde_json::Value\", status_code, response_body, attempt_number, latency_ms, delivered_at, next_retry_at, created_at",
            input.id.into_uuid(),
            input.dispatch_id.into_uuid(),
            input.domain_event_id.into_uuid(),
            input.endpoint_id.into_uuid(),
            input.event_type,
            input.payload,
            input.status_code,
            input.response_body,
            input.attempt_number,
            input.latency_ms,
            input.delivered_at,
            input.next_retry_at,
            Utc::now(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx)?;

        Ok(row.into())
    }

    async fn mark_dispatch_delivered(
        &self,
        dispatch_id: WebhookDispatchId,
        delivered_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE webhook_dispatches SET status = 'delivered', delivered_at = $2, exhausted_at = NULL, last_error = NULL, updated_at = $2 WHERE id = $1",
            dispatch_id.into_uuid(),
            delivered_at
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn mark_dispatch_exhausted(
        &self,
        dispatch_id: WebhookDispatchId,
        exhausted_at: DateTime<Utc>,
        last_error: String,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE webhook_dispatches SET status = 'exhausted', exhausted_at = $2, last_error = $3, updated_at = $2 WHERE id = $1",
            dispatch_id.into_uuid(),
            exhausted_at,
            last_error
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn project_next_events(
        &self,
        batch_size: i64,
    ) -> Result<Vec<WebhookProjectionEvent>, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx)?;
        sqlx::query!(
            "INSERT INTO projector_cursors (projector_name, updated_at) \
             VALUES ('webhook_projector', $1) \
             ON CONFLICT (projector_name) DO NOTHING",
            Utc::now()
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx)?;

        let cursor = sqlx::query!(
            "SELECT last_seen_created_at, last_seen_event_id \
             FROM projector_cursors \
             WHERE projector_name = 'webhook_projector' \
             FOR UPDATE",
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx)?;

        let last_seen_event_id: Option<Uuid> = cursor.and_then(|r| r.last_seen_event_id);

        let events = sqlx::query_as!(
            EventRow,
            "SELECT id, event_type, aggregate_type, aggregate_id, user_id, payload AS \"payload!: serde_json::Value\", created_at \
             FROM domain_events \
             WHERE ($1::uuid IS NULL OR id > $1) \
               AND created_at <= now() - interval '500 milliseconds' \
             ORDER BY id ASC LIMIT $2",
            last_seen_event_id,
            batch_size
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(map_sqlx)?;

        let mut projected = Vec::new();
        for row in events {
            let event: DomainEvent = row.into();
            let dispatch_rows = sqlx::query_as!(
                DispatchRow,
                "INSERT INTO webhook_dispatches (id, domain_event_id, endpoint_id, event_type, status, first_enqueued_at, created_at, updated_at) \
                 SELECT gen_random_uuid(), $1, e.id, $2, 'pending', $4, $4, $4 \
                 FROM webhook_endpoints e \
                 WHERE e.user_id = $3 AND e.is_active = true AND $2 = ANY(e.events) \
                 ON CONFLICT (domain_event_id, endpoint_id) DO NOTHING \
                 RETURNING id, domain_event_id, endpoint_id, event_type, status, first_enqueued_at, delivered_at, exhausted_at, last_error, created_at, updated_at",
                event.id.into_uuid(),
                &event.event_type,
                event.user_id.into_uuid(),
                Utc::now(),
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(map_sqlx)?;

            let mut dispatch_ids = Vec::new();
            for dispatch_row in dispatch_rows {
                let dispatch: WebhookDispatch = dispatch_row.try_into()?;
                let payload = serde_json::to_value(ind_domain::WebhookDeliverJob {
                    dispatch_id: dispatch.id,
                })
                .map_err(|e| AppError::Repository(Box::new(e)))?;
                sqlx::query!(
                    "INSERT INTO job_outbox (id, job_type, payload, dedupe_key, available_at, created_at) \
                     VALUES ($1, 'webhook.deliver', $2, $3, $4, $4) \
                     ON CONFLICT (dedupe_key) WHERE dedupe_key IS NOT NULL DO NOTHING",
                    ind_domain::JobOutboxId::new().into_uuid(),
                    payload,
                    format!("webhook.deliver:{}", dispatch.id),
                    Utc::now()
                )
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx)?;
                dispatch_ids.push(dispatch.id);
            }

            sqlx::query!(
                "INSERT INTO projector_cursors (projector_name, last_seen_created_at, last_seen_event_id, updated_at) \
                 VALUES ('webhook_projector', $1, $2, $3) \
                 ON CONFLICT (projector_name) DO UPDATE SET last_seen_created_at = EXCLUDED.last_seen_created_at, last_seen_event_id = EXCLUDED.last_seen_event_id, updated_at = EXCLUDED.updated_at",
                event.created_at,
                event.id.into_uuid(),
                Utc::now()
            )
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx)?;

            projected.push(WebhookProjectionEvent {
                event,
                dispatch_ids,
            });
        }

        tx.commit().await.map_err(map_sqlx)?;
        Ok(projected)
    }
}
