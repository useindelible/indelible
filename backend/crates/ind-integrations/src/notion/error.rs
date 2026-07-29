use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotionError {
    #[error("Notion rate limited; retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("Notion API error {status}: {body}")]
    Api { status: u16, body: String },
    #[error("Notion HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Notion JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Notion export state error: {0}")]
    State(String),
}
