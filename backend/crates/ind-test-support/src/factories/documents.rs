use super::prelude::*;

pub struct DocumentFactory {
    user_id: UserId,
    document_type: DocumentType,
    canonical_url: String,
    title: Option<String>,
    excerpt: Option<String>,
}

impl DocumentFactory {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            document_type: DocumentType::Article,
            canonical_url: format!(
                "https://{}.example.com/{}",
                uuid::Uuid::now_v7().simple(),
                short_unique_suffix()
            ),
            title: None,
            excerpt: None,
        }
    }

    pub fn with_document_type(mut self, document_type: DocumentType) -> Self {
        self.document_type = document_type;
        self
    }

    pub fn with_canonical_url(mut self, url: impl Into<String>) -> Self {
        self.canonical_url = url.into();
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }

    fn title_value(&self) -> String {
        self.title.clone().unwrap_or_else(|| {
            format!(
                "{} {}",
                Sentence(3..8).fake::<String>(),
                short_unique_suffix()
            )
        })
    }

    pub async fn insert(self, pool: &sqlx::PgPool) -> Document {
        let repo = PgDocumentRepository::new(pool.clone());
        let title = self.title_value();
        repo.upsert_url_backed(NewUrlDocument {
            id: DocumentId::new(),
            user_id: self.user_id,
            document_type: self.document_type,
            canonical_url: self.canonical_url,
            original_url: None,
            content_hash: None,
            title,
            author: None,
            excerpt: self.excerpt,
            published_at: None,
            language: None,
            domain: None,
            lead_image_url: None,
            thumbnail_url: None,
        })
        .await
        .expect("DocumentFactory url-backed insert failed")
    }
}
