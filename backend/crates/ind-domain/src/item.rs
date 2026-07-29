use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportSource {
    ReadwiseImport,
    NotionImport,
}

impl ImportSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadwiseImport => "readwise_import",
            Self::NotionImport => "notion_import",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Article,
    Book,
    Email,
    Pdf,
    Tweet,
    Video,
    Podcast,
}

impl ItemType {
    pub const NAMES: &'static [&'static str] = &[
        "article", "book", "email", "pdf", "tweet", "video", "podcast",
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Article => "article",
            Self::Book => "book",
            Self::Email => "email",
            Self::Pdf => "pdf",
            Self::Tweet => "tweet",
            Self::Video => "video",
            Self::Podcast => "podcast",
        }
    }
}

impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ItemType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "article" => Ok(Self::Article),
            "book" => Ok(Self::Book),
            "email" => Ok(Self::Email),
            "pdf" => Ok(Self::Pdf),
            "tweet" => Ok(Self::Tweet),
            "video" => Ok(Self::Video),
            "podcast" => Ok(Self::Podcast),
            other => Err(format!("invalid item type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriageState {
    #[default]
    Inbox,
    Later,
    Archive,
}

impl TriageState {
    pub const NAMES: &'static [&'static str] = &["inbox", "later", "archive"];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Later => "later",
            Self::Archive => "archive",
        }
    }
}

impl fmt::Display for TriageState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TriageState {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "inbox" => Ok(Self::Inbox),
            "later" => Ok(Self::Later),
            "archive" => Ok(Self::Archive),
            other => Err(format!("invalid triage state: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentSource {
    Manual,
    Extension,
    ShareSheet,
    Feed,
    Email,
    Api,
    Cli,
    Import,
}

impl ContentSource {
    pub const NAMES: &'static [&'static str] = &[
        "manual",
        "extension",
        "share_sheet",
        "feed",
        "email",
        "api",
        "cli",
        "import",
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Extension => "extension",
            Self::ShareSheet => "share_sheet",
            Self::Feed => "feed",
            Self::Email => "email",
            Self::Api => "api",
            Self::Cli => "cli",
            Self::Import => "import",
        }
    }
}

impl fmt::Display for ContentSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ContentSource {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "manual" => Ok(Self::Manual),
            "extension" => Ok(Self::Extension),
            "share_sheet" => Ok(Self::ShareSheet),
            "feed" => Ok(Self::Feed),
            "email" => Ok(Self::Email),
            "api" => Ok(Self::Api),
            "cli" => Ok(Self::Cli),
            "import" => Ok(Self::Import),
            other => Err(format!("invalid content source: {other}")),
        }
    }
}
