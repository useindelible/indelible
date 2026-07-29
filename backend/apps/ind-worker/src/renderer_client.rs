use std::time::Duration;

use ind_application::error::AppError;
use ind_application::renderer::{
    RenderMonolithRequest, RenderResult, RenderUrlRequest, RendererClient,
};

const RENDERER_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct HttpRendererClient {
    client: reqwest::Client,
    base_url: String,
}

impl HttpRendererClient {
    pub fn new(base_url: &str) -> Self {
        #[expect(
            clippy::expect_used,
            reason = "reqwest client builds from a static timeout config; construction is infallible"
        )]
        let client = reqwest::Client::builder()
            .connect_timeout(RENDERER_CONNECT_TIMEOUT)
            .build()
            .expect("failed to build renderer HTTP client");
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait::async_trait]
impl RendererClient for HttpRendererClient {
    async fn render_url(&self, req: RenderUrlRequest) -> Result<RenderResult, AppError> {
        let url = format!("{}/render/url", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| AppError::ExternalService {
                service: "renderer".into(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            // 422 = the renderer refused the URL (SSRF pre-flight / invalid URL).
            // This is permanent: surface as a validation error so the job is not
            // retried.
            if status == 422 {
                return Err(AppError::Domain(ind_domain::DomainError::Validation {
                    field: "url".into(),
                    message: format!("renderer rejected url: {body}"),
                }));
            }
            return Err(AppError::ExternalService {
                service: "renderer".into(),
                message: format!("renderer returned HTTP {status}: {body}"),
            });
        }

        resp.json::<RenderResult>()
            .await
            .map_err(|e| AppError::ExternalService {
                service: "renderer".into(),
                message: e.to_string(),
            })
    }

    async fn render_monolith(&self, req: RenderMonolithRequest) -> Result<RenderResult, AppError> {
        let url = format!("{}/render/monolith", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| AppError::ExternalService {
                service: "renderer".into(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            // 422 = the renderer refused the URL (SSRF pre-flight / invalid URL).
            // This is permanent: surface as a validation error so the job is not
            // retried.
            if status == 422 {
                return Err(AppError::Domain(ind_domain::DomainError::Validation {
                    field: "url".into(),
                    message: format!("renderer rejected url: {body}"),
                }));
            }
            return Err(AppError::ExternalService {
                service: "renderer".into(),
                message: format!("renderer returned HTTP {status}: {body}"),
            });
        }

        resp.json::<RenderResult>()
            .await
            .map_err(|e| AppError::ExternalService {
                service: "renderer".into(),
                message: e.to_string(),
            })
    }
}
