use async_trait::async_trait;
use ind_egress::{
    EgressPolicy, GuardedClientOptions, GuardedHttpClient, UrlRules, build_guarded_client,
};
use reqwest::header::{CONTENT_TYPE, HeaderName, HeaderValue};

use ind_application::ports::{FetchRequest, FetchResponse, HttpFetchError, HttpFetcher};

const USER_AGENT: &str = "Indelible/1.0 (+https://useindelible.com)";

/// Build an ingest-class guarded HTTP client (http/https, SSRF-filtered DNS and
/// redirects) from an [`EgressPolicy`].
pub fn build_ingest_http_client(
    policy: EgressPolicy,
) -> Result<GuardedHttpClient, ind_egress::EgressError> {
    build_guarded_client(
        GuardedClientOptions::new(UrlRules::ingest(), policy).user_agent(USER_AGENT),
    )
}

pub struct ReqwestHttpFetcher {
    client: GuardedHttpClient,
}

impl ReqwestHttpFetcher {
    pub fn new(client: GuardedHttpClient) -> Self {
        Self { client }
    }

    /// Convenience constructor that builds the guarded client from a policy.
    pub fn with_policy(policy: EgressPolicy) -> Result<Self, ind_egress::EgressError> {
        Ok(Self::new(build_ingest_http_client(policy)?))
    }
}

#[async_trait]
impl HttpFetcher for ReqwestHttpFetcher {
    async fn fetch(&self, request: FetchRequest) -> Result<FetchResponse, HttpFetchError> {
        let mut builder = self
            .client
            .get(&request.url)
            .map_err(|err| HttpFetchError::Disallowed(err.client_message().to_string()))?;
        for (name, value) in &request.headers {
            let header_name = HeaderName::try_from(name.as_str())
                .map_err(|err| HttpFetchError::Send(format!("invalid header name: {err}")))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|err| HttpFetchError::Send(format!("invalid header value: {err}")))?;
            builder = builder.header(header_name, header_value);
        }

        let response = builder
            .send()
            .await
            .map_err(|err| HttpFetchError::Send(err.to_string()))?;

        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        let body = response
            .bytes()
            .await
            .map_err(|err| HttpFetchError::Body(err.to_string()))?;

        Ok(FetchResponse {
            status,
            content_type,
            body,
        })
    }
}
