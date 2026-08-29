mod projection;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::user_document_state::{AppendOutcome, UserDocumentStateRepository};
use ind_domain::{
    BasisPoints, DocumentId, DomainError, EventOrigin, NewReadingEvent, ReadingCause,
    ReadingEventId, ReadingEventKind, ReadingPosition, UserDocumentState, UserId,
};

pub struct PgUserDocumentStateRepository {
    pool: PgPool,
}

impl PgUserDocumentStateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub(super) struct UserDocumentStateRow {
    user_id: uuid::Uuid,
    document_id: uuid::Uuid,
    progress_percent: Option<i32>,
    max_progress_percent: Option<i32>,
    scroll_position: Option<serde_json::Value>,
    chapter_locator: Option<String>,
    chapter_offset: Option<i32>,
    last_read_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
    first_opened_at: Option<DateTime<Utc>>,
    last_opened_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl UserDocumentStateRow {
    pub(super) fn into_state(self) -> UserDocumentState {
        UserDocumentState {
            user_id: UserId::from_uuid(self.user_id),
            document_id: DocumentId::from_uuid(self.document_id),
            progress_percent: self.progress_percent,
            max_progress_percent: self.max_progress_percent,
            scroll_position: self.scroll_position,
            chapter_locator: self.chapter_locator,
            chapter_offset: self.chapter_offset,
            last_read_at: self.last_read_at,
            finished_at: self.finished_at,
            first_opened_at: self.first_opened_at,
            last_opened_at: self.last_opened_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

pub(super) fn map_state_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("user_document_state", "reading event conflict", err)
}

fn not_found(document_id: DocumentId) -> AppError {
    AppError::Domain(DomainError::NotFound {
        entity: "Document",
        id: document_id.to_string(),
    })
}

#[async_trait::async_trait]
impl UserDocumentStateRepository for PgUserDocumentStateRepository {
    async fn find(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<UserDocumentState>, AppError> {
        let row = sqlx::query_as!(
            UserDocumentStateRow,
            "SELECT user_id, document_id, progress_percent, max_progress_percent, \
                    scroll_position AS \"scroll_position?: serde_json::Value\", chapter_locator, \
                    chapter_offset, last_read_at, finished_at, first_opened_at, last_opened_at, \
                    created_at, updated_at \
             FROM user_document_state WHERE user_id = $1 AND document_id = $2",
            user_id.into_uuid(),
            document_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_state_error)?;
        Ok(row.map(UserDocumentStateRow::into_state))
    }

    async fn record_progress(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        progress_percent: i32,
        position: Option<ReadingPosition>,
        origin: EventOrigin,
    ) -> Result<UserDocumentState, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_state_error)?;
        projection::require_owned(&mut tx, user_id, document_id)
            .await?
            .then_some(())
            .ok_or_else(|| not_found(document_id))?;
        let event = NewReadingEvent {
            id: ReadingEventId::new(),
            origin,
            origin_seq: None,
            kind: if progress_percent >= 100 {
                ReadingEventKind::Finished
            } else {
                ReadingEventKind::Progress
            },
            cause: ReadingCause::Reader,
            session_id: None,
            attempt: 1,
            progress: Some(BasisPoints::from_percent(progress_percent)?),
            position,
            asset_kind: None,
            position_version: NewReadingEvent::CURRENT_POSITION_VERSION,
            active_ms: None,
            recorded_at: Utc::now(),
        };
        let inserted = projection::insert_event(&mut tx, user_id, document_id, &event).await?;
        let state = match inserted {
            projection::Inserted::New {
                effective_at,
                received_at,
                origin_seq,
            } => {
                projection::project(
                    &mut tx,
                    user_id,
                    document_id,
                    &event,
                    effective_at,
                    received_at,
                    origin_seq,
                )
                .await?
            }
            projection::Inserted::Replayed => None,
        }
        .ok_or_else(|| not_found(document_id))?;
        tx.commit().await.map_err(map_state_error)?;
        Ok(state)
    }

    async fn append_reading_events(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        events: &[NewReadingEvent],
    ) -> Result<AppendOutcome, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_state_error)?;
        if !projection::require_owned(&mut tx, user_id, document_id).await? {
            return Err(not_found(document_id));
        }
        let mut outcome = AppendOutcome::default();
        for event in events {
            let high_water =
                projection::high_water_mark(&mut tx, user_id, document_id, &event.origin).await?;
            match projection::insert_event(&mut tx, user_id, document_id, event).await? {
                projection::Inserted::Replayed => outcome.replayed += 1,
                projection::Inserted::New {
                    effective_at,
                    received_at,
                    origin_seq,
                } => {
                    outcome.accepted += 1;
                    let behind_own_watermark = high_water.is_some_and(|hwm| origin_seq <= hwm);
                    if event.kind == ReadingEventKind::Opened {
                        projection::project_opened(&mut tx, user_id, document_id).await?;
                    } else if !behind_own_watermark {
                        projection::project(
                            &mut tx,
                            user_id,
                            document_id,
                            event,
                            effective_at,
                            received_at,
                            origin_seq,
                        )
                        .await?;
                    }
                }
            }
        }
        tx.commit().await.map_err(map_state_error)?;
        Ok(outcome)
    }

    async fn record_document_opened(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_state_error)?;
        projection::project_opened(&mut tx, user_id, document_id).await?;
        tx.commit().await.map_err(map_state_error)
    }
}
