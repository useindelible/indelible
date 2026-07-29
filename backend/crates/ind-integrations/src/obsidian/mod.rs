mod category;
mod document_text;
mod hash;
mod paths;
mod render;
mod settings;
mod template;
mod types;

#[cfg(test)]
mod tests;

pub use category::category_for_item_type;
pub use document_text::format_full_document_text;
pub use hash::obsidian_content_hash;
pub use ind_domain::{
    ObsidianExportSettings, default_highlight_header_template, default_highlight_template,
    default_metadata_template, default_page_title_template, default_sync_notification_template,
};
pub use paths::{
    MAX_PATH_SEGMENT_BYTES, SERVER_BASE_FOLDER, full_document_path_for_note_path,
    obsidian_link_for_path, stable_subject_path_suffix,
};
pub use render::{render_document, render_sync_notification};
pub use settings::{settings_from_config, write_settings_to_config};
pub use types::{
    ObsidianArtifactEntry, ObsidianRenderCursor, ObsidianRenderDocument, ObsidianRenderError,
    ObsidianRenderHighlight, RenderedObsidianDocument,
};
