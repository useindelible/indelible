use serde::{Deserialize, Serialize};

pub mod ai_output;
pub mod ai_preset;
pub mod ai_run;
pub mod apalis_job;
pub mod api_token;
pub mod authorization_code;
pub mod background_job_recovery;
pub mod billing;
pub mod billing_usage_event;
pub mod collection;
pub mod content_vector;
pub mod dead_letter;
pub mod document;
pub mod document_asset;
pub mod document_lifecycle;
pub mod document_note;
pub mod document_reprocess;
pub mod document_upload;
pub mod email_alias;
pub mod email_ingest;
pub mod email_sender;
pub mod email_unsubscribe_commit;
pub mod email_unsubscribe_target;
pub mod email_verification;
pub mod embedding_backfill;
pub mod entity;
pub mod event;
pub mod export_cursor;
pub mod export_subject;
pub mod feed;
pub mod feed_delivery;
pub mod highlight;
pub mod home;
pub mod import_job;
pub mod integration_connection;
pub mod integration_oauth_token;
pub mod integrity;
pub mod library;
pub mod lifecycle_outbox;
pub mod maintenance;
pub mod mila_config;
pub mod mila_session;
pub mod notification_preferences;
pub mod oauth_flow;
pub mod oauth_identity;
pub mod obsidian_export;
pub mod obsidian_preview;
pub mod outbox;
pub mod password_reset;
pub mod playback_state;
pub mod prepared_content;
pub mod refresh_token;
pub mod retention_cleanup;
pub mod search;
pub mod search_reindex;
pub mod smart_list;
pub mod tag;
pub mod tts_audio_asset;
pub mod tts_chunk;
pub mod tts_element_timing;
pub mod tts_session;
pub mod tts_voice_persona;
pub mod usage_counter;
pub mod user;
pub mod user_document_state;
pub mod user_preferences;
pub mod webhook;

pub use retention_cleanup::{
    FeedDeliveryPruneCounts, FeedDeliveryRetentionWindows, RetentionCleanupRepository,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor(pub String);

#[derive(Debug)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<Cursor>,
}
