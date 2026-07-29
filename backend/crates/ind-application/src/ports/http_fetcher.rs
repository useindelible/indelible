use async_trait::async_trait;
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct FetchRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
}

impl FetchRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: Vec::new(),
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Bytes,
}

impl FetchResponse {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HttpFetchError {
    #[error("fetch failed: {0}")]
    Send(String),
    #[error("body read failed: {0}")]
    Body(String),
    #[error("{0}")]
    Disallowed(String),
}

#[async_trait]
pub trait HttpFetcher: Send + Sync {
    async fn fetch(&self, request: FetchRequest) -> Result<FetchResponse, HttpFetchError>;
}
