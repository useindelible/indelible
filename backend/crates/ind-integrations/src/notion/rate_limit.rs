use tokio::sync::Mutex;

pub struct NotionRateLimiter {
    state: Mutex<RateLimiterState>,
}

struct RateLimiterState {
    tokens: f64,
    last_refill: std::time::Instant,
    rate: f64,
    burst: f64,
}

impl NotionRateLimiter {
    pub fn new(rate_per_second: f64) -> Self {
        Self {
            state: Mutex::new(RateLimiterState {
                tokens: rate_per_second,
                last_refill: std::time::Instant::now(),
                rate: rate_per_second,
                burst: rate_per_second,
            }),
        }
    }

    pub async fn acquire(&self) {
        loop {
            let wait = {
                let mut state = self.state.lock().await;
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(state.last_refill).as_secs_f64();
                state.tokens = (state.tokens + elapsed * state.rate).min(state.burst);
                state.last_refill = now;
                if state.tokens >= 1.0 {
                    state.tokens -= 1.0;
                    return;
                }
                let wait_secs = (1.0 - state.tokens) / state.rate;
                std::time::Duration::from_secs_f64(wait_secs)
            };
            tokio::time::sleep(wait).await;
        }
    }
}
