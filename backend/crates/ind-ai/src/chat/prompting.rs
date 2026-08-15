use ind_application::AppError;
use ind_domain::{DomainError, MilaConfig, UserId};

use crate::token_estimate::{
    approximate_token_count, chars_for_tokens, chat_response_reserve_tokens,
};
use crate::untrusted::truncate_fenced;
use crate::{ChatCompletionRequest, ChatMessage};

use super::MilaChatRequest;

pub(super) fn build_chat_request(
    config: &MilaConfig,
    user_id: UserId,
    messages: Vec<ChatMessage>,
    max_output_tokens: u32,
) -> ChatCompletionRequest {
    let messages = fit_messages_to_context_window(
        messages,
        config.model_context_window,
        max_output_tokens as i32,
    );
    let mut request = ChatCompletionRequest::new(config.chat_model.clone(), messages);
    if !config.supports_reasoning_effort {
        request.temperature = Some(0.3);
        request.top_p = Some(0.95);
    }
    request.max_completion_tokens = Some(max_output_tokens);
    request.user = Some(user_id.to_string());
    request
}

const MESSAGE_OVERHEAD_TOKENS: usize = 4;
const REQUEST_OVERHEAD_TOKENS: usize = 8;

fn estimated_request_tokens(messages: &[ChatMessage]) -> usize {
    REQUEST_OVERHEAD_TOKENS
        + messages
            .iter()
            .map(|message| approximate_token_count(&message.content) + MESSAGE_OVERHEAD_TOKENS)
            .sum::<usize>()
}

fn fit_messages_to_context_window(
    mut messages: Vec<ChatMessage>,
    model_context_window: i32,
    max_output_tokens: i32,
) -> Vec<ChatMessage> {
    let budget = model_context_window
        .saturating_sub(chat_response_reserve_tokens(max_output_tokens))
        .max(1) as usize;

    while estimated_request_tokens(&messages) > budget && history_tokens(&messages) > budget / 2 {
        let Some(index) = oldest_history_index(&messages) else {
            break;
        };
        messages.remove(index);
    }

    if estimated_request_tokens(&messages) > budget && messages.len() > 2 {
        shrink_context_message(&mut messages, 1, budget);
    }

    while estimated_request_tokens(&messages) > budget {
        let Some(index) = oldest_history_index(&messages) else {
            break;
        };
        messages.remove(index);
    }

    while estimated_request_tokens(&messages) > budget && messages.len() > 2 {
        shrink_context_message(&mut messages, 1, budget);
    }

    if estimated_request_tokens(&messages) > budget {
        tracing::warn!(
            estimated_prompt_tokens = estimated_request_tokens(&messages),
            input_budget_tokens = budget,
            model_context_window,
            "fixed chat prompt content exceeds the configured input budget"
        );
    }
    messages
}

fn oldest_history_index(messages: &[ChatMessage]) -> Option<usize> {
    let last = messages.len().saturating_sub(1);
    messages
        .iter()
        .enumerate()
        .find(|(index, message)| {
            *index > 0 && *index < last && message.role != ind_domain::MessageRole::System
        })
        .map(|(index, _)| index)
}

fn history_tokens(messages: &[ChatMessage]) -> usize {
    let last = messages.len().saturating_sub(1);
    messages
        .iter()
        .enumerate()
        .filter(|(index, message)| {
            *index > 0 && *index < last && message.role != ind_domain::MessageRole::System
        })
        .map(|(_, message)| approximate_token_count(&message.content) + MESSAGE_OVERHEAD_TOKENS)
        .sum()
}

fn shrink_context_message(messages: &mut Vec<ChatMessage>, index: usize, budget: usize) {
    let excess = estimated_request_tokens(messages).saturating_sub(budget);
    let current_tokens = approximate_token_count(&messages[index].content);
    let target_tokens = current_tokens.saturating_sub(excess).saturating_sub(1);
    if target_tokens < 16 {
        messages.remove(index);
        return;
    }

    let previous_len = messages[index].content.len();
    let max_chars = messages[index]
        .content
        .chars()
        .count()
        .saturating_sub(chars_for_tokens(excess.saturating_add(16)));
    match truncate_fenced(&messages[index].content, max_chars) {
        Some(content) if content.len() < previous_len => messages[index].content = content,
        _ => {
            messages.remove(index);
        }
    }
}

pub(super) fn validate_chat_request(request: &MilaChatRequest) -> Result<(), AppError> {
    if request.question.trim().is_empty() {
        return Err(AppError::Domain(DomainError::Validation {
            field: "question".into(),
            message: "question cannot be empty".into(),
        }));
    }

    if request.highlight_offset.is_some()
        && request
            .highlight_text
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
    {
        return Err(AppError::Domain(DomainError::Validation {
            field: "highlight_text".into(),
            message: "highlight_text is required when highlight_offset is provided".into(),
        }));
    }

    Ok(())
}

pub(super) fn validate_question_for_context(
    question: &str,
    model_context_window: i32,
    max_output_tokens: i32,
) -> Result<(), AppError> {
    let input_budget = model_context_window
        .saturating_sub(chat_response_reserve_tokens(max_output_tokens))
        .max(1) as usize;
    let question_budget = (input_budget / 2).max(1);
    if approximate_token_count(question.trim()) > question_budget {
        return Err(AppError::Domain(DomainError::Validation {
            field: "question".into(),
            message: format!(
                "question is too long for the configured model context; limit it to approximately {question_budget} tokens"
            ),
        }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use ind_domain::MessageRole;

    use super::*;
    use crate::untrusted::fence;

    #[test]
    fn chat_request_and_prompt_reserve_follow_the_configured_output_budget() {
        let config = ind_domain::MilaPlatformDefaults {
            chat_max_output_tokens: 4_096,
            ..ind_test_support::test_mila_defaults()
        }
        .materialize(UserId::new(), chrono::Utc::now());
        let request = build_chat_request(&config, UserId::new(), Vec::new(), 4_096);

        assert_eq!(request.max_completion_tokens, Some(4_096));
        assert_eq!(chat_response_reserve_tokens(4_096), 5_120);
    }

    #[test]
    fn prompt_assembly_bounds_oversized_context_and_long_history() {
        let mut messages = vec![
            ChatMessage::system("system guidance".repeat(40)),
            ChatMessage::system(fence(&"ranked evidence ".repeat(4_000))),
        ];
        for turn in 0..20 {
            messages.push(ChatMessage::user(format!(
                "old question {turn} {}",
                "detail ".repeat(100)
            )));
            messages.push(ChatMessage::assistant(format!(
                "old answer {turn} {}",
                "detail ".repeat(100)
            )));
        }
        messages.push(ChatMessage::user("current question"));

        let fitted = fit_messages_to_context_window(messages, 4_096, 1_024);

        assert!(estimated_request_tokens(&fitted) <= (4_096 - 2_048) as usize);
        assert_eq!(fitted.last().unwrap().content, "current question");
        assert_eq!(
            fitted
                .iter()
                .filter(|message| message.role == MessageRole::System)
                .nth(1)
                .unwrap()
                .content
                .matches("<<<UNTRUSTED_CONTENT>>>")
                .count(),
            1
        );
        assert_eq!(
            fitted
                .iter()
                .filter(|message| message.role == MessageRole::System)
                .nth(1)
                .unwrap()
                .content
                .matches("<<<END_UNTRUSTED_CONTENT>>>")
                .count(),
            1
        );
    }

    #[test]
    fn prompt_assembly_preserves_history_when_the_complete_request_fits() {
        let messages = vec![
            ChatMessage::system("system"),
            ChatMessage::system(fence("evidence")),
            ChatMessage::user("history ".repeat(1_500)),
            ChatMessage::assistant("answer ".repeat(100)),
            ChatMessage::user("current question"),
        ];

        let fitted = fit_messages_to_context_window(messages.clone(), 8_192, 1_024);

        assert_eq!(fitted, messages);
    }

    #[test]
    fn question_validation_uses_the_configured_window_and_non_ascii_estimate() {
        assert!(validate_question_for_context(&"a".repeat(4_096), 4_096, 1_024).is_ok());
        let error = validate_question_for_context(&"界".repeat(1_025), 4_096, 1_024).unwrap_err();
        assert!(matches!(
            error,
            AppError::Domain(DomainError::Validation { ref field, .. }) if field == "question"
        ));
    }
}
