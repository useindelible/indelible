use std::sync::Mutex;

use async_trait::async_trait;
use futures::stream;

use crate::{
    AiError, AiProviderClient, AiProviderConfig, ChatCompletionChunk, ChatCompletionChunkChoice,
    ChatCompletionRequest, ChatCompletionResponse, ChatCompletionStream, ChatMessageDelta,
    EmbeddingRequest, EmbeddingResponse, approximate_token_count,
};

pub(crate) struct ScriptedAiProvider {
    pub(crate) chat_response: Mutex<String>,
    pub(crate) chat_requests: Mutex<Vec<ChatCompletionRequest>>,
    pub(crate) embedding_inputs: Mutex<Vec<String>>,
    pub(crate) fail_chat_stream: Mutex<bool>,
    pub(crate) fail_embedding: Mutex<bool>,
    pub(crate) fail_embedding_call: Mutex<Option<usize>>,
    pub(crate) embedding_context_limit: Mutex<Option<usize>>,
}

impl Default for ScriptedAiProvider {
    fn default() -> Self {
        Self {
            chat_response: Mutex::new("answer".into()),
            chat_requests: Mutex::new(Vec::new()),
            embedding_inputs: Mutex::new(Vec::new()),
            fail_chat_stream: Mutex::new(false),
            fail_embedding: Mutex::new(false),
            fail_embedding_call: Mutex::new(None),
            embedding_context_limit: Mutex::new(None),
        }
    }
}

#[async_trait]
impl AiProviderClient for ScriptedAiProvider {
    async fn chat_completion(
        &self,
        _: &AiProviderConfig,
        _: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiError> {
        unreachable!()
    }

    async fn chat_completion_stream(
        &self,
        _: &AiProviderConfig,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream, AiError> {
        self.chat_requests.lock().unwrap().push(request);
        if *self.fail_chat_stream.lock().unwrap() {
            return Err(AiError::ProviderUnreachable {
                message: "scripted provider offline".into(),
            });
        }
        let content = self.chat_response.lock().unwrap().clone();
        Ok(Box::pin(stream::iter([Ok(ChatCompletionChunk {
            id: None,
            model: None,
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatMessageDelta {
                    role: None,
                    content: Some(content),
                },
                finish_reason: None,
            }],
            usage: None,
        })])))
    }

    async fn embedding(
        &self,
        _: &AiProviderConfig,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, AiError> {
        let mut inputs = self.embedding_inputs.lock().unwrap();
        inputs.push(request.input.clone());
        let call = inputs.len();
        let exceeds_context = self
            .embedding_context_limit
            .lock()
            .unwrap()
            .is_some_and(|limit| approximate_token_count(&request.input) > limit);
        if *self.fail_embedding.lock().unwrap()
            || *self.fail_embedding_call.lock().unwrap() == Some(call)
            || exceeds_context
        {
            return Err(AiError::ProviderError {
                status_code: if exceeds_context { 400 } else { 503 },
                message: if exceeds_context {
                    "maximum context length exceeded"
                } else {
                    "provider unavailable"
                }
                .into(),
            });
        }
        Ok(EmbeddingResponse {
            model: request.model,
            embedding: vec![0.1; 4],
            usage: None,
        })
    }
}
