use ind_domain::AiPromptAction;

use crate::token_estimate::approximate_token_count;

/// Floor for the per-action document input cap (chars/4 token estimate). Matches the previous
/// `context_threshold.max(512)` behaviour so a misconfigured window never yields a zero/negative cap.
pub(super) const MIN_ACTION_INPUT_TOKENS: i32 = 512;

/// Fixed cushion (tokens) reserved on top of the measured system prompt and the per-action output
/// budget. Covers the structured-output instruction line appended after truncation and the chars/4
/// imprecision in `approximate_token_count`.
const SAFETY_MARGIN_TOKENS: i32 = 256;

/// Per-action `max_completion_tokens`. Single source of truth shared by the input-budget calc and
/// the chat-completion request builder in `run_model_completion`.
pub(super) const fn output_budget_tokens(action: AiPromptAction) -> i32 {
    match action {
        AiPromptAction::Summary => 1024,
        AiPromptAction::Tags => 1024,
        AiPromptAction::Entities => 2000,
        AiPromptAction::Chat | AiPromptAction::Custom => 1024,
    }
}

pub(super) const SUMMARY_WORD_LIMIT: usize = 120;

/// Tokens available for the document input, reserving the (measured) system prompt, the per-action
/// output budget, and a fixed safety margin out of the model's total context window. Floored at
/// `MIN_ACTION_INPUT_TOKENS`.
pub(super) fn action_input_budget(
    model_context_window: i32,
    system_prompt: &str,
    action: AiPromptAction,
) -> i32 {
    let system_tokens = i32::try_from(approximate_token_count(system_prompt)).unwrap_or(i32::MAX);
    model_context_window
        .saturating_sub(system_tokens)
        .saturating_sub(output_budget_tokens(action))
        .saturating_sub(SAFETY_MARGIN_TOKENS)
        .max(MIN_ACTION_INPUT_TOKENS)
}
