//! Shared document-identity construction for feed-delivered content.
//!
//! Both the save-from-delivery flow (`LibraryService`) and the readable-preparation flow
//! (`FeedPreparationService`) turn a `feed_source_entries` row into a `MaterializeIdentity`:
//! URL-backed when the entry has a canonical URL, otherwise origin-backed via the feed source
//! entry. Keeping this in one place ensures both flows resolve the same document identity.

use ind_domain::{
    DocumentId, DocumentOriginType, DocumentType, FeedSourceEntry, ItemType, NewOriginDocument,
    NewUrlDocument, UserId,
};

use crate::dispatch::infer_item_type_for_url;
use crate::repos::document_lifecycle::{MaterializeIdentity, MaterializeOrigin};

/// The document content families mirror the legacy item families one-to-one.
pub fn document_type_for(item_type: ItemType) -> DocumentType {
    match item_type {
        ItemType::Article => DocumentType::Article,
        ItemType::Book => DocumentType::Book,
        ItemType::Email => DocumentType::Email,
        ItemType::Pdf => DocumentType::Pdf,
        ItemType::Tweet => DocumentType::Tweet,
        ItemType::Video => DocumentType::Video,
        ItemType::Podcast => DocumentType::Podcast,
    }
}

pub fn domain_from_url(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
}

/// Build the document identity for a feed source entry. URL-backed when the entry has a
/// canonical URL (the common RSS case), otherwise origin-backed. The provenance origin is
/// always the feed source entry, so adoption/back-linking is idempotent across save and prepare.
pub fn feed_entry_identity(user_id: UserId, entry: &FeedSourceEntry) -> MaterializeIdentity {
    let document_type = document_type_for(
        entry
            .url
            .as_deref()
            .map(infer_item_type_for_url)
            .unwrap_or(ItemType::Article),
    );
    let origin = MaterializeOrigin {
        origin_type: DocumentOriginType::FeedSourceEntry,
        origin_id: *entry.id.as_uuid(),
    };

    match entry.canonical_url.clone() {
        Some(canonical) => MaterializeIdentity::Url {
            document: NewUrlDocument {
                id: DocumentId::new(),
                user_id,
                document_type,
                canonical_url: canonical,
                original_url: entry.url.clone(),
                content_hash: None,
                title: entry.title.clone(),
                author: entry.author.clone(),
                excerpt: entry.excerpt.clone(),
                published_at: entry.published_at,
                language: entry.language.clone(),
                domain: entry.url.as_deref().and_then(domain_from_url),
                lead_image_url: entry.lead_image_url.clone(),
                thumbnail_url: entry.lead_image_url.clone(),
            },
            origin: Some(origin),
        },
        None => MaterializeIdentity::Origin {
            document: NewOriginDocument {
                id: DocumentId::new(),
                user_id,
                document_type,
                content_hash: None,
                original_url: entry.url.clone(),
                title: entry.title.clone(),
                author: entry.author.clone(),
                excerpt: entry.excerpt.clone(),
                published_at: entry.published_at,
                language: entry.language.clone(),
                domain: None,
                lead_image_url: entry.lead_image_url.clone(),
                thumbnail_url: entry.lead_image_url.clone(),
                sender_id: None,
            },
            origin,
        },
    }
}
