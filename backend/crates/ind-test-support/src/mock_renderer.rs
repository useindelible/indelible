use std::sync::Arc;

use bytes::Bytes;
use ind_application::AppError;
use ind_application::renderer::{
    ArtifactMetadata, RenderArtifact, RenderMonolithRequest, RenderResult, RenderUrlRequest,
    RendererClient,
};
use ind_application::storage::ObjectStorage;
use ind_domain::{ItemId, UserId};

pub struct StorageBackedMockRenderer {
    storage: Arc<dyn ObjectStorage>,
}

impl StorageBackedMockRenderer {
    pub fn new(storage: Arc<dyn ObjectStorage>) -> Self {
        Self { storage }
    }

    pub fn storage(&self) -> Arc<dyn ObjectStorage> {
        self.storage.clone()
    }

    async fn render_outputs(
        &self,
        user_id: UserId,
        item_id: ItemId,
        requested_outputs: &[String],
        final_url: Option<String>,
    ) -> Result<RenderResult, AppError> {
        let mut artifacts = Vec::new();
        for kind in requested_outputs {
            let Some((extension, content_type, bytes)) = artifact_payload(kind) else {
                continue;
            };
            let upload = self
                .storage
                .upload(
                    &format!("{user_id}/{item_id}/{kind}.{extension}"),
                    content_type,
                    Bytes::from(bytes),
                )
                .await?;
            artifacts.push(RenderArtifact {
                kind: kind.clone(),
                s3_key: upload.key,
                content_type: content_type.into(),
                size_bytes: upload.size_bytes,
                metadata: metadata_for(kind),
            });
        }
        Ok(RenderResult {
            artifacts,
            asset_errors: vec![],
            wall_time_ms: 50,
            final_url,
        })
    }
}

#[async_trait::async_trait]
impl RendererClient for StorageBackedMockRenderer {
    async fn render_url(&self, req: RenderUrlRequest) -> Result<RenderResult, AppError> {
        self.render_outputs(req.user_id, req.item_id, &req.outputs, Some(req.url))
            .await
    }

    async fn render_monolith(&self, req: RenderMonolithRequest) -> Result<RenderResult, AppError> {
        self.render_outputs(req.user_id, req.item_id, &req.outputs, None)
            .await
    }
}

fn metadata_for(kind: &str) -> Option<ArtifactMetadata> {
    (kind == "readable_html").then(|| ArtifactMetadata {
        title: Some("Test Article".into()),
        byline: Some("Test Author".into()),
        excerpt: Some(
            "This is a stable integration-test article used by the save pipeline tests.".into(),
        ),
        word_count: Some(150),
        reading_time_minutes: Some(1),
        domain: Some("example.com".into()),
        lead_image: Some("https://example.com/mock-lead.jpg".into()),
    })
}

fn artifact_payload(kind: &str) -> Option<(&'static str, &'static str, Vec<u8>)> {
    match kind {
        "readable_html" | "monolith" => Some((
            "html",
            "text/html",
            include_bytes!("../fixtures/article_simple.html").to_vec(),
        )),
        "pdf" => Some((
            "pdf",
            "application/pdf",
            b"%PDF-1.4\n% indelible-test\n".to_vec(),
        )),
        "screenshot" | "thumbnail" => Some((
            "png",
            "image/png",
            // Minimal PNG signature plus deterministic bytes. The tests only need
            // storage-backed objects, not image decoding.
            b"\x89PNG\r\n\x1a\nindelible-test".to_vec(),
        )),
        _ => None,
    }
}
