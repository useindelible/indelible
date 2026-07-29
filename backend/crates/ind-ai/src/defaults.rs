use ind_domain::AiPromptAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltInPromptPreset {
    pub action: AiPromptAction,
    pub name: &'static str,
    pub system_prompt: &'static str,
}

pub fn built_in_prompt_presets() -> &'static [BuiltInPromptPreset] {
    &[]
}
