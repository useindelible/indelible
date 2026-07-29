pub mod adapter;
pub mod dashscope;
pub mod http;
pub mod mock;
pub mod registry;
pub mod unreal;

pub use adapter::{
    TtsAdapter, TtsAdapterError, TtsDesignRequest, TtsDesignResult, TtsSynthesisRequest,
    TtsSynthesisResult,
};
pub use dashscope::{
    DASHSCOPE_DEFAULT_BASE, DEFAULT_SYNTHESIS_MODEL, DEFAULT_VOICE_DESIGN_TARGET_MODEL,
    DashScopeAdapter,
};
pub use mock::MockTtsAdapter;
pub use registry::TtsAdapterRegistry;
pub use unreal::{UNREAL_DEFAULT_BASE, UnrealSpeechAdapter};
