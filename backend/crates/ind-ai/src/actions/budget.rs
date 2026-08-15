use ind_domain::AiPromptAction;

use crate::token_estimate::{MilaTokenBudgets, approximate_token_count};

/// Floor for the per-action document input cap (chars/4 token estimate). Matches the previous
/// `context_threshold.max(512)` behaviour so a misconfigured window never yields a zero/negative cap.
pub(super) const MIN_ACTION_INPUT_TOKENS: i32 = 512;

/// Fixed cushion (tokens) reserved on top of the measured system prompt and the per-action output
/// budget. Covers the structured-output instruction line appended after truncation and the chars/4
/// imprecision in `approximate_token_count`.
const SAFETY_MARGIN_TOKENS: i32 = 256;

/// Product limit applied to generated summaries independently of the model token budget.
pub(super) const SUMMARY_WORD_LIMIT: usize = 120;

/// Tokens available for the document input, reserving the (measured) system prompt, the per-action
/// output budget, and a fixed safety margin out of the model's total context window. Floored at
/// `MIN_ACTION_INPUT_TOKENS`.
pub(super) fn action_input_budget(
    model_context_window: i32,
    system_prompt: &str,
    action: AiPromptAction,
    budgets: MilaTokenBudgets,
) -> i32 {
    let system_tokens = i32::try_from(approximate_token_count(system_prompt)).unwrap_or(i32::MAX);
    model_context_window
        .saturating_sub(system_tokens)
        .saturating_sub(budgets.action_output_tokens(action) as i32)
        .saturating_sub(SAFETY_MARGIN_TOKENS)
        .max(MIN_ACTION_INPUT_TOKENS)
}
