use std::collections::VecDeque;

use async_trait::async_trait;
use futures::StreamExt;
use futures::stream::BoxStream;
use ind_egress::{
    EgressPolicy, GuardedClientOptions, GuardedHttpClient, UrlRules, build_guarded_client,
};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use secrecy::ExposeSecret;
use serde::Serialize;

use crate::error::{AiError, map_error_response};
use crate::sse::{ParsedSseEvent, SseDecoder};
use crate::types::{
    AiHttpClientConfig, AiProviderConfig, ApiEmbeddingResponse, ChatCompletionChunk,
    ChatCompletionRequest, ChatCompletionResponse, EmbeddingRequest, EmbeddingResponse,
};

pub type ChatCompletionStream = BoxStream<'static, Result<ChatCompletionChunk, AiError>>;

#[async_trait]
pub trait AiProviderClient: Send + Sync {
    async fn chat_completion(
        &self,
        provider: &AiProviderConfig,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiError>;

    async fn chat_completion_stream(
        &self,
        provider: &AiProviderConfig,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream, AiError>;

    async fn embedding(
        &self,
        provider: &AiProviderConfig,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, AiError>;
}

#[derive(Clone)]
pub struct ReqwestAiProviderClient {
    client: GuardedHttpClient,
}

impl ReqwestAiProviderClient {
    /// Build the provider client. The `policy` guards the user-configurable
    /// `api_base`: requests to private/internal hosts are refused, and BYOK
    /// keys are never sent in cleartext to a public host (https required).
    pub fn new(config: AiHttpClientConfig, policy: EgressPolicy) -> Result<Self, AiError> {
        let client = build_guarded_client(
            GuardedClientOptions::new(UrlRules::ai_endpoint(), policy)
                .connect_timeout(config.connect_timeout)
                .request_timeout(config.request_timeout)
                .pool_idle_timeout(config.pool_idle_timeout)
                .pool_max_idle_per_host(config.max_idle_per_host),
        )
        .map_err(|err| AiError::ProviderUnreachable {
            message: err.client_message().to_string(),
        })?;

        Ok(Self { client })
    }

    async fn send_json<TBody: Serialize>(
        &self,
        provider: &AiProviderConfig,
        path: &str,
        body: &TBody,
    ) -> Result<reqwest::Response, AiError> {
        let url = provider.endpoint_url(path);
        let response = self
            .client
            .post(&url)
            .map_err(|err| AiError::EndpointDisallowed {
                message: err.client_message().to_string(),
            })?
            .headers(headers_for(provider)?)
            .json(body)
            .send()
            .await
            .map_err(AiError::from_transport)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(map_error_response(response).await)
        }
    }
}

#[async_trait]
impl AiProviderClient for ReqwestAiProviderClient {
    async fn chat_completion(
        &self,
        provider: &AiProviderConfig,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiError> {
        let response = self
            .send_json(
                provider,
                "chat/completions",
                &ChatCompletionRequestBody::new(request, false),
            )
            .await?;

        response.json().await.map_err(AiError::from_decode)
    }

    async fn chat_completion_stream(
        &self,
        provider: &AiProviderConfig,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream, AiError> {
        let response = self
            .send_json(
                provider,
                "chat/completions",
                &ChatCompletionRequestBody::new(request, true),
            )
            .await?;

        let stream = futures::stream::unfold(
            StreamState {
                bytes_stream: response.bytes_stream().boxed(),
                decoder: SseDecoder::default(),
                pending: VecDeque::new(),
                seen_done: false,
            },
            |mut state| async move {
                loop {
                    if let Some(chunk) = state.pending.pop_front() {
                        return Some((Ok(chunk), state));
                    }

                    if state.seen_done {
                        return None;
                    }

                    match state.bytes_stream.next().await {
                        Some(Ok(bytes)) => match state.decoder.push(&bytes) {
                            Ok(events) => {
                                for event in events {
                                    match event {
                                        ParsedSseEvent::Done => state.seen_done = true,
                                        ParsedSseEvent::Json(payload) => {
                                            match serde_json::from_str::<ChatCompletionChunk>(
                                                &payload,
                                            ) {
                                                Ok(chunk) => state.pending.push_back(chunk),
                                                Err(err) => {
                                                    state.seen_done = true;
                                                    return Some((
                                                        Err(AiError::from_decode(err)),
                                                        state,
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                state.seen_done = true;
                                return Some((Err(err), state));
                            }
                        },
                        Some(Err(err)) => {
                            state.seen_done = true;
                            return Some((Err(AiError::from_stream(err)), state));
                        }
                        None => {
                            state.seen_done = true;
                            return Some((
                                Err(AiError::from_stream("provider stream ended before [DONE]")),
                                state,
                            ));
                        }
                    }
                }
            },
        )
        .boxed();

        Ok(stream)
    }

    async fn embedding(
        &self,
        provider: &AiProviderConfig,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, AiError> {
        let response = self.send_json(provider, "embeddings", &request).await?;
        let payload: ApiEmbeddingResponse = response.json().await.map_err(AiError::from_decode)?;
        let embedding =
            payload
                .data
                .into_iter()
                .next()
                .ok_or_else(|| AiError::MalformedResponse {
                    message: "embedding response contained no vectors".into(),
                })?;

        Ok(EmbeddingResponse {
            model: payload.model,
            embedding: embedding.embedding,
            usage: payload.usage,
        })
    }
}

struct StreamState {
    bytes_stream: BoxStream<'static, Result<bytes::Bytes, reqwest::Error>>,
    decoder: SseDecoder,
    pending: VecDeque<ChatCompletionChunk>,
    seen_done: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ChatCompletionRequestBody {
    model: String,
    messages: Vec<crate::types::ChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<crate::types::ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<crate::types::ReasoningEffort>,
}

impl ChatCompletionRequestBody {
    fn new(request: ChatCompletionRequest, stream: bool) -> Self {
        Self {
            model: request.model,
            messages: request.messages,
            stream,
            temperature: request.temperature,
            top_p: request.top_p,
            max_completion_tokens: request.max_completion_tokens,
            user: request.user,
            response_format: request.response_format,
            reasoning_effort: request.reasoning_effort,
        }
    }
}

fn headers_for(provider: &AiProviderConfig) -> Result<HeaderMap, AiError> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    if let Some(api_key) = provider.api_key.as_ref() {
        let value = format!("Bearer {}", api_key.expose_secret());
        let header = HeaderValue::from_str(&value).map_err(AiError::from_decode)?;
        headers.insert(AUTHORIZATION, header);
    }

    Ok(headers)
}
