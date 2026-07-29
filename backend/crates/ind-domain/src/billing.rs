use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{BillingAccountId, PlanId, SubscriptionId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanSlug {
    CloudFree,
    CloudStarter,
    CloudPro,
    CloudPower,
    Family,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    PastDue,
    Canceled,
    Trialing,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: PlanId,
    pub slug: PlanSlug,
    pub name: String,
    pub version: i32,
    pub is_active: bool,
    pub stripe_price_id_monthly: Option<String>,
    pub stripe_price_id_annual: Option<String>,
    pub entitlements: serde_json::Value,
    pub quotas: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: SubscriptionId,
    pub billing_account_id: BillingAccountId,
    pub plan_id: PlanId,
    pub stripe_subscription_id: Option<String>,
    pub status: SubscriptionStatus,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementSnapshot {
    pub id: Uuid,
    pub user_id: UserId,
    pub effective_plan_id: PlanId,
    pub entitlements: serde_json::Value,
    pub quotas: serde_json::Value,
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingUsageEvent {
    pub id: Uuid,
    pub user_id: UserId,
    pub billing_account_id: Option<BillingAccountId>,
    pub product_area: String,
    pub event_type: String,
    pub provider: Option<String>,
    pub billing_mode: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub units: serde_json::Value,
    pub cost_units: i64,
    pub amount_cents: Option<i32>,
    pub currency: Option<String>,
    pub idempotency_key: String,
    pub metadata: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
