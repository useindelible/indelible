use chrono::{DateTime, Utc};

use ind_domain::{
    ArchiveAssetKind, AttachProvidedContentJob, DocumentId, EmbedDocumentJob,
    ExtractEntitiesDocumentJob, PrepareDocumentJob, SearchReindexDocumentJob,
    SuggestTagsDocumentJob, SummarizeDocumentJob, UserId, YoutubeIngestDocumentJob, job_types,
};

/// Payload for a single job-outbox entry committed atomically with a lifecycle mutation.
#[derive(Debug, Clone)]
pub struct OutboxEntry {
    pub job_type: String,
    pub payload: serde_json::Value,
    pub dedupe_key: Option<String>,
    pub available_at: DateTime<Utc>,
}

/// Durable document reindex (TASK-233). Enqueued atomically with the state change that requires
/// it (save/materialize, document highlight/note, feed prepare render-complete).
pub fn search_reindex_document_outbox(
    document_id: DocumentId,
    available_at: DateTime<Utc>,
) -> OutboxEntry {
    #[expect(
        clippy::expect_used,
        reason = "serializing a plain owned struct with no map keys or non-string keys into serde_json::Value cannot fail"
    )]
    let payload = serde_json::to_value(SearchReindexDocumentJob { document_id })
        .expect("SearchReindexDocumentJob serializes");
    OutboxEntry {
        job_type: job_types::SEARCH_REINDEX_DOCUMENT.into(),
        payload,
        dedupe_key: Some(format!(
            "{}:{document_id}",
            job_types::SEARCH_REINDEX_DOCUMENT
        )),
        available_at,
    }
}

/// Document-keyed embed (TASK-234). Enqueued by the content-gated engagement rule
/// (`build_engaged_document_ai_outbox_tx`) when a completed readable asset already exists, and
/// by `feed.prepare_document` completion for engaged documents. All producers share the dedupe
/// key so concurrent triggers collapse to one embed.
pub fn document_ai_embed_outbox(
    document_id: DocumentId,
    available_at: DateTime<Utc>,
) -> OutboxEntry {
    #[expect(
        clippy::expect_used,
        reason = "serializing a plain owned struct with no map keys or non-string keys into serde_json::Value cannot fail"
    )]
    let payload = serde_json::to_value(EmbedDocumentJob { document_id })
        .expect("EmbedDocumentJob serializes");
    OutboxEntry {
        job_type: job_types::DOCUMENT_AI_EMBED.into(),
        payload,
        dedupe_key: Some(format!("{}:{document_id}", job_types::DOCUMENT_AI_EMBED)),
        available_at,
    }
}

pub fn document_ai_summarize_outbox(
    document_id: DocumentId,
    available_at: DateTime<Utc>,
) -> OutboxEntry {
    #[expect(
        clippy::expect_used,
        reason = "serializing a plain owned struct with no map keys or non-string keys into serde_json::Value cannot fail"
    )]
    let payload = serde_json::to_value(SummarizeDocumentJob { document_id })
        .expect("SummarizeDocumentJob serializes");
    OutboxEntry {
        job_type: job_types::DOCUMENT_AI_SUMMARIZE.into(),
        payload,
        dedupe_key: Some(format!(
            "{}:{document_id}",
            job_types::DOCUMENT_AI_SUMMARIZE
        )),
        available_at,
    }
}

pub fn document_ai_tags_outbox(
    document_id: DocumentId,
    available_at: DateTime<Utc>,
) -> OutboxEntry {
    #[expect(
        clippy::expect_used,
        reason = "serializing a plain owned struct with no map keys or non-string keys into serde_json::Value cannot fail"
    )]
    let payload = serde_json::to_value(SuggestTagsDocumentJob { document_id })
        .expect("SuggestTagsDocumentJob serializes");
    OutboxEntry {
        job_type: job_types::DOCUMENT_AI_TAGS.into(),
        payload,
        dedupe_key: Some(format!("{}:{document_id}", job_types::DOCUMENT_AI_TAGS)),
        available_at,
    }
}

pub fn document_ai_entities_outbox(
    document_id: DocumentId,
    available_at: DateTime<Utc>,
) -> OutboxEntry {
    #[expect(
        clippy::expect_used,
        reason = "serializing a plain owned struct with no map keys or non-string keys into serde_json::Value cannot fail"
    )]
    let payload = serde_json::to_value(ExtractEntitiesDocumentJob { document_id })
        .expect("ExtractEntitiesDocumentJob serializes");
    OutboxEntry {
        job_type: job_types::DOCUMENT_AI_ENTITIES.into(),
        payload,
        dedupe_key: Some(format!("{}:{document_id}", job_types::DOCUMENT_AI_ENTITIES)),
        available_at,
    }
}

pub fn document_ai_processing_outbox(
    document_id: DocumentId,
    available_at: DateTime<Utc>,
) -> Vec<OutboxEntry> {
    vec![
        document_ai_embed_outbox(document_id, available_at),
        document_ai_summarize_outbox(document_id, available_at),
        document_ai_tags_outbox(document_id, available_at),
        document_ai_entities_outbox(document_id, available_at),
    ]
}

/// The dedupe key is per `(document_id, asset_kind)` so a re-run of the same save collapses to one
/// pending attach while full-archive's monolith + readable rows stay independent.
#[allow(clippy::too_many_arguments)]
pub fn attach_provided_content_outbox(
    document_id: DocumentId,
    user_id: UserId,
    asset_kind: ArchiveAssetKind,
    storage_key: String,
    storage_bucket: String,
    content_type: String,
    size_bytes: i64,
    available_at: DateTime<Utc>,
) -> OutboxEntry {
    let dedupe_key = Some(format!(
        "{}:{document_id}:{asset_kind}",
        job_types::DOCUMENT_ATTACH_PROVIDED_CONTENT
    ));
    #[expect(
        clippy::expect_used,
        reason = "serializing a plain owned struct with no map keys or non-string keys into serde_json::Value cannot fail"
    )]
    let payload = serde_json::to_value(AttachProvidedContentJob {
        document_id,
        user_id,
        asset_kind,
        storage_key,
        storage_bucket,
        content_type,
        size_bytes,
    })
    .expect("AttachProvidedContentJob serializes");
    OutboxEntry {
        job_type: job_types::DOCUMENT_ATTACH_PROVIDED_CONTENT.into(),
        payload,
        dedupe_key,
        available_at,
    }
}

/// Readable-content preparation for an engaged document with no completed readable asset yet
/// (TASK-234). On completion the feed.prepare_document handler enqueues `document.ai.embed`
/// because the document is now engaged. The payload carries the canonical URL to render.
pub fn feed_prepare_document_outbox(
    document_id: DocumentId,
    user_id: UserId,
    url: String,
    available_at: DateTime<Utc>,
) -> OutboxEntry {
    #[expect(
        clippy::expect_used,
        reason = "serializing a plain owned struct with no map keys or non-string keys into serde_json::Value cannot fail"
    )]
    let payload = serde_json::to_value(PrepareDocumentJob {
        document_id,
        user_id,
        url,
    })
    .expect("PrepareDocumentJob serializes");
    OutboxEntry {
        job_type: job_types::FEED_PREPARE_DOCUMENT.into(),
        payload,
        dedupe_key: Some(format!(
            "{}:{document_id}",
            job_types::FEED_PREPARE_DOCUMENT
        )),
        available_at,
    }
}

/// Document-keyed YouTube transcript ingest (TASK-240). Enqueued atomically with the save that
/// routes a YouTube URL away from the generic readable render (extension reader/full-archive save
/// side-effect) and by the worker prepare choke point / Readwise import. All producers share the
/// dedupe key so overlapping routes collapse to one ingest.
pub fn youtube_ingest_document_outbox(
    document_id: DocumentId,
    user_id: UserId,
    url: String,
    available_at: DateTime<Utc>,
) -> OutboxEntry {
    #[expect(
        clippy::expect_used,
        reason = "serializing a plain owned struct with no map keys or non-string keys into serde_json::Value cannot fail"
    )]
    let payload = serde_json::to_value(YoutubeIngestDocumentJob {
        document_id,
        user_id,
        url,
    })
    .expect("YoutubeIngestDocumentJob serializes");
    OutboxEntry {
        job_type: job_types::DOCUMENT_YOUTUBE_INGEST.into(),
        payload,
        dedupe_key: Some(format!(
            "{}:{document_id}",
            job_types::DOCUMENT_YOUTUBE_INGEST
        )),
        available_at,
    }
}
