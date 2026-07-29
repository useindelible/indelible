use std::sync::Arc;

use chrono::{Duration, Timelike, Utc};
use ind_application::AppError;
use ind_application::repos::billing::BillingRepository;
use ind_application::repos::usage_counter::UsageCounterRepository;
use ind_domain::{SearchRateLimitStatus, UserId};

#[derive(Debug, Clone, Copy)]
pub struct SearchRateLimitDefaults {
    pub search_per_minute: u32,
    pub suggestions_per_minute: u32,
    pub recent_search_limit: i64,
}

impl Default for SearchRateLimitDefaults {
    fn default() -> Self {
        Self {
            search_per_minute: 60,
            suggestions_per_minute: 120,
            recent_search_limit: 50,
        }
    }
}

pub struct SearchRateLimiter {
    billing_repo: Arc<dyn BillingRepository>,
    usage_repo: Arc<dyn UsageCounterRepository>,
    defaults: SearchRateLimitDefaults,
}

impl SearchRateLimiter {
    pub fn new(
        billing_repo: Arc<dyn BillingRepository>,
        usage_repo: Arc<dyn UsageCounterRepository>,
        defaults: SearchRateLimitDefaults,
    ) -> Self {
        Self {
            billing_repo,
            usage_repo,
            defaults,
        }
    }

    pub async fn consume_search(&self, user_id: UserId) -> Result<SearchRateLimitStatus, AppError> {
        self.consume(
            user_id,
            "search_requests_per_minute",
            self.defaults.search_per_minute,
        )
        .await
    }

    pub async fn consume_suggestions(
        &self,
        user_id: UserId,
    ) -> Result<SearchRateLimitStatus, AppError> {
        self.consume(
            user_id,
            "search_suggestions_per_minute",
            self.defaults.suggestions_per_minute,
        )
        .await
    }

    async fn consume(
        &self,
        user_id: UserId,
        quota_name: &str,
        default_limit: u32,
    ) -> Result<SearchRateLimitStatus, AppError> {
        let quota_limit = self
            .billing_repo
            .find_entitlements(user_id)
            .await?
            .as_ref()
            .and_then(|snapshot| quota_from_json(&snapshot.quotas, quota_name))
            .unwrap_or(default_limit as i64)
            .max(0);

        let now = Utc::now();
        #[expect(
            clippy::expect_used,
            reason = "second 0 / nanosecond 0 are always valid; flooring to the minute cannot fail"
        )]
        let period_start = now
            .with_second(0)
            .and_then(|value| value.with_nanosecond(0))
            .expect("minute floor");
        let period_end = period_start + Duration::minutes(1);
        let usage = self
            .usage_repo
            .increment_window(user_id, quota_name, period_start, period_end, quota_limit)
            .await?;

        let allowed = usage.current_value <= usage.limit_value;
        let remaining = (usage.limit_value - usage.current_value).max(0) as u32;
        let retry_after_secs =
            (!allowed).then(|| (usage.period_end - now).num_seconds().max(1) as u64);

        Ok(SearchRateLimitStatus {
            allowed,
            quota_name: quota_name.to_string(),
            limit: usage.limit_value.max(0) as u32,
            remaining,
            reset_at: usage.period_end,
            retry_after_secs,
        })
    }
}

fn quota_from_json(value: &serde_json::Value, quota_name: &str) -> Option<i64> {
    value
        .get(quota_name)
        .and_then(|value| value.as_i64())
        .or_else(|| {
            value.get("search").and_then(|search| match quota_name {
                "search_requests_per_minute" => search
                    .get("requests_per_minute")
                    .and_then(|value| value.as_i64()),
                "search_suggestions_per_minute" => search
                    .get("suggestions_per_minute")
                    .and_then(|value| value.as_i64()),
                _ => None,
            })
        })
}
