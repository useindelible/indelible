use crate::error::AppError;
use ind_domain::{EntitlementSnapshot, Plan, PlanSlug, Subscription, UserId};

#[async_trait::async_trait]
pub trait BillingRepository: Send + Sync {
    async fn find_plan_by_slug(&self, slug: &PlanSlug) -> Result<Option<Plan>, AppError>;
    async fn find_subscription_by_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<Subscription>, AppError>;
    async fn find_entitlements(
        &self,
        user_id: UserId,
    ) -> Result<Option<EntitlementSnapshot>, AppError>;
}
