use chrono::Utc;
use sqlx::{Postgres, Transaction};

use ind_application::AppError;
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::lifecycle_outbox::OutboxEntry;
use ind_domain::{DomainEvent, DomainEventId, JobOutboxId, NewDomainEvent, UserId};

pub(crate) async fn insert_domain_event_tx(
    tx: &mut Transaction<'_, Postgres>,
    event: NewDomainEvent,
) -> Result<DomainEvent, AppError> {
    sqlx::query!(
        "INSERT INTO domain_events \
         (id, event_type, aggregate_type, aggregate_id, user_id, payload, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        event.id.as_uuid(),
        event.event_type.as_str(),
        event.aggregate_type.as_str(),
        event.aggregate_id,
        event.user_id.as_uuid(),
        &event.payload,
        event.created_at,
    )
    .execute(&mut **tx)
    .await
    .map_err(|e| AppError::Repository(Box::new(e)))?;

    Ok(DomainEvent {
        id: event.id,
        event_type: event.event_type,
        aggregate_type: event.aggregate_type,
        aggregate_id: event.aggregate_id,
        user_id: event.user_id,
        payload: event.payload,
        created_at: event.created_at,
    })
}

pub(crate) async fn apply_domain_events_tx(
    tx: &mut Transaction<'_, Postgres>,
    events: Vec<NewDomainEvent>,
) -> Result<Vec<DomainEvent>, AppError> {
    let mut persisted = Vec::with_capacity(events.len());
    for event in events {
        persisted.push(insert_domain_event_tx(tx, event).await?);
    }
    Ok(persisted)
}

pub(crate) async fn enqueue_outbox_tx(
    tx: &mut Transaction<'_, Postgres>,
    outbox: &OutboxEntry,
) -> Result<JobOutboxId, AppError> {
    let id = JobOutboxId::new();
    let now = Utc::now();

    let stored_id = if let Some(dedupe_key) = outbox.dedupe_key.as_deref() {
        // On dedupe conflict the surviving row keeps its original id; RETURNING
        // reports the id callers must reference.
        sqlx::query_scalar!(
            "INSERT INTO job_outbox \
             (id, job_type, payload, dedupe_key, available_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (dedupe_key) WHERE dedupe_key IS NOT NULL DO UPDATE \
                SET payload = EXCLUDED.payload, \
                    available_at = CASE \
                        WHEN job_outbox.dispatched_at IS NULL \
                            THEN LEAST(job_outbox.available_at, EXCLUDED.available_at) \
                        ELSE EXCLUDED.available_at \
                    END, \
                    dispatched_at = NULL \
             RETURNING id",
            id.as_uuid(),
            outbox.job_type.as_str(),
            &outbox.payload,
            dedupe_key,
            outbox.available_at,
            now,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?
    } else {
        sqlx::query_scalar!(
            "INSERT INTO job_outbox \
             (id, job_type, payload, dedupe_key, available_at, created_at) \
             VALUES ($1, $2, $3, NULL, $4, $5) \
             RETURNING id",
            id.as_uuid(),
            outbox.job_type.as_str(),
            &outbox.payload,
            outbox.available_at,
            now,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?
    };

    Ok(JobOutboxId::from_uuid(stored_id))
}

pub(crate) async fn apply_outbox_tx(
    tx: &mut Transaction<'_, Postgres>,
    outbox: &[OutboxEntry],
) -> Result<(), AppError> {
    for entry in outbox {
        enqueue_outbox_tx(tx, entry).await?;
    }
    Ok(())
}

pub(crate) async fn apply_mutation_side_effects_tx(
    tx: &mut Transaction<'_, Postgres>,
    effects: MutationSideEffects,
) -> Result<(), AppError> {
    apply_outbox_tx(tx, &effects.outbox).await?;
    apply_domain_events_tx(tx, effects.events).await?;
    Ok(())
}

pub(crate) fn event_row_to_domain(
    id: uuid::Uuid,
    event_type: String,
    aggregate_type: String,
    aggregate_id: uuid::Uuid,
    user_id: uuid::Uuid,
    payload: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
) -> DomainEvent {
    DomainEvent {
        id: DomainEventId::from_uuid(id),
        event_type,
        aggregate_type,
        aggregate_id,
        user_id: UserId::from_uuid(user_id),
        payload,
        created_at,
    }
}
