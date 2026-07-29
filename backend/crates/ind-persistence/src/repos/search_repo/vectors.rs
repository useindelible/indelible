use ind_application::repos::content_vector::ContentVectorRepository;

use super::*;

impl PgSearchRepository {
    pub(super) async fn upsert_content_vector_impl(
        &self,
        vector: &ContentVector,
    ) -> Result<ContentVector, AppError> {
        self.content_vectors.upsert_chunk(vector).await
    }
}
