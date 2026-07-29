use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct DashScopeSynthesisResponse {
    pub(super) output: DashScopeSynthesisOutput,
    #[serde(default)]
    pub(super) usage: Option<DashScopeSynthesisUsage>,
}

#[derive(Debug, Deserialize)]
pub(super) struct DashScopeSynthesisOutput {
    pub(super) audio: DashScopeAudioPayload,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct DashScopeAudioPayload {
    #[serde(default)]
    pub(super) data: Option<String>,
    #[serde(default)]
    pub(super) url: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct DashScopeSynthesisUsage {
    #[serde(default)]
    pub(super) tts_tokens: Option<i64>,
    #[serde(default)]
    pub(super) output_tokens: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(super) struct DashScopeDesignResponse {
    #[serde(default)]
    pub(super) request_id: Option<String>,
    pub(super) output: DashScopeDesignOutput,
}

#[derive(Debug, Deserialize)]
pub(super) struct DashScopeDesignOutput {
    #[serde(default)]
    pub(super) voice: Option<String>,
}
