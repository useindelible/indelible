use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Per-job-type concurrency limiter using semaphores.
pub struct ConcurrencyLimiter {
    semaphores: HashMap<String, Arc<Semaphore>>,
    default_semaphore: Arc<Semaphore>,
}

impl Default for ConcurrencyLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl ConcurrencyLimiter {
    pub fn new() -> Self {
        let mut semaphores = HashMap::new();

        semaphores.insert("document.ai.embed".into(), Arc::new(Semaphore::new(5)));
        semaphores.insert("document.ai.summarize".into(), Arc::new(Semaphore::new(3)));
        semaphores.insert("document.ai.tags".into(), Arc::new(Semaphore::new(3)));
        semaphores.insert("document.ai.entities".into(), Arc::new(Semaphore::new(3)));
        semaphores.insert("webhook.deliver".into(), Arc::new(Semaphore::new(5)));

        Self {
            semaphores,
            default_semaphore: Arc::new(Semaphore::new(5)),
        }
    }

    /// Register (or override) the concurrency limit for a job type. Used to wire config-driven
    /// limits such as feed prefetch `max_concurrency`. A zero is clamped to 1.
    pub fn with_limit(mut self, job_type: impl Into<String>, permits: usize) -> Self {
        self.semaphores
            .insert(job_type.into(), Arc::new(Semaphore::new(permits.max(1))));
        self
    }

    /// Acquire a permit for the given job type. Returns when a slot is available.
    /// Unknown job types share the default semaphore so they are still bounded.
    #[expect(
        clippy::expect_used,
        reason = "the limiter owns its semaphores and never closes them; acquire only errors on a closed semaphore"
    )]
    pub async fn acquire(&self, job_type: &str) -> OwnedSemaphorePermit {
        let sem = self
            .semaphores
            .get(job_type)
            .cloned()
            .unwrap_or_else(|| self.default_semaphore.clone());

        sem.acquire_owned()
            .await
            .expect("semaphore closed unexpectedly")
    }
}
