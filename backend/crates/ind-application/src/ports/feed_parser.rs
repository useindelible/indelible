#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedFeedKind {
    Atom,
    Rss,
    Json,
    Other,
}

#[derive(Debug, Clone)]
pub struct ParsedFeedLink {
    pub href: String,
    pub rel: Option<String>,
    pub media_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedFeedMediaContent {
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedFeedEntry {
    pub media_contents: Vec<ParsedFeedMediaContent>,
    pub links: Vec<ParsedFeedLink>,
}

#[derive(Debug, Clone)]
pub struct ParsedFeed {
    pub kind: ParsedFeedKind,
    pub title: Option<String>,
    pub description: Option<String>,
    pub links: Vec<ParsedFeedLink>,
    pub icon_url: Option<String>,
    pub logo_url: Option<String>,
    pub entries: Vec<ParsedFeedEntry>,
}

#[derive(Debug, thiserror::Error)]
pub enum FeedParseError {
    #[error("feed parse failed: {0}")]
    Parse(String),
}

pub trait FeedParser: Send + Sync {
    fn parse(&self, body: &[u8]) -> Result<ParsedFeed, FeedParseError>;
}
