#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::unimplemented,
        clippy::todo
    )
)]

mod actions;
mod chat;
mod chunker;
mod client;
mod content;
mod defaults;
mod embed;
mod error;
mod operations;
mod prompt;
mod provider_test;
mod resolution;
mod sse;
#[cfg(test)]
mod test_support;
mod token_estimate;
pub mod tts;
mod types;
mod untrusted;

pub use actions::AiActionRunner;
pub use chat::{
    CompletedChatTurn, MilaChatDelta, MilaChatRequest, MilaChatService, MilaChatStream,
};
pub use chunker::{ChunkingConfig, TextChunk, approximate_token_count, chunk_text};
pub use client::{AiProviderClient, ChatCompletionStream, ReqwestAiProviderClient};
pub use content::{chat_provider_from_config, embedding_provider_from_config};
pub use defaults::{BuiltInPromptPreset, built_in_prompt_presets};
pub use embed::EmbeddingIndexer;
pub use error::AiError;
pub use operations::{MilaOperationsDeps, MilaOperationsService};
pub use provider_test::MilaProviderTestService;
pub use token_estimate::{
    APPROX_CHARS_PER_TOKEN, HIGHLIGHT_WINDOW_TOKENS,
    approximate_token_count as approximate_chat_tokens, chars_for_tokens,
    exceeds_context_threshold,
};
pub use types::{
    AiHttpClientConfig, AiProviderConfig, ChatCompletionChoice, ChatCompletionChunk,
    ChatCompletionChunkChoice, ChatCompletionRequest, ChatCompletionResponse, ChatMessage,
    ChatMessageDelta, EmbeddingRequest, EmbeddingResponse, JsonSchemaFormat, ReasoningEffort,
    ResponseFormat, UsageStats,
};
