use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::error::AppError;
use ind_application::repos::event::EventRepository;
use ind_application::repos::{Cursor, Page};
use ind_domain::{DomainEvent, DomainEventId, NewDomainEvent, UserId};

use super::write_helpers::{event_row_to_domain, insert_domain_event_tx};

pub struct PgEventRepository {
    pool: PgPool,
}

impl PgEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EventRepository for PgEventRepository {
    async fn append_event(&self, event: NewDomainEvent) -> Result<DomainEvent, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;
        let persisted = insert_domain_event_tx(&mut tx, event).await?;
        tx.commit()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;
        Ok(persisted)
    }

    async fn list_events_after(
        &self,
        user_id: UserId,
        cursor: Option<DomainEventId>,
        limit: u32,
    ) -> Result<Page<DomainEvent>, AppError> {
        let rows = sqlx::query_as!(
            EventRow,
            "SELECT id, event_type, aggregate_type, aggregate_id, user_id, payload, created_at \
             FROM domain_events \
             WHERE user_id = $1 \
               AND ($2::uuid IS NULL OR id > $2) \
             ORDER BY id ASC \
             LIMIT $3",
            user_id.into_uuid(),
            cursor.map(|c| c.into_uuid()),
            limit as i64 + 1,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        let has_more = rows.len() as u32 > limit;
        let take = if has_more { limit as usize } else { rows.len() };

        let items = rows[..take]
            .iter()
            .map(|r| {
                event_row_to_domain(
                    r.id,
                    r.event_type.clone(),
                    r.aggregate_type.clone(),
                    r.aggregate_id,
                    r.user_id,
                    r.payload.clone(),
                    r.created_at,
                )
            })
            .collect();

        let next_cursor = if has_more {
            rows.get(take - 1).map(|r| Cursor(r.id.to_string()))
        } else {
            None
        };

        Ok(Page { items, next_cursor })
    }

    async fn current_tail(
        &self,
        user_id: UserId,
        visible_before: DateTime<Utc>,
        event_types: &[String],
    ) -> Result<Option<DomainEventId>, AppError> {
        let row = sqlx::query!(
            "SELECT id FROM domain_events \
             WHERE user_id = $1 \
               AND created_at <= $2 \
               AND event_type = ANY($3) \
             ORDER BY id DESC \
             LIMIT 1",
            user_id.as_uuid(),
            visible_before,
            event_types,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(row.map(|row| DomainEventId::from_uuid(row.id)))
    }

    async fn drain_events_after(
        &self,
        user_id: UserId,
        cursor: Option<DomainEventId>,
        visible_before: DateTime<Utc>,
        event_types: &[String],
        limit: i64,
    ) -> Result<Vec<DomainEvent>, AppError> {
        let cursor_id = cursor.map(|id| id.into_uuid());
        let rows = sqlx::query!(
            "SELECT id, event_type, aggregate_type, aggregate_id, user_id, payload, created_at \
             FROM domain_events \
             WHERE user_id = $1 \
               AND ($2::uuid IS NULL OR id > $2) \
               AND created_at <= $3 \
               AND event_type = ANY($4) \
             ORDER BY id ASC \
             LIMIT $5",
            user_id.as_uuid(),
            cursor_id,
            visible_before,
            event_types,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                event_row_to_domain(
                    row.id,
                    row.event_type,
                    row.aggregate_type,
                    row.aggregate_id,
                    row.user_id,
                    row.payload,
                    row.created_at,
                )
            })
            .collect())
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    id: Uuid,
    event_type: String,
    aggregate_type: String,
    aggregate_id: Uuid,
    user_id: Uuid,
    payload: serde_json::Value,
    created_at: DateTime<Utc>,
}
