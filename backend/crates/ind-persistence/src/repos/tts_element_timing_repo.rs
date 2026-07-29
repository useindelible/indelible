use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::tts_element_timing::TtsElementTimingRepository;
use ind_domain::{TtsChunkRecordId, TtsElementTiming};

pub struct PgTtsElementTimingRepository {
    pool: PgPool,
}

impl PgTtsElementTimingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("tts_element_timing", "element timing already exists", err)
}

#[async_trait::async_trait]
impl TtsElementTimingRepository for PgTtsElementTimingRepository {
    async fn insert_batch(&self, timings: &[TtsElementTiming]) -> Result<(), AppError> {
        if timings.is_empty() {
            return Ok(());
        }

        // Postgres UNNEST carries parallel arrays in a single round-trip, which
        // is cheaper than issuing one INSERT per row for long elements lists.
        let chunk_ids: Vec<Uuid> = timings
            .iter()
            .map(|t| t.chunk_record_id.into_uuid())
            .collect();
        let element_indices: Vec<i32> = timings.iter().map(|t| t.element_index).collect();
        let starts: Vec<f64> = timings.iter().map(|t| t.start_timestamp).collect();
        let ends: Vec<Option<f64>> = timings.iter().map(|t| t.end_timestamp).collect();

        sqlx::query!(
            r#"
            INSERT INTO tts_element_timings (
                chunk_record_id, element_index, start_timestamp, end_timestamp
            )
            SELECT * FROM UNNEST($1::uuid[], $2::int4[], $3::float8[], $4::float8[])
            ON CONFLICT (chunk_record_id, element_index) DO NOTHING
            "#,
            &chunk_ids,
            &element_indices,
            &starts,
            &ends as &[Option<f64>],
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(())
    }

    async fn get_by_element(
        &self,
        chunk_record_id: TtsChunkRecordId,
        element_index: i32,
    ) -> Result<Option<TtsElementTiming>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT chunk_record_id, element_index, start_timestamp, end_timestamp
            FROM tts_element_timings
            WHERE chunk_record_id = $1 AND element_index = $2
            "#,
            chunk_record_id.into_uuid(),
            element_index,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(|r| TtsElementTiming {
            chunk_record_id: TtsChunkRecordId::from_uuid(r.chunk_record_id),
            element_index: r.element_index,
            start_timestamp: r.start_timestamp,
            end_timestamp: r.end_timestamp,
        }))
    }
}
