use super::documents::DocumentFactory;
use super::library_entries::LibraryEntryFactory;
use super::prelude::*;

pub struct SavedDocument {
    pub document_id: DocumentId,
    pub library_entry_id: LibraryEntryId,
}

pub struct SavedDocumentFactory {
    user_id: UserId,
    document_type: DocumentType,
    title: Option<String>,
    excerpt: Option<String>,
    canonical_url: Option<String>,
}

impl SavedDocumentFactory {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            document_type: DocumentType::Article,
            title: None,
            excerpt: None,
            canonical_url: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.canonical_url = Some(url.into());
        self
    }

    pub fn with_document_type(mut self, document_type: DocumentType) -> Self {
        self.document_type = document_type;
        self
    }

    pub async fn insert(self, pool: &sqlx::PgPool) -> SavedDocument {
        let mut factory = DocumentFactory::new(self.user_id).with_document_type(self.document_type);
        if let Some(title) = self.title {
            factory = factory.with_title(title);
        }
        if let Some(url) = self.canonical_url {
            factory = factory.with_canonical_url(url);
        }
        if let Some(excerpt) = self.excerpt {
            factory = factory.with_excerpt(excerpt);
        }
        let document = factory.insert(pool).await;

        let entry = LibraryEntryFactory::new(self.user_id, document.id)
            .insert(pool)
            .await;
        SavedDocument {
            document_id: document.id,
            library_entry_id: entry.id,
        }
    }
}
