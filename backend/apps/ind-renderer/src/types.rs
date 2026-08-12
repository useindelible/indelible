use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RenderUrlRequest {
    pub item_id: String,
    pub user_id: String,
    pub url: String,
    pub outputs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RenderMonolithRequest {
    pub item_id: String,
    pub user_id: String,
    pub monolith_s3_key: String,
    pub outputs: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AssetError {
    pub kind: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct RenderResponse {
    pub artifacts: Vec<ArtifactResponse>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub asset_errors: Vec<AssetError>,
    pub wall_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RenderErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct ArtifactResponse {
    pub kind: String,
    pub s3_key: String,
    pub content_type: String,
    pub size_bytes: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ArtifactMetadata>,
}

#[derive(Debug, Serialize)]
pub struct ArtifactMetadata {
    pub title: Option<String>,
    pub byline: Option<String>,
    pub excerpt: Option<String>,
    pub word_count: Option<i32>,
    pub reading_time_minutes: Option<i32>,
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_image: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub browser_running: bool,
}

#[derive(Debug, Deserialize)]
pub struct JsDefuddleArticle {
    pub title: Option<String>,
    pub editorial_title: Option<String>,
    pub content: Option<String>,
    pub author: Option<String>,
    pub editorial_author: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_omits_empty_errors_but_serializes_partial_failures() {
        let mut response = RenderResponse {
            artifacts: vec![],
            asset_errors: vec![],
            wall_time_ms: 100,
            final_url: None,
        };
        assert!(
            !serde_json::to_string(&response)
                .unwrap()
                .contains("asset_errors")
        );
        response.asset_errors.push(AssetError {
            kind: "pdf".into(),
            error: "timeout".into(),
        });
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("asset_errors") && json.contains("pdf") && json.contains("timeout"));
    }
}
