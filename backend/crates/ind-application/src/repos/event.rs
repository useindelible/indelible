use crate::error::AppError;
use crate::repos::Page;
use chrono::{DateTime, Utc};
use ind_domain::{DomainEvent, DomainEventId, NewDomainEvent, UserId};

use super::lifecycle_outbox::OutboxEntry;

#[derive(Debug, Clone, Default)]
pub struct MutationSideEffects {
    pub events: Vec<NewDomainEvent>,
    pub outbox: Vec<OutboxEntry>,
}

impl MutationSideEffects {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn with_event(event: NewDomainEvent) -> Self {
        Self {
            events: vec![event],
            outbox: vec![],
        }
    }

    pub fn with_events(events: Vec<NewDomainEvent>) -> Self {
        Self {
            events,
            outbox: vec![],
        }
    }

    pub fn with_outbox(outbox: OutboxEntry) -> Self {
        Self {
            events: vec![],
            outbox: vec![outbox],
        }
    }

    pub fn with_event_and_outbox(event: NewDomainEvent, outbox: OutboxEntry) -> Self {
        Self {
            events: vec![event],
            outbox: vec![outbox],
        }
    }

    pub fn push_event(&mut self, event: NewDomainEvent) {
        self.events.push(event);
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty() && self.outbox.is_empty()
    }
}

#[async_trait::async_trait]
pub trait EventRepository: Send + Sync {
    async fn append_event(&self, event: NewDomainEvent) -> Result<DomainEvent, AppError>;

    async fn list_events_after(
        &self,
        user_id: UserId,
        cursor: Option<DomainEventId>,
        limit: u32,
    ) -> Result<Page<DomainEvent>, AppError>;

    async fn current_tail(
        &self,
        user_id: UserId,
        visible_before: DateTime<Utc>,
        event_types: &[String],
    ) -> Result<Option<DomainEventId>, AppError>;

    async fn drain_events_after(
        &self,
        user_id: UserId,
        cursor: Option<DomainEventId>,
        visible_before: DateTime<Utc>,
        event_types: &[String],
        limit: i64,
    ) -> Result<Vec<DomainEvent>, AppError>;
}
