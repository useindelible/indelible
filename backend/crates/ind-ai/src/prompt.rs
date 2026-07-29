use ind_domain::Document;

use crate::types::ChatMessage;
use crate::untrusted::{GUIDANCE, fence};

#[derive(Debug, Clone)]
pub struct RetrievedPassage {
    pub source_label: String,
    pub title: String,
    pub snippet: String,
    pub child_excerpt: Option<String>,
    pub section_title: Option<String>,
    pub url: Option<String>,
}

pub fn build_single_document_stuffing_messages(
    system_prompt: &str,
    metadata: &str,
    full_text: &str,
    history: &[ChatMessage],
    question: &str,
) -> Vec<ChatMessage> {
    let mut messages = vec![
        ChatMessage::system(base_chat_system_prompt(system_prompt, false)),
        ChatMessage::system(fence(&format!(
            "Source context:\n{metadata}\n\nFull text:\n{full_text}"
        ))),
    ];
    messages.extend_from_slice(history);
    messages.push(ChatMessage::user(question));
    messages
}

pub fn build_single_document_rag_messages(
    system_prompt: &str,
    metadata: &str,
    passages: &[RetrievedPassage],
    history: &[ChatMessage],
    question: &str,
) -> Vec<ChatMessage> {
    let mut messages = vec![
        ChatMessage::system(base_chat_system_prompt(system_prompt, !passages.is_empty())),
        ChatMessage::system(fence(&format!(
            "Source context:\n{}\n\nRelevant passages:\n{}",
            metadata,
            format_passages(passages)
        ))),
    ];
    messages.extend_from_slice(history);
    messages.push(ChatMessage::user(question));
    messages
}

#[allow(clippy::too_many_arguments)]
pub fn build_highlight_messages(
    system_prompt: &str,
    metadata: &str,
    highlight_text: &str,
    window_text: &str,
    full_text: Option<&str>,
    passages: &[RetrievedPassage],
    history: &[ChatMessage],
    question: &str,
) -> Vec<ChatMessage> {
    // Passages are only rendered when there is no inline full text to stuff.
    let mut messages = vec![ChatMessage::system(base_chat_system_prompt(
        system_prompt,
        full_text.is_none() && !passages.is_empty(),
    ))];

    if let Some(full_text) = full_text {
        messages.push(ChatMessage::system(fence(&format!(
            "Source context:\n{metadata}\n\nFull text:\n{full_text}"
        ))));
    } else {
        messages.push(ChatMessage::system(fence(&format!(
            "Source context:\n{}\n\nRelevant passages:\n{}",
            metadata,
            format_passages(passages)
        ))));
    }

    messages.push(ChatMessage::system(fence(&format!(
        "The user highlighted this passage:\n> {}\n\nSurrounding context:\n{}",
        highlight_text, window_text
    ))));
    messages.extend_from_slice(history);
    messages.push(ChatMessage::user(question));
    messages
}

pub fn build_cross_item_messages(
    system_prompt: &str,
    scope_label: &str,
    passages: &[RetrievedPassage],
    history: &[ChatMessage],
    question: &str,
) -> Vec<ChatMessage> {
    let mut messages = vec![
        ChatMessage::system(base_chat_system_prompt(system_prompt, !passages.is_empty())),
        ChatMessage::system(fence(&format!(
            "Scope: {scope_label}\n\nRelevant passages:\n{}",
            format_passages(passages)
        ))),
    ];
    messages.extend_from_slice(history);
    messages.push(ChatMessage::user(question));
    messages
}

/// `citable_sources` must be true only when the turn supplies labelled `[S<n>]` passages.
/// Asking for citations without them makes models emit placeholder tokens such as `[S_]`,
/// which resolve to no source and render as raw text in every client.
fn base_chat_system_prompt(base: &str, citable_sources: bool) -> String {
    let citation_rules = if citable_sources {
        "Cite evidence by appending bare source tokens such as [S1]. Use only the source labels listed in the supplied sources; never invent labels that are not listed. For multiple supporting sources, write adjacent tokens such as [S1][S2]. Never repeat the same source token in one citation cluster; write [S1], not [S1][S1]. Never place words or punctuation between adjacent source tokens. Source tokens are placeholders for inline source chips, not prose."
    } else {
        "No source labels are supplied for this answer. Never write source tokens or citation markers of any kind."
    };
    format!(
        "{base} Answer directly first. Use only the supplied reading context. {citation_rules} Do not describe the retrieval mechanism. Do not begin with phrases like \"based on the provided context\" or \"from the passages supplied\". If the saved content does not contain the answer, say that once and name the specific missing fact. {GUIDANCE}"
    )
}

pub(crate) fn format_document_metadata(document: &Document) -> String {
    let mut lines = vec![format!("Title: {}", document.title)];

    if let Some(author) = document.author.as_deref() {
        lines.push(format!("Author: {author}"));
    }
    if let Some(url) = document
        .canonical_url
        .as_deref()
        .or(document.original_url.as_deref())
    {
        lines.push(format!("URL: {url}"));
    }
    lines.push(format!("Saved: {}", document.created_at.to_rfc3339()));

    lines.join("\n")
}

fn format_passages(passages: &[RetrievedPassage]) -> String {
    if passages.is_empty() {
        return "No retrieved passages were available.".into();
    }

    passages
        .iter()
        .map(|passage| {
            let mut header = format!("[{}] {}", passage.source_label, passage.title);
            if let Some(section_title) = passage.section_title.as_deref() {
                header.push_str(&format!(" - {section_title}"));
            }
            if let Some(url) = passage.url.as_deref() {
                header.push_str(&format!(" ({url})"));
            }
            let mut lines = vec![header];
            if let Some(child_excerpt) = passage.child_excerpt.as_deref() {
                lines.push(format!("Matched excerpt: {child_excerpt}"));
            }
            lines.push(format!("Context: {}", passage.snippet));
            lines.join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passage() -> RetrievedPassage {
        RetrievedPassage {
            source_label: "S1".into(),
            title: "Oyo Empire".into(),
            snippet: "The Nupe sacked the capital.".into(),
            child_excerpt: None,
            section_title: None,
            url: None,
        }
    }

    /// Asking for `[S1]` citations without offering labels makes models emit placeholder
    /// tokens like `[S_]`, which resolve to no source and render raw in every client.
    #[test]
    fn citation_instructions_appear_only_when_labelled_sources_are_supplied() {
        let passages = [passage()];
        let cases = [
            (
                "single-document stuffing",
                build_single_document_stuffing_messages("base", "meta", "full text", &[], "q"),
                false,
            ),
            (
                "single-document rag with passages",
                build_single_document_rag_messages("base", "meta", &passages, &[], "q"),
                true,
            ),
            (
                "single-document rag with no hits",
                build_single_document_rag_messages("base", "meta", &[], &[], "q"),
                false,
            ),
            (
                "highlight over inline full text",
                build_highlight_messages("base", "meta", "h", "w", Some("full"), &[], &[], "q"),
                false,
            ),
            (
                "highlight over passages",
                build_highlight_messages("base", "meta", "h", "w", None, &passages, &[], "q"),
                true,
            ),
            (
                "cross-item with passages",
                build_cross_item_messages("base", "your library", &passages, &[], "q"),
                true,
            ),
            (
                "cross-item with no hits",
                build_cross_item_messages("base", "your library", &[], &[], "q"),
                false,
            ),
        ];

        for (name, messages, expects_citations) in cases {
            let system = &messages[0].content;
            assert_eq!(
                system.contains("[S1]"),
                expects_citations,
                "{name}: unexpected citation instructions in system prompt"
            );
        }
    }
}
