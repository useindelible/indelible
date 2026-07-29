#[derive(Debug, thiserror::Error)]
pub enum OpmlParseError {
    #[error("invalid OPML XML: {0}")]
    Invalid(String),
}

pub trait OpmlParser: Send + Sync {
    fn parse_feed_urls(&self, opml_xml: &str) -> Result<Vec<String>, OpmlParseError>;
}
