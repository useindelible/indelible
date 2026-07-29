use chrono::{DateTime, Utc};
use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::user_document_state::UserDocumentStateRepository;
use ind_domain::{DocumentId, DomainError, UserDocumentState, UserId};

pub struct PgUserDocumentStateRepository {
    pool: PgPool,
}

impl PgUserDocumentStateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct UserDocumentStateRow {
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
    fn into_state(self) -> UserDocumentState {
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
        .map_err(|err| super::map_sqlx_error("user_document_state", "state conflict", err))?;

        Ok(row.map(UserDocumentStateRow::into_state))
    }

    async fn record_progress(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        progress_percent: i32,
        chapter_locator: Option<String>,
        chapter_offset: Option<i32>,
    ) -> Result<UserDocumentState, AppError> {
        // Ownership-guarded upsert via INSERT...SELECT FROM documents: a missing or non-owned
        // document yields no row, surfacing NotFound instead of writing state against a foreign id.
        // max_progress_percent only grows (GREATEST); finished_at latches on first reach of 100%
        // and is never cleared by later lower progress.
        let row = sqlx::query_as!(
            UserDocumentStateRow,
            "INSERT INTO user_document_state \
                (user_id, document_id, progress_percent, max_progress_percent, finished_at, \
                 chapter_locator, chapter_offset, last_read_at, created_at, updated_at) \
             SELECT $1, $2, $3, $3, CASE WHEN $3 >= 100 THEN now() END, \
                 $4, $5, now(), now(), now() \
             FROM documents d WHERE d.id = $2 AND d.user_id = $1 \
             ON CONFLICT (user_id, document_id) DO UPDATE SET \
                progress_percent = EXCLUDED.progress_percent, \
                max_progress_percent = GREATEST(user_document_state.max_progress_percent, EXCLUDED.max_progress_percent), \
                finished_at = CASE \
                    WHEN GREATEST(COALESCE(user_document_state.max_progress_percent, 0), EXCLUDED.max_progress_percent) >= 100 \
                    THEN COALESCE(user_document_state.finished_at, now()) \
                    ELSE user_document_state.finished_at END, \
                chapter_locator = COALESCE(EXCLUDED.chapter_locator, user_document_state.chapter_locator), \
                chapter_offset = COALESCE(EXCLUDED.chapter_offset, user_document_state.chapter_offset), \
                last_read_at = now(), \
                updated_at = now() \
             RETURNING user_id, document_id, progress_percent, max_progress_percent, \
                       scroll_position AS \"scroll_position?: serde_json::Value\", chapter_locator, \
                       chapter_offset, last_read_at, finished_at, first_opened_at, last_opened_at, \
                       created_at, updated_at",
            user_id.into_uuid(),
            document_id.into_uuid(),
            progress_percent,
            chapter_locator,
            chapter_offset,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| super::map_sqlx_error("user_document_state", "state conflict", err))?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "Document",
                id: document_id.to_string(),
            })
        })?;

        Ok(row.into_state())
    }

    async fn record_document_opened(
        &self,
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
        .execute(&self.pool)
        .await
        .map_err(|err| super::map_sqlx_error("user_document_state", "state conflict", err))?;

        Ok(())
    }
}
