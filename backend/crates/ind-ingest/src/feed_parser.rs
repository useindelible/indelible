use ind_application::ports::{
    FeedParseError, FeedParser, ParsedFeed, ParsedFeedEntry, ParsedFeedKind, ParsedFeedLink,
    ParsedFeedMediaContent,
};

pub struct FeedRsFeedParser;

impl FeedRsFeedParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FeedRsFeedParser {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedParser for FeedRsFeedParser {
    fn parse(&self, body: &[u8]) -> Result<ParsedFeed, FeedParseError> {
        let feed =
            feed_rs::parser::parse(body).map_err(|err| FeedParseError::Parse(err.to_string()))?;

        Ok(ParsedFeed {
            kind: map_feed_kind(&feed.feed_type),
            title: feed.title.as_ref().map(|t| t.content.clone()),
            description: feed.description.as_ref().map(|d| d.content.clone()),
            links: feed.links.iter().map(map_link).collect(),
            icon_url: feed.icon.as_ref().map(|i| i.uri.clone()),
            logo_url: feed.logo.as_ref().map(|l| l.uri.clone()),
            entries: feed.entries.iter().map(map_entry).collect(),
        })
    }
}

fn map_feed_kind(kind: &feed_rs::model::FeedType) -> ParsedFeedKind {
    match kind {
        feed_rs::model::FeedType::Atom => ParsedFeedKind::Atom,
        feed_rs::model::FeedType::RSS0
        | feed_rs::model::FeedType::RSS1
        | feed_rs::model::FeedType::RSS2 => ParsedFeedKind::Rss,
        feed_rs::model::FeedType::JSON => ParsedFeedKind::Json,
    }
}

fn map_link(link: &feed_rs::model::Link) -> ParsedFeedLink {
    ParsedFeedLink {
        href: link.href.clone(),
        rel: link.rel.clone(),
        media_type: link.media_type.clone(),
    }
}

fn map_entry(entry: &feed_rs::model::Entry) -> ParsedFeedEntry {
    let media_contents = entry
        .media
        .iter()
        .flat_map(|media| media.content.iter())
        .map(|content| ParsedFeedMediaContent {
            content_type: content.content_type.as_ref().map(|mime| mime.to_string()),
        })
        .collect();

    ParsedFeedEntry {
        media_contents,
        links: entry.links.iter().map(map_link).collect(),
    }
}
