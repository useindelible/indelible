use ind_application::AppError;
use ind_application::repos::content_vector::VectorReplacementOutcome;
use ind_application::repos::embedding_backfill::EffectiveEmbeddingTarget;
use ind_domain::{ContentVector, DocumentId, DomainError, MilaPlatformDefaults, UserId};
use sqlx::{Postgres, Transaction};

use super::PgContentVectorRepository;
use super::types::*;

fn document_lock_key(document_id: DocumentId) -> i64 {
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&document_id.into_uuid().as_bytes()[..8]);
    i64::from_be_bytes(bytes)
}

async fn lock_document(
    tx: &mut Transaction<'_, Postgres>,
    document_id: DocumentId,
) -> Result<(), AppError> {
    sqlx::query!(
        "SELECT pg_advisory_xact_lock($1)",
        document_lock_key(document_id)
    )
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

impl PgContentVectorRepository {
    pub(super) async fn upsert_chunk_impl(
        &self,
        vector: &ContentVector,
    ) -> Result<ContentVector, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        lock_document(&mut tx, vector.document_id).await?;
        let embedding_literal = build_vector_literal(&vector.embedding);
        let row = sqlx::query_as!(
            ContentVectorRow,
            r#"
            INSERT INTO content_vectors (
                id,
                document_id,
                user_id,
                embedding_model,
                embedding_dim,
                section_kind,
                section_key,
                chunk_index,
                content,
                token_count,
                search_config,
                embedding,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                CAST($11 AS text)::regconfig,
                CAST($12 AS text)::vector,
                $13
            )
            ON CONFLICT (document_id, section_key, chunk_index) WHERE document_id IS NOT NULL DO UPDATE SET
                user_id = EXCLUDED.user_id,
                embedding_model = EXCLUDED.embedding_model,
                embedding_dim = EXCLUDED.embedding_dim,
                section_kind = EXCLUDED.section_kind,
                content = EXCLUDED.content,
                token_count = EXCLUDED.token_count,
                search_config = EXCLUDED.search_config,
                embedding = EXCLUDED.embedding,
                created_at = EXCLUDED.created_at
            RETURNING
                id,
                document_id,
                user_id,
                embedding_model,
                embedding_dim,
                section_kind,
                section_key,
                chunk_index,
                content,
                token_count,
                search_config::text AS "search_config!",
                embedding::text AS "embedding!",
                created_at
            "#,
            vector.id.as_uuid(),
            vector.document_id.into_uuid(),
            vector.user_id.into_uuid(),
            &vector.embedding_model,
            vector.embedding_dim,
            search_section_kind_to_str(vector.section_kind),
            &vector.section_key,
            vector.chunk_index,
            &vector.content,
            vector.token_count,
            &vector.search_config,
            &embedding_literal,
            vector.created_at,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        ContentVector::try_from(row)
    }

    pub(super) async fn replace_for_document_impl(
        &self,
        document_id: DocumentId,
        vectors: &[ContentVector],
    ) -> Result<(), AppError> {
        if let Some(first) = vectors.first()
            && vectors.iter().any(|vector| {
                vector.document_id != document_id
                    || vector.user_id != first.user_id
                    || vector.embedding_model != first.embedding_model
                    || vector.embedding_dim != first.embedding_dim
                    || vector.search_config != first.search_config
            })
        {
            return Err(AppError::Domain(DomainError::InvariantViolation {
                message: "replacement vectors must share document and embedding identity".into(),
            }));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        lock_document(&mut tx, document_id).await?;
        sqlx::query!(
            "DELETE FROM content_vectors WHERE document_id = $1",
            document_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        for vector in vectors {
            insert_vector_tx(&mut tx, vector).await?;
        }

        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(())
    }

    pub(super) async fn replace_for_document_if_target_current_impl(
        &self,
        document_id: DocumentId,
        user_id: UserId,
        vectors: &[ContentVector],
        generated_target: &EffectiveEmbeddingTarget,
        platform_defaults: &MilaPlatformDefaults,
    ) -> Result<VectorReplacementOutcome, AppError> {
        validate_replacement_vectors(document_id, Some(user_id), vectors)?;
        if vectors.first().is_some_and(|vector| {
            vector.embedding_model != generated_target.embedding_model
                || vector.embedding_dim != generated_target.embedding_dim
        }) {
            return Err(AppError::Domain(DomainError::InvariantViolation {
                message: "replacement vector identity must match its generated target".into(),
            }));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        lock_document(&mut tx, document_id).await?;
        let stored = sqlx::query!(
            r#"
            SELECT embedding_model, embedding_dim, byo_enabled
            FROM mila_config
            WHERE user_id = $1
            FOR SHARE
            "#,
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;
        let current_target = match stored {
            Some(stored) if stored.byo_enabled => EffectiveEmbeddingTarget {
                embedding_model: stored.embedding_model,
                embedding_dim: stored.embedding_dim,
            },
            _ => EffectiveEmbeddingTarget {
                embedding_model: platform_defaults.embedding_model.clone(),
                embedding_dim: platform_defaults.embedding_dim,
            },
        };
        if current_target != *generated_target {
            tx.rollback()
                .await
                .map_err(|err| AppError::Repository(Box::new(err)))?;
            return Ok(VectorReplacementOutcome::Superseded);
        }

        sqlx::query!(
            "DELETE FROM content_vectors WHERE document_id = $1",
            document_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;
        for vector in vectors {
            insert_vector_tx(&mut tx, vector).await?;
        }
        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        Ok(VectorReplacementOutcome::Committed)
    }

    pub(super) async fn delete_for_document_impl(
        &self,
        document_id: DocumentId,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM content_vectors WHERE document_id = $1",
            document_id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    pub(super) async fn delete_for_user_impl(&self, user_id: UserId) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM content_vectors WHERE user_id = $1",
            user_id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    pub(super) async fn count_documents_by_user_impl(
        &self,
        user_id: UserId,
    ) -> Result<i64, AppError> {
        sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT cv.document_id) AS "count!"
            FROM content_vectors cv
            WHERE cv.user_id = $1
            "#,
            user_id.into_uuid(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)
    }
}

fn validate_replacement_vectors(
    document_id: DocumentId,
    user_id: Option<UserId>,
    vectors: &[ContentVector],
) -> Result<(), AppError> {
    if let Some(first) = vectors.first()
        && (user_id.is_some_and(|user_id| first.user_id != user_id)
            || vectors.iter().any(|vector| {
                vector.document_id != document_id
                    || vector.user_id != first.user_id
                    || vector.embedding_model != first.embedding_model
                    || vector.embedding_dim != first.embedding_dim
                    || vector.search_config != first.search_config
            }))
    {
        return Err(AppError::Domain(DomainError::InvariantViolation {
            message: "replacement vectors must share document and embedding identity".into(),
        }));
    }
    Ok(())
}

async fn insert_vector_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    vector: &ContentVector,
) -> Result<(), AppError> {
    let embedding_literal = build_vector_literal(&vector.embedding);
    sqlx::query!(
        r#"
        INSERT INTO content_vectors (
            id,
            document_id,
            user_id,
            embedding_model,
            embedding_dim,
            section_kind,
            section_key,
            chunk_index,
            content,
            token_count,
            search_config,
            embedding,
            created_at
        )
        VALUES (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9,
            $10,
            CAST($11 AS text)::regconfig,
            CAST($12 AS text)::vector,
            $13
        )
        "#,
        vector.id.as_uuid(),
        vector.document_id.into_uuid(),
        vector.user_id.into_uuid(),
        &vector.embedding_model,
        vector.embedding_dim,
        search_section_kind_to_str(vector.section_kind),
        &vector.section_key,
        vector.chunk_index,
        &vector.content,
        vector.token_count,
        &vector.search_config,
        &embedding_literal,
        vector.created_at,
    )
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;

    Ok(())
}
