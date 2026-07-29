use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::mila_session::{MilaSessionRepository, MilaSessionWithPreview};
use ind_domain::{
    CollectionId, DocumentId, DomainError, MessageRole, MilaMessage, MilaSession, MilaSessionType,
    UserId,
};

pub struct PgMilaSessionRepository {
    pool: PgPool,
}

impl PgMilaSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct MilaSessionRow {
    id: Uuid,
    user_id: Uuid,
    document_id: Option<Uuid>,
    collection_id: Option<Uuid>,
    session_type: String,
    created_at: DateTime<Utc>,
    last_active: DateTime<Utc>,
}

struct MilaMessageRow {
    id: Uuid,
    session_id: Uuid,
    role: String,
    content: String,
    source_chunks: Vec<Uuid>,
    created_at: DateTime<Utc>,
}

struct MilaSessionPreviewRow {
    id: Uuid,
    user_id: Uuid,
    document_id: Option<Uuid>,
    collection_id: Option<Uuid>,
    session_type: String,
    created_at: DateTime<Utc>,
    last_active: DateTime<Utc>,
    preview_content: Option<String>,
    preview_role: Option<String>,
}

impl TryFrom<MilaSessionRow> for MilaSession {
    type Error = AppError;

    fn try_from(row: MilaSessionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ind_domain::MilaSessionId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            document_id: row.document_id.map(DocumentId::from_uuid),
            collection_id: row.collection_id.map(CollectionId::from_uuid),
            session_type: parse_session_type(&row.session_type)?,
            created_at: row.created_at,
            last_active: row.last_active,
        })
    }
}

impl TryFrom<MilaMessageRow> for MilaMessage {
    type Error = AppError;

    fn try_from(row: MilaMessageRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ind_domain::MilaMessageId::from_uuid(row.id),
            session_id: ind_domain::MilaSessionId::from_uuid(row.session_id),
            role: parse_message_role(&row.role)?,
            content: row.content,
            source_chunks: row.source_chunks,
            created_at: row.created_at,
        })
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("mila_session", "duplicate mila session", err)
}

fn format_session_type(value: MilaSessionType) -> &'static str {
    match value {
        MilaSessionType::SingleDocument => "single_document",
        MilaSessionType::CrossItem => "cross_item",
        MilaSessionType::Collection => "collection",
    }
}

fn parse_session_type(value: &str) -> Result<MilaSessionType, AppError> {
    match value {
        "single_document" => Ok(MilaSessionType::SingleDocument),
        "cross_item" => Ok(MilaSessionType::CrossItem),
        "collection" => Ok(MilaSessionType::Collection),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("unknown mila session type: {other}"),
        })),
    }
}

fn format_message_role(value: MessageRole) -> &'static str {
    match value {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
    }
}

fn parse_message_role(value: &str) -> Result<MessageRole, AppError> {
    match value {
        "user" => Ok(MessageRole::User),
        "assistant" => Ok(MessageRole::Assistant),
        "system" => Ok(MessageRole::System),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("unknown mila message role: {other}"),
        })),
    }
}

/// Insert a `mila_sessions` row inside a caller-owned transaction (TASK-234). Used by
/// `DocumentLifecycle::start_single_document_chat` so the session is committed atomically with
/// the materialize/back-link/retained-state/outbox writes, not as a separate follow-up call.
pub(crate) async fn insert_mila_session_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    session: &MilaSession,
) -> Result<MilaSession, AppError> {
    validate_session_scope(session)?;

    let row = sqlx::query_as!(
        MilaSessionRow,
        r#"
        INSERT INTO mila_sessions (
            id, user_id, document_id, collection_id, session_type, created_at, last_active
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id, user_id, document_id, collection_id, session_type, created_at, last_active
        "#,
        session.id.into_uuid(),
        session.user_id.into_uuid(),
        session.document_id.map(|id| id.into_uuid()),
        session.collection_id.map(|id| id.into_uuid()),
        format_session_type(session.session_type),
        session.created_at,
        session.last_active,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;

    MilaSession::try_from(row)
}

fn validate_session_scope(session: &MilaSession) -> Result<(), AppError> {
    // document_id is single-document-only; reject it on every other session type so a persisted
    // row cannot carry a contradictory scope (TASK-234).
    if session.session_type != MilaSessionType::SingleDocument && session.document_id.is_some() {
        return Err(AppError::Domain(DomainError::Validation {
            field: "document_id".into(),
            message: "document_id is only valid for single_document sessions".into(),
        }));
    }

    match session.session_type {
        MilaSessionType::SingleDocument => {
            if session.document_id.is_none() {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "document_id".into(),
                    message: "single_document sessions require document_id".into(),
                }));
            }
            if session.collection_id.is_some() {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "document_id".into(),
                    message: "single_document sessions cannot include collection_id".into(),
                }));
            }
        }
        MilaSessionType::CrossItem => {
            if session.collection_id.is_some() {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "collection_id".into(),
                    message: "cross_item sessions cannot include collection_id".into(),
                }));
            }
        }
        MilaSessionType::Collection => {
            if session.collection_id.is_none() {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "collection_id".into(),
                    message: "collection sessions require collection_id".into(),
                }));
            }
            if session.document_id.is_some() {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "document_id".into(),
                    message: "collection sessions cannot include document_id".into(),
                }));
            }
        }
    }

    Ok(())
}

#[async_trait::async_trait]
impl MilaSessionRepository for PgMilaSessionRepository {
    async fn create_session(&self, session: &MilaSession) -> Result<MilaSession, AppError> {
        validate_session_scope(session)?;

        let row = sqlx::query_as!(
            MilaSessionRow,
            r#"
            INSERT INTO mila_sessions (
                id,
                user_id,
                document_id,
                collection_id,
                session_type,
                created_at,
                last_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id,
                user_id,
                document_id,
                collection_id,
                session_type,
                created_at,
                last_active
            "#,
            session.id.into_uuid(),
            session.user_id.into_uuid(),
            session.document_id.map(|id| id.into_uuid()),
            session.collection_id.map(|id| id.into_uuid()),
            format_session_type(session.session_type),
            session.created_at,
            session.last_active,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        MilaSession::try_from(row)
    }

    async fn list_sessions_for_user(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<MilaSessionWithPreview>, AppError> {
        let rows = sqlx::query_as!(
            MilaSessionPreviewRow,
            r#"
            SELECT
                s.id,
                s.user_id,
                s.session_type,
                s.document_id,
                s.collection_id,
                s.created_at,
                s.last_active,
                m.content AS "preview_content: _",
                m.role AS "preview_role: _"
            FROM mila_sessions s
            LEFT JOIN LATERAL (
                SELECT content, role
                FROM mila_messages
                WHERE session_id = s.id
                ORDER BY created_at DESC
                LIMIT 1
            ) m ON true
            WHERE s.user_id = $1
            ORDER BY s.last_active DESC
            LIMIT $2
            "#,
            user_id.into_uuid(),
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(|row| {
                let session = MilaSession::try_from(MilaSessionRow {
                    id: row.id,
                    user_id: row.user_id,
                    document_id: row.document_id,
                    collection_id: row.collection_id,
                    session_type: row.session_type,
                    created_at: row.created_at,
                    last_active: row.last_active,
                })?;
                let preview_role = row
                    .preview_role
                    .as_deref()
                    .map(parse_message_role)
                    .transpose()?;
                Ok(MilaSessionWithPreview {
                    session,
                    preview_content: row.preview_content,
                    preview_role,
                })
            })
            .collect()
    }

    async fn find_session_for_user(
        &self,
        session_id: ind_domain::MilaSessionId,
        user_id: UserId,
    ) -> Result<Option<MilaSession>, AppError> {
        let row = sqlx::query_as!(
            MilaSessionRow,
            r#"
            SELECT
                id,
                user_id,
                document_id,
                collection_id,
                session_type,
                created_at,
                last_active
            FROM mila_sessions
            WHERE id = $1
              AND user_id = $2
            "#,
            session_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(MilaSession::try_from).transpose()
    }

    async fn insert_message(
        &self,
        user_id: UserId,
        message: &MilaMessage,
    ) -> Result<MilaMessage, AppError> {
        let row = sqlx::query_as!(
            MilaMessageRow,
            r#"
            INSERT INTO mila_messages (id, session_id, role, content, source_chunks, created_at)
            SELECT $1, $2, $3, $4, $5, $6
            WHERE EXISTS (
                SELECT 1
                FROM mila_sessions
                WHERE id = $2
                  AND user_id = $7
            )
            RETURNING
                id,
                session_id,
                role,
                content,
                source_chunks as "source_chunks!: Vec<Uuid>",
                created_at
            "#,
            message.id.into_uuid(),
            message.session_id.into_uuid(),
            format_message_role(message.role),
            message.content,
            &message.source_chunks[..],
            message.created_at,
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "mila_session",
                id: message.session_id.to_string(),
            })
        })?;

        MilaMessage::try_from(row)
    }

    async fn list_messages(
        &self,
        session_id: ind_domain::MilaSessionId,
        user_id: UserId,
    ) -> Result<Vec<MilaMessage>, AppError> {
        let rows = sqlx::query_as!(
            MilaMessageRow,
            r#"
            SELECT
                m.id,
                m.session_id,
                m.role,
                m.content,
                m.source_chunks as "source_chunks!: Vec<Uuid>",
                m.created_at
            FROM mila_messages m
            JOIN mila_sessions s ON s.id = m.session_id
            WHERE m.session_id = $1
              AND s.user_id = $2
            ORDER BY m.created_at ASC, m.id ASC
            "#,
            session_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(MilaMessage::try_from).collect()
    }

    async fn touch_session(
        &self,
        session_id: ind_domain::MilaSessionId,
        user_id: UserId,
        last_active: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE mila_sessions
            SET last_active = $3
            WHERE id = $1
              AND user_id = $2
            "#,
            session_id.into_uuid(),
            user_id.into_uuid(),
            last_active,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "mila_session",
                id: session_id.to_string(),
            }));
        }

        Ok(())
    }

    async fn delete_session(
        &self,
        session_id: ind_domain::MilaSessionId,
        user_id: UserId,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM mila_sessions
            WHERE id = $1
              AND user_id = $2
            "#,
            session_id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "mila_session",
                id: session_id.to_string(),
            }));
        }

        Ok(())
    }
}
