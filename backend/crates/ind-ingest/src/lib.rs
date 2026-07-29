#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::unimplemented,
        clippy::todo
    )
)]

pub mod archive_limits;
pub mod artifacts;
pub mod canonicalize;
pub mod egress_guard;
pub mod epub_processing;
pub mod feed_lead_image;
pub mod feed_parser;
pub mod file_upload;
pub mod html_extractor;
pub mod http_fetcher;
pub mod opml_parser;
pub mod pdf_extraction;
pub mod prepared_content;

pub use artifacts::build_document_assets;
pub use canonicalize::{CanonicalUrl, CanonicalizationConfig, CanonicalizeError, canonicalize_url};
pub use egress_guard::EgressUrlGuard;
pub use epub_processing::{EpubError, EpubMetadata, EpubTocEntry, EpubTocResponse, ProcessedEpub};
pub use feed_lead_image::extract_feed_lead_image;
pub use feed_parser::FeedRsFeedParser;
pub use file_upload::DocumentFileUploadProcessor;
pub use html_extractor::ScraperHtmlExtractor;
pub use http_fetcher::{ReqwestHttpFetcher, build_ingest_http_client};
pub use opml_parser::QuickXmlOpmlParser;
pub use pdf_extraction::{PdfExtractionError, extract_pdf_text};
pub use prepared_content::AssetBackedPreparedContentProvider;

pub const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;
