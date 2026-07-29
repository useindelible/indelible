pub const APPROX_CHARS_PER_TOKEN: usize = 4;
pub const HIGHLIGHT_WINDOW_TOKENS: usize = 500;

pub fn approximate_token_count(text: &str) -> usize {
    let (ascii, non_ascii) = text.chars().fold((0_usize, 0_usize), |counts, character| {
        if character.is_ascii() {
            (counts.0 + 1, counts.1)
        } else {
            (counts.0, counts.1 + 1)
        }
    });
    ascii
        .div_ceil(APPROX_CHARS_PER_TOKEN)
        .saturating_add(non_ascii)
}

pub fn exceeds_context_threshold(text: &str, threshold: i32) -> bool {
    approximate_token_count(text) >= threshold.max(1) as usize
}

/// Tokens reserved out of the window for the chat reply plus a baseline of system prompt and the
/// user's question, so even `chat_context_pct = 100` can never inline a document that leaves no
/// room for the response. Larger per-conversation history headroom comes from the (100 - pct)% slack.
pub const CHAT_RESPONSE_RESERVE_TOKENS: i32 = 2048;

/// Token budget for stuffing a document inline into a chat turn before switching to RAG: the
/// smaller of `chat_context_pct` percent of the window and the window minus the response reserve,
/// so the inlined document can never crowd out the reply regardless of the configured percent.
pub fn chat_inline_token_budget(model_context_window: i32, chat_context_pct: i32) -> i32 {
    let pct_budget = model_context_window.saturating_mul(chat_context_pct) / 100;
    let response_safe = model_context_window.saturating_sub(CHAT_RESPONSE_RESERVE_TOKENS);
    pct_budget.min(response_safe).max(1)
}

pub fn chars_for_tokens(tokens: usize) -> usize {
    tokens.saturating_mul(APPROX_CHARS_PER_TOKEN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_estimate_is_compact_for_ascii_and_conservative_for_non_ascii() {
        assert_eq!(approximate_token_count("abcdefgh"), 2);
        assert_eq!(approximate_token_count("你好世界"), 4);
        assert_eq!(approximate_token_count("abcd你好"), 3);
    }
}
