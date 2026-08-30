use std::cmp::Ordering;

use chrono::{DateTime, SubsecRound, Utc};
use sqlx::{Postgres, Transaction};

use ind_application::AppError;
use ind_domain::{
    BasisPoints, DocumentId, DomainError, EventOrigin, NewReadingEvent, ReadingAnchor,
    ReadingEventKind, UserDocumentState, UserId,
};

use super::{UserDocumentStateRow, map_state_error};

pub(super) enum Inserted {
    New {
        effective_at: DateTime<Utc>,
        received_at: DateTime<Utc>,
        /// The stored value, which the server assigned when the caller had no device counter.
        origin_seq: i64,
    },
    Replayed,
}

pub(super) async fn require_owned(
    tx: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    document_id: DocumentId,
) -> Result<bool, AppError> {
    sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1 AND user_id = $2) AS \"owned!\"",
        document_id.into_uuid(),
        user_id.into_uuid(),
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(map_state_error)
}

pub(super) async fn high_water_mark(
    tx: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    document_id: DocumentId,
    origin: &EventOrigin,
) -> Result<Option<i64>, AppError> {
    sqlx::query_scalar!(
        "SELECT max(origin_seq) AS \"max?\" FROM reading_events \
         WHERE user_id = $1 AND document_id = $2 AND origin = $3",
        user_id.into_uuid(),
        document_id.into_uuid(),
        origin.to_string(),
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(map_state_error)
}

/// `timestamptz` keeps microseconds, so a nanosecond-precision client clock would never compare
/// equal to what was stored and an exact retry would read as divergent.
fn pg_precision(at: DateTime<Utc>) -> DateTime<Utc> {
    at.trunc_subsecs(6)
}

fn position_json(event: &NewReadingEvent) -> Result<Option<serde_json::Value>, AppError> {
    event
        .position
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|e| AppError::Repository(Box::new(e)))
}

/// Insert-first so two concurrent identical requests cannot both miss a `SELECT` and race the
/// primary key: `ON CONFLICT DO NOTHING` makes the loser read the stored row and compare. An
/// exact replay is `Replayed`; any difference is a conflict, as is a reused
/// `(origin, origin_seq)` under a new id, which trips the unique constraint.
pub(super) async fn insert_event(
    tx: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    document_id: DocumentId,
    event: &NewReadingEvent,
) -> Result<Inserted, AppError> {
    let position = position_json(event)?;
    let recorded_at = pg_precision(event.recorded_at);
    let origin = event.origin.to_string();
    let cause = event.cause.to_string();
    let asset_kind = event.asset_kind.map(|k| k.to_string());
    let progress = event.progress.map(BasisPoints::get);
    let inserted = sqlx::query!(
        "INSERT INTO reading_events \
            (id, user_id, document_id, origin, origin_seq, event_kind, cause, session_id, \
             attempt, progress_basis_points, position, position_version, asset_kind, \
             active_ms, recorded_at, effective_at) \
         VALUES ($1, $2, $3, $4, COALESCE($5, nextval('reading_events_surface_seq')), \
                 $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, LEAST($15, now())) \
         ON CONFLICT DO NOTHING \
         RETURNING received_at, effective_at, origin_seq",
        event.id.into_uuid(),
        user_id.into_uuid(),
        document_id.into_uuid(),
        origin,
        event.origin_seq,
        event.kind.as_str(),
        cause,
        event.session_id,
        event.attempt,
        progress,
        position,
        event.position_version,
        asset_kind,
        event.active_ms,
        recorded_at,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_state_error)?;
    if let Some(row) = inserted {
        return Ok(Inserted::New {
            effective_at: row.effective_at,
            received_at: row.received_at,
            origin_seq: row.origin_seq,
        });
    }
    // No row under this id means the insert lost on `(origin, origin_seq)` instead: the
    // sequence was reused under a different id, which is a divergence, not a replay.
    let Some(row) = sqlx::query!(
        "SELECT user_id, document_id, origin, origin_seq, event_kind, cause, session_id, \
                attempt, progress_basis_points, position, position_version, asset_kind, \
                active_ms, recorded_at \
         FROM reading_events WHERE id = $1",
        event.id.into_uuid(),
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_state_error)?
    else {
        return Err(AppError::Domain(DomainError::Conflict {
            entity: "ReadingEvent",
            message: format!(
                "origin sequence for event {} is already used by another event",
                event.id
            ),
        }));
    };
    let same = row.user_id == user_id.into_uuid()
        && row.document_id == document_id.into_uuid()
        && row.origin == origin
        && event.origin_seq.is_none_or(|seq| row.origin_seq == seq)
        && row.event_kind == event.kind.as_str()
        && row.cause == cause
        && row.session_id == event.session_id
        && row.attempt == event.attempt
        && row.progress_basis_points == progress
        && row.position == position
        && row.position_version == event.position_version
        && row.asset_kind == asset_kind
        && row.active_ms == event.active_ms
        && row.recorded_at == recorded_at;
    if same {
        Ok(Inserted::Replayed)
    } else {
        Err(AppError::Domain(DomainError::Conflict {
            entity: "ReadingEvent",
            message: format!("event {} already exists with different content", event.id),
        }))
    }
}

/// The current position row, locked for update. Taking the lock before deciding is what makes
/// the decision safe: the previous shape compared sequences inside a nine-branch `ON CONFLICT`
/// predicate, which grew a copy per column and could not express attempt ordering readably.
struct LockedState {
    current_attempt: i16,
    max_progress_percent: Option<i32>,
    finished_at: Option<DateTime<Utc>>,
    position_origin: Option<String>,
    position_origin_seq: Option<i64>,
    position_effective_at: Option<DateTime<Utc>>,
    position_received_at: Option<DateTime<Utc>>,
    position_event_id: Option<uuid::Uuid>,
}

async fn lock_state(
    tx: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    document_id: DocumentId,
) -> Result<Option<LockedState>, AppError> {
    sqlx::query_as!(
        LockedState,
        "SELECT current_attempt, max_progress_percent, finished_at, position_origin, \
                position_origin_seq, position_effective_at, position_received_at, \
                position_event_id \
         FROM user_document_state WHERE user_id = $1 AND document_id = $2 FOR UPDATE",
        user_id.into_uuid(),
        document_id.into_uuid(),
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_state_error)
}

/// Does this event's position replace the stored one?
///
/// A higher `attempt` always wins: starting a reread is a deliberate act, not a regression to
/// be discarded. Within one attempt, the same origin wins only on a higher `origin_seq`, and a
/// different origin wins on `(effective_at, received_at, id)` — the cross-origin tie-break must
/// not apply within an origin, or a lower sequence with a later wall clock would beat it.
fn position_wins(state: &LockedState, event: &Positioned) -> bool {
    if state.position_origin.is_none() || event.attempt > state.current_attempt {
        return true;
    }
    if event.attempt < state.current_attempt {
        return false;
    }
    if state.position_origin.as_deref() == Some(event.origin.as_str()) {
        return Some(event.origin_seq) > state.position_origin_seq;
    }
    (
        Some(event.effective_at),
        Some(event.received_at),
        Some(event.id),
    ) > (
        state.position_effective_at,
        state.position_received_at,
        state.position_event_id,
    )
}

struct Positioned {
    id: uuid::Uuid,
    origin: String,
    origin_seq: i64,
    attempt: i16,
    effective_at: DateTime<Utc>,
    received_at: DateTime<Utc>,
}

/// Applies one event to the current-state row. `max_progress_percent` and the `finished_at`
/// latch are scoped to the attempt, so a reread starts clean rather than inheriting the
/// previous pass's ceiling. `finished_at` is set by the event's kind, never by its number:
/// a reader who stops at 94% because the rest is appendices has finished the book.
pub(super) async fn project(
    tx: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    document_id: DocumentId,
    event: &NewReadingEvent,
    effective_at: DateTime<Utc>,
    received_at: DateTime<Utc>,
    origin_seq: i64,
) -> Result<Option<UserDocumentState>, AppError> {
    if !require_owned(tx, user_id, document_id).await? {
        return Ok(None);
    }
    let position = position_json(event)?;
    let anchor = event.anchor().map(ReadingAnchor::locator);
    let offset = event.offset();
    let percent = event.progress.map(BasisPoints::to_percent);
    let finished = matches!(event.kind, ReadingEventKind::Finished).then_some(received_at);
    let positioned = Positioned {
        id: event.id.into_uuid(),
        origin: event.origin.to_string(),
        origin_seq,
        attempt: event.attempt,
        effective_at,
        received_at,
    };

    let Some(state) = lock_state(tx, user_id, document_id).await? else {
        let row = sqlx::query_as!(
            UserDocumentStateRow,
            "INSERT INTO user_document_state \
                (user_id, document_id, current_attempt, progress_percent, max_progress_percent, \
                 finished_at, chapter_locator, chapter_offset, scroll_position, last_read_at, \
                 position_origin, position_origin_seq, position_effective_at, \
                 position_received_at, position_event_id, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $4, $5, $6, $7, $8, now(), $9, $10, $11, $12, $13, now(), now()) \
             ON CONFLICT (user_id, document_id) DO NOTHING \
             RETURNING user_id, document_id, progress_percent, max_progress_percent, \
                       scroll_position AS \"scroll_position?: serde_json::Value\", chapter_locator, \
                       chapter_offset, last_read_at, finished_at, first_opened_at, last_opened_at, \
                       created_at, updated_at",
            user_id.into_uuid(),
            document_id.into_uuid(),
            event.attempt,
            percent,
            finished,
            anchor,
            offset,
            position,
            positioned.origin,
            origin_seq,
            effective_at,
            received_at,
            positioned.id,
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_state_error)?;
        return match row {
            Some(row) => Ok(Some(row.into_state())),
            // Another writer inserted between the lock attempt and here; re-apply against it.
            None => {
                Box::pin(project(
                    tx,
                    user_id,
                    document_id,
                    event,
                    effective_at,
                    received_at,
                    origin_seq,
                ))
                .await
            }
        };
    };

    let wins = position_wins(&state, &positioned);
    let new_attempt = state.current_attempt.max(event.attempt);
    // Three cases, not two: a higher attempt restarts the ceiling and the latch, the same
    // attempt accumulates them, and a lower one contributes nothing. Folding the stale case in
    // with the same-attempt branch lets an old pass raise the current pass's max and relatch it.
    let (max_progress, finished_at) = match event.attempt.cmp(&state.current_attempt) {
        Ordering::Greater => (percent, finished),
        Ordering::Equal => (
            match (state.max_progress_percent, percent) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            },
            state.finished_at.or(finished),
        ),
        Ordering::Less => (state.max_progress_percent, state.finished_at),
    };

    let row = sqlx::query_as!(
        UserDocumentStateRow,
        "UPDATE user_document_state SET \
            current_attempt = $3, \
            max_progress_percent = $4, \
            finished_at = $5, \
            progress_percent = CASE WHEN $6 THEN $7 ELSE progress_percent END, \
            chapter_locator = CASE WHEN $6 THEN COALESCE($8, chapter_locator) ELSE chapter_locator END, \
            chapter_offset = CASE WHEN $6 THEN COALESCE($9, chapter_offset) ELSE chapter_offset END, \
            scroll_position = CASE WHEN $6 THEN COALESCE($10, scroll_position) ELSE scroll_position END, \
            position_origin = CASE WHEN $6 THEN $11 ELSE position_origin END, \
            position_origin_seq = CASE WHEN $6 THEN $12 ELSE position_origin_seq END, \
            position_effective_at = CASE WHEN $6 THEN $13 ELSE position_effective_at END, \
            position_received_at = CASE WHEN $6 THEN $14 ELSE position_received_at END, \
            position_event_id = CASE WHEN $6 THEN $15 ELSE position_event_id END, \
            last_read_at = now(), \
            updated_at = now() \
         WHERE user_id = $1 AND document_id = $2 \
         RETURNING user_id, document_id, progress_percent, max_progress_percent, \
                   scroll_position AS \"scroll_position?: serde_json::Value\", chapter_locator, \
                   chapter_offset, last_read_at, finished_at, first_opened_at, last_opened_at, \
                   created_at, updated_at",
        user_id.into_uuid(),
        document_id.into_uuid(),
        new_attempt,
        max_progress,
        finished_at,
        wins,
        percent,
        anchor,
        offset,
        position,
        positioned.origin,
        origin_seq,
        effective_at,
        received_at,
        positioned.id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_state_error)?;
    Ok(row.map(UserDocumentStateRow::into_state))
}

pub(super) async fn project_opened(
    tx: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    document_id: DocumentId,
) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO user_document_state \
            (user_id, document_id, first_opened_at, last_opened_at, created_at, updated_at) \
         VALUES ($1, $2, now(), now(), now(), now()) \
         ON CONFLICT (user_id, document_id) DO UPDATE SET \
            first_opened_at = COALESCE(user_document_state.first_opened_at, EXCLUDED.first_opened_at), \
            last_opened_at = GREATEST(user_document_state.last_opened_at, EXCLUDED.last_opened_at), \
            updated_at = now()",
        user_id.into_uuid(),
        document_id.into_uuid(),
    )
    .execute(&mut **tx)
    .await
    .map_err(map_state_error)?;
    Ok(())
}
