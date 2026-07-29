// Library facade so cross-crate tests can exercise the worker.
// Must expose enough of the module tree for integration tests.
pub mod auto_heal;
pub mod concurrency;
pub mod config;
pub mod context;
pub mod jobs {
    pub mod ai;
    pub mod article_toc;
    pub mod attach_provided_content;
    pub mod backfill;
    pub mod email_ingest;
    pub mod email_unsubscribe;
    pub mod feed;
    pub mod integrations;
    pub mod reading_metrics;
    pub mod render;
    pub mod reprocess;
    pub mod retention_cleanup;
    pub mod search;
    pub mod trash_cleanup;
    pub mod webhooks;
    pub mod youtube;
}

pub mod recovery_handler;
pub mod recovery_sweeper;
