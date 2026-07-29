use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::billing::BillingRepository;
use ind_domain::{
    DomainError, EntitlementSnapshot, Plan, PlanId, PlanSlug, Subscription, SubscriptionId,
    SubscriptionStatus, UserId,
};

pub struct PgBillingRepository {
    pool: PgPool,
}

impl PgBillingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PlanRow {
    id: Uuid,
    slug: String,
    name: String,
    version: i32,
    is_active: bool,
    stripe_price_id_monthly: Option<String>,
    stripe_price_id_annual: Option<String>,
    entitlements: serde_json::Value,
    quotas: serde_json::Value,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct SubscriptionRow {
    id: Uuid,
    billing_account_id: Uuid,
    plan_id: Uuid,
    stripe_subscription_id: Option<String>,
    status: String,
    current_period_start: Option<DateTime<Utc>>,
    current_period_end: Option<DateTime<Utc>>,
    cancel_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct EntitlementSnapshotRow {
    id: Uuid,
    user_id: Uuid,
    effective_plan_id: Uuid,
    entitlements: serde_json::Value,
    quotas: serde_json::Value,
    computed_at: DateTime<Utc>,
}

#[async_trait::async_trait]
impl BillingRepository for PgBillingRepository {
    async fn find_plan_by_slug(&self, slug: &PlanSlug) -> Result<Option<Plan>, AppError> {
        let row = sqlx::query_as!(
            PlanRow,
            r#"
            SELECT
                id,
                slug,
                name,
                version,
                is_active,
                stripe_price_id_monthly,
                stripe_price_id_annual,
                entitlements,
                quotas,
                created_at
            FROM plans
            WHERE slug = $1
            "#,
            format_plan_slug(*slug),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        row.map(|row| {
            Ok(Plan {
                id: PlanId::from_uuid(row.id),
                slug: parse_plan_slug(&row.slug)?,
                name: row.name,
                version: row.version,
                is_active: row.is_active,
                stripe_price_id_monthly: row.stripe_price_id_monthly,
                stripe_price_id_annual: row.stripe_price_id_annual,
                entitlements: row.entitlements,
                quotas: row.quotas,
                created_at: row.created_at,
            })
        })
        .transpose()
    }

    async fn find_subscription_by_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<Subscription>, AppError> {
        let row = sqlx::query_as!(
            SubscriptionRow,
            r#"
            SELECT
                s.id,
                s.billing_account_id,
                s.plan_id,
                s.stripe_subscription_id,
                s.status,
                s.current_period_start,
                s.current_period_end,
                s.cancel_at,
                s.created_at,
                s.updated_at
            FROM subscriptions s
            JOIN billing_accounts ba ON ba.id = s.billing_account_id
            WHERE ba.owner_user_id = $1
            ORDER BY s.created_at DESC
            LIMIT 1
            "#,
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        row.map(|row| {
            Ok(Subscription {
                id: SubscriptionId::from_uuid(row.id),
                billing_account_id: ind_domain::BillingAccountId::from_uuid(row.billing_account_id),
                plan_id: PlanId::from_uuid(row.plan_id),
                stripe_subscription_id: row.stripe_subscription_id,
                status: parse_subscription_status(&row.status)?,
                current_period_start: row.current_period_start,
                current_period_end: row.current_period_end,
                cancel_at: row.cancel_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        })
        .transpose()
    }

    async fn find_entitlements(
        &self,
        user_id: UserId,
    ) -> Result<Option<EntitlementSnapshot>, AppError> {
        let row = sqlx::query_as!(
            EntitlementSnapshotRow,
            r#"
            SELECT
                id,
                user_id,
                effective_plan_id,
                entitlements,
                quotas,
                computed_at
            FROM entitlement_snapshots
            WHERE user_id = $1
            "#,
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(row.map(|row| EntitlementSnapshot {
            id: row.id,
            user_id: UserId::from_uuid(row.user_id),
            effective_plan_id: PlanId::from_uuid(row.effective_plan_id),
            entitlements: row.entitlements,
            quotas: row.quotas,
            computed_at: row.computed_at,
        }))
    }
}

pub(crate) fn format_plan_slug(value: PlanSlug) -> &'static str {
    match value {
        PlanSlug::CloudFree => "cloud_free",
        PlanSlug::CloudStarter => "cloud_starter",
        PlanSlug::CloudPro => "cloud_pro",
        PlanSlug::CloudPower => "cloud_power",
        PlanSlug::Family => "family",
    }
}

pub(crate) fn parse_plan_slug(value: &str) -> Result<PlanSlug, AppError> {
    match value {
        "cloud_free" => Ok(PlanSlug::CloudFree),
        "cloud_starter" => Ok(PlanSlug::CloudStarter),
        "cloud_pro" => Ok(PlanSlug::CloudPro),
        "cloud_power" => Ok(PlanSlug::CloudPower),
        "family" => Ok(PlanSlug::Family),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("unknown plan slug: {other}"),
        })),
    }
}

pub(crate) fn parse_subscription_status(value: &str) -> Result<SubscriptionStatus, AppError> {
    match value {
        "active" => Ok(SubscriptionStatus::Active),
        "past_due" => Ok(SubscriptionStatus::PastDue),
        "canceled" => Ok(SubscriptionStatus::Canceled),
        "trialing" => Ok(SubscriptionStatus::Trialing),
        "paused" => Ok(SubscriptionStatus::Paused),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("unknown subscription status: {other}"),
        })),
    }
}
