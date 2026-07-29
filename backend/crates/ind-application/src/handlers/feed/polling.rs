use chrono::{Duration, Utc};

use ind_domain::ActiveSubscription;

use super::FeedPollScheduleConfig;

pub fn effective_poll_interval_minutes(
    subscriptions: &[ActiveSubscription],
    schedule: FeedPollScheduleConfig,
) -> i64 {
    let schedule = schedule.normalized();
    let fastest = subscriptions
        .iter()
        .filter_map(|sub| sub.poll_interval_override_minutes.map(i64::from))
        .min()
        .unwrap_or(schedule.default_public_poll_interval_minutes);
    fastest.max(schedule.min_public_poll_interval_minutes)
}
pub fn next_poll_after_success(
    subscriptions: &[ActiveSubscription],
    now: chrono::DateTime<Utc>,
    schedule: FeedPollScheduleConfig,
) -> chrono::DateTime<Utc> {
    now + Duration::minutes(effective_poll_interval_minutes(subscriptions, schedule))
}
pub fn next_poll_after_failure(
    subscriptions: &[ActiveSubscription],
    now: chrono::DateTime<Utc>,
    consecutive_failures: i32,
    schedule: FeedPollScheduleConfig,
) -> chrono::DateTime<Utc> {
    let base_minutes = effective_poll_interval_minutes(subscriptions, schedule);
    let factor = 2_i64.saturating_pow(consecutive_failures.clamp(0, 5) as u32);
    now + Duration::minutes((base_minutes * factor).min(24 * 60))
}
