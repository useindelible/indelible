use crate::error::AppError;
use ind_domain::BillingUsageEvent;

#[async_trait::async_trait]
pub trait BillingUsageEventRepository: Send + Sync {
    async fn insert(&self, event: &BillingUsageEvent) -> Result<BillingUsageEvent, AppError>;
}
