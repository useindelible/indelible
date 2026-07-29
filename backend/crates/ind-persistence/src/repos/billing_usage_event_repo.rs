use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::billing_usage_event::BillingUsageEventRepository;
use ind_domain::{BillingAccountId, BillingUsageEvent, UserId};

pub struct PgBillingUsageEventRepository {
    pool: PgPool,
}

impl PgBillingUsageEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct BillingUsageEventRow {
    id: Uuid,
    user_id: Uuid,
    billing_account_id: Option<Uuid>,
    product_area: String,
    event_type: String,
    provider: Option<String>,
    billing_mode: String,
    resource_type: String,
    resource_id: Option<Uuid>,
    units: serde_json::Value,
    cost_units: i64,
    amount_cents: Option<i32>,
    currency: Option<String>,
    idempotency_key: String,
    metadata: serde_json::Value,
    occurred_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<BillingUsageEventRow> for BillingUsageEvent {
    fn from(row: BillingUsageEventRow) -> Self {
        Self {
            id: row.id,
            user_id: UserId::from_uuid(row.user_id),
            billing_account_id: row.billing_account_id.map(BillingAccountId::from_uuid),
            product_area: row.product_area,
            event_type: row.event_type,
            provider: row.provider,
            billing_mode: row.billing_mode,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            units: row.units,
            cost_units: row.cost_units,
            amount_cents: row.amount_cents,
            currency: row.currency,
            idempotency_key: row.idempotency_key,
            metadata: row.metadata,
            occurred_at: row.occurred_at,
            created_at: row.created_at,
        }
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("billing_usage_event", "usage event already exists", err)
}

#[async_trait::async_trait]
impl BillingUsageEventRepository for PgBillingUsageEventRepository {
    async fn insert(&self, event: &BillingUsageEvent) -> Result<BillingUsageEvent, AppError> {
        let row = sqlx::query_as!(
            BillingUsageEventRow,
            r#"
            INSERT INTO billing_usage_events (
                id, user_id, billing_account_id, product_area, event_type, provider,
                billing_mode, resource_type, resource_id, units, cost_units,
                amount_cents, currency, idempotency_key, metadata, occurred_at, created_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11,
                $12, $13, $14, $15, $16, $17
            )
            RETURNING
                id, user_id, billing_account_id, product_area, event_type, provider,
                billing_mode, resource_type, resource_id, units, cost_units,
                amount_cents, currency, idempotency_key, metadata, occurred_at, created_at
            "#,
            event.id,
            event.user_id.into_uuid(),
            event.billing_account_id.map(|id| id.into_uuid()),
            &event.product_area,
            &event.event_type,
            event.provider.as_deref(),
            &event.billing_mode,
            &event.resource_type,
            event.resource_id,
            &event.units,
            event.cost_units,
            event.amount_cents,
            event.currency.as_deref(),
            &event.idempotency_key,
            &event.metadata,
            event.occurred_at,
            event.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.into())
    }
}
