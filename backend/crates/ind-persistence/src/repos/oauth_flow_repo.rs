use chrono::{DateTime, Utc};
use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::oauth_flow::OAuthFlowRepository;

pub struct PgOAuthFlowRepository {
    pool: PgPool,
}

impl PgOAuthFlowRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OAuthFlowRepository for PgOAuthFlowRepository {
    async fn insert_strict(
        &self,
        state_hash: &str,
        provider: &str,
        flow_kind: &str,
        sealed_flow: Vec<u8>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO oauth_flows \
                (state_hash, provider, flow_kind, sealed_flow, used_at, expires_at, created_at) \
             VALUES ($1, $2, $3, $4, NULL, $5, $6)",
            state_hash,
            provider,
            flow_kind,
            sealed_flow,
            expires_at,
            Utc::now(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(())
    }

    async fn upsert(
        &self,
        state_hash: &str,
        provider: &str,
        flow_kind: &str,
        sealed_flow: Vec<u8>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO oauth_flows \
                (state_hash, provider, flow_kind, sealed_flow, used_at, expires_at, created_at) \
             VALUES ($1, $2, $3, $4, NULL, $5, $6) \
             ON CONFLICT (state_hash) DO UPDATE SET \
                provider = EXCLUDED.provider, \
                flow_kind = EXCLUDED.flow_kind, \
                sealed_flow = EXCLUDED.sealed_flow, \
                used_at = NULL, \
                expires_at = EXCLUDED.expires_at, \
                created_at = EXCLUDED.created_at",
            state_hash,
            provider,
            flow_kind,
            sealed_flow,
            expires_at,
            Utc::now(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(())
    }

    async fn consume(&self, state_hash: &str) -> Result<Option<Vec<u8>>, AppError> {
        let row = sqlx::query!(
            "UPDATE oauth_flows SET used_at = now() \
             WHERE state_hash = $1 AND used_at IS NULL AND expires_at > now() \
             RETURNING sealed_flow",
            state_hash,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(row.map(|row| row.sealed_flow))
    }

    async fn consume_scoped(
        &self,
        state_hash: &str,
        provider: &str,
        flow_kind: &str,
    ) -> Result<Option<Vec<u8>>, AppError> {
        let row = sqlx::query!(
            "UPDATE oauth_flows SET used_at = now() \
             WHERE state_hash = $1 AND provider = $2 AND flow_kind = $3 \
               AND used_at IS NULL AND expires_at > now() \
             RETURNING sealed_flow",
            state_hash,
            provider,
            flow_kind,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(row.map(|row| row.sealed_flow))
    }
}
