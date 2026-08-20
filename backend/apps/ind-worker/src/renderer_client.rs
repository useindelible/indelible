use std::time::Duration;

use ind_application::error::AppError;
use ind_application::renderer::{
    RenderMonolithRequest, RenderResult, RenderUrlRequest, RendererClient,
};
use serde::Serialize;

const RENDERER_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct HttpRendererClient {
    client: reqwest::Client,
    base_url: String,
    request_timeout: Duration,
}

impl HttpRendererClient {
    /// `request_timeout` bounds the whole exchange, not just the connection: a capture the
    /// renderer never answers must still hand the job back to the retry policy instead of
    /// holding a worker slot open indefinitely.
    pub fn new(base_url: &str, request_timeout: Duration) -> Self {
        #[expect(
            clippy::expect_used,
            reason = "reqwest client builds from a static timeout config; construction is infallible"
        )]
        let client = reqwest::Client::builder()
            .connect_timeout(RENDERER_CONNECT_TIMEOUT)
            .timeout(request_timeout)
            .build()
            .expect("failed to build renderer HTTP client");
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            request_timeout,
        }
    }

    async fn post(&self, path: &str, body: &impl Serialize) -> Result<RenderResult, AppError> {
        let response = self
            .client
            .post(format!("{}{path}", self.base_url))
            .json(body)
            .send()
            .await
            .map_err(|error| self.transport_error(error))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            // 422 = the renderer refused the URL (SSRF pre-flight / invalid URL).
            // This is permanent: surface as a validation error so the job is not
            // retried.
            if status == 422 {
                return Err(AppError::Domain(ind_domain::DomainError::Validation {
                    field: "url".into(),
                    message: format!("renderer rejected url: {body}"),
                }));
            }
            return Err(renderer_error(format!(
                "renderer returned HTTP {status}: {body}"
            )));
        }

        response
            .json::<RenderResult>()
            .await
            .map_err(|error| self.transport_error(error))
    }

    fn transport_error(&self, error: reqwest::Error) -> AppError {
        if error.is_timeout() {
            return renderer_error(format!(
                "renderer request timed out after {}s",
                self.request_timeout.as_secs_f64()
            ));
        }
        renderer_error(error.to_string())
    }
}

fn renderer_error(message: String) -> AppError {
    AppError::ExternalService {
        service: "renderer".into(),
        message,
    }
}

#[async_trait::async_trait]
impl RendererClient for HttpRendererClient {
    async fn render_url(&self, req: RenderUrlRequest) -> Result<RenderResult, AppError> {
        self.post("/render/url", &req).await
    }

    async fn render_monolith(&self, req: RenderMonolithRequest) -> Result<RenderResult, AppError> {
        self.post("/render/monolith", &req).await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use ind_domain::{ItemId, UserId};

    use super::*;

    /// Accepts connections and holds them open without ever writing a byte back.
    async fn silent_server() -> (String, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let mut held_open = Vec::new();
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                held_open.push(socket);
            }
        });
        (format!("http://{address}"), task)
    }

    #[tokio::test]
    async fn a_renderer_that_never_answers_fails_within_the_request_timeout() {
        let (base_url, server) = silent_server().await;
        let client = HttpRendererClient::new(&base_url, Duration::from_millis(300));

        let started = Instant::now();
        let error = client
            .render_url(RenderUrlRequest {
                item_id: ItemId::new(),
                user_id: UserId::new(),
                url: "https://example.com/article".into(),
                outputs: vec!["readable_html".into()],
            })
            .await
            .unwrap_err();

        assert!(started.elapsed() < Duration::from_secs(5));
        match error {
            AppError::ExternalService { service, message } => {
                assert_eq!(service, "renderer");
                assert!(message.contains("timed out after 0.3s"), "{message}");
            }
            other => panic!("expected a renderer timeout, got {other:?}"),
        }
        server.abort();
    }
}
