mod blocks;
mod client;
mod config;
mod error;
mod page;
mod rate_limit;
mod schema;

pub use blocks::{
    HighlightLocation, HighlightText, MAX_BLOCKS_PER_REQUEST, MAX_PAYLOAD_BYTES,
    MAX_RICH_TEXT_CHARS, NotionBlock, build_highlight_blocks, build_highlight_blocks_with_options,
    chunk_blocks_for_request, notion_block_to_json,
};
pub use client::NotionClient;
pub use config::{
    NotionManagedTarget, NotionPropertyIds, notion_settings_from_config, property_ids_from_config,
    write_managed_target_to_config, write_settings_to_config,
};
pub use error::NotionError;
pub use page::NotionPageSpec;
pub use rate_limit::NotionRateLimiter;

// Re-exported from ind-domain so ind-auth (which can't depend on
// ind-integrations) and this crate share one source of truth for the
// Notion API version header. Bump in ind-domain to upgrade everywhere.
pub use ind_domain::NOTION_API_VERSION;
