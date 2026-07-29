// The adapter trait and DTOs live in `ind-application::ports::tts_adapter`
// so the application layer can depend on them without importing ind-ai.
// This module re-exports them for ind-ai's concrete adapter impls and for
// the http-api layer, which wires adapters into the registry.

pub use ind_application::ports::{
    TtsAdapter, TtsAdapterError, TtsDesignRequest, TtsDesignResult, TtsSynthesisRequest,
    TtsSynthesisResult,
};
