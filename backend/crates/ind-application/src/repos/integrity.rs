use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityStats {
    pub documents_missing_search_rows: i64,
    pub documents_missing_vectors: i64,
    pub failed_derived_assets: i64,
    pub dead_letter_jobs: i64,
}

#[async_trait]
pub trait IntegrityStatsRepository: Send + Sync {
    async fn stats(&self) -> Result<IntegrityStats, AppError>;
}
