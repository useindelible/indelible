use ind_domain::Document;

use crate::chunker::approximate_token_count;
use crate::untrusted::{GUIDANCE, fence};

pub(super) fn build_document_user_prompt(
    document: &Document,
    plain_text: &str,
    input_budget_tokens: i32,
) -> String {
    let mut lines = vec![
        format!("title: {}", document.title),
        format!("type: {}", document.document_type.as_str()),
    ];

    if let Some(author) = document.author.as_deref() {
        lines.push(format!("author: {author}"));
    }
    if let Some(url) = document
        .canonical_url
        .as_deref()
        .or(document.original_url.as_deref())
    {
        lines.push(format!("url: {url}"));
    }
    if let Some(domain) = document.domain.as_deref() {
        lines.push(format!("domain: {domain}"));
    }
    if let Some(excerpt) = document.excerpt.as_deref() {
        lines.push(format!("excerpt: {excerpt}"));
    }

    let max_tokens = input_budget_tokens.max(super::budget::MIN_ACTION_INPUT_TOKENS) as usize;
    let truncated = truncate_to_approx_tokens(plain_text, max_tokens);
    lines.push("content:".into());
    lines.push(truncated);

    format!(
        "Summarize or analyze the following saved library item using only the provided \
         information. {GUIDANCE}\n\n{}",
        fence(&lines.join("\n"))
    )
}

fn truncate_to_approx_tokens(text: &str, max_tokens: usize) -> String {
    if approximate_token_count(text) <= max_tokens {
        return text.to_string();
    }

    let target_chars = max_tokens.saturating_mul(4);
    let mut truncated = text.chars().take(target_chars).collect::<String>();
    if let Some(last_whitespace) = truncated.rfind(char::is_whitespace) {
        truncated.truncate(last_whitespace);
    }
    format!("{}\n\n[truncated]", truncated.trim_end())
}
