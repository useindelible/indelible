use super::prelude::*;

pub struct LibraryEntryFactory {
    user_id: UserId,
    document_id: DocumentId,
}

impl LibraryEntryFactory {
    pub fn new(user_id: UserId, document_id: DocumentId) -> Self {
        Self {
            user_id,
            document_id,
        }
    }
    fn build(self) -> LibraryEntry {
        let now = Utc::now();
        LibraryEntry {
            id: LibraryEntryId::new(),
            user_id: self.user_id,
            document_id: self.document_id,
            saved_at: now,
            triage_state: TriageState::Inbox,
            is_favorite: false,
            is_shortlisted: false,
            deleted_at: None,
            source: ContentSource::Manual,
            source_delivery_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub async fn insert(self, pool: &sqlx::PgPool) -> LibraryEntry {
        let repo = PgLibraryRepository::new(pool.clone());
        repo.insert_entry(self.build())
            .await
            .expect("LibraryEntryFactory::insert failed")
    }
}
