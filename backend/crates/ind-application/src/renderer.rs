use serde::{Deserialize, Serialize};

use crate::error::AppError;
use ind_domain::{ItemId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderUrlRequest {
    pub item_id: ItemId,
    pub user_id: UserId,
    pub url: String,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMonolithRequest {
    pub item_id: ItemId,
    pub user_id: UserId,
    pub monolith_s3_key: String,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderArtifact {
    pub kind: String,
    pub s3_key: String,
    pub content_type: String,
    pub size_bytes: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ArtifactMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub title: Option<String>,
    pub byline: Option<String>,
    pub excerpt: Option<String>,
    pub word_count: Option<i32>,
    pub reading_time_minutes: Option<i32>,
    pub domain: Option<String>,
    #[serde(default)]
    pub lead_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetError {
    pub kind: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResult {
    pub artifacts: Vec<RenderArtifact>,
    #[serde(default)]
    pub asset_errors: Vec<AssetError>,
    pub wall_time_ms: u64,
    pub final_url: Option<String>,
}

#[async_trait::async_trait]
pub trait RendererClient: Send + Sync {
    async fn render_url(&self, req: RenderUrlRequest) -> Result<RenderResult, AppError>;
    async fn render_monolith(&self, req: RenderMonolithRequest) -> Result<RenderResult, AppError>;
}
