use ind_application::AppError;

use crate::ChatCompletionResponse;
use crate::chunker::approximate_token_count;

pub(super) struct ModelResult {
    pub(super) raw_content: String,
    pub(super) input_tokens: Option<i32>,
    pub(super) output_tokens: Option<i32>,
}

pub(crate) fn first_choice_content(response: &ChatCompletionResponse) -> Result<String, AppError> {
    let choice = response
        .choices
        .first()
        .ok_or_else(|| AppError::ExternalService {
            service: "mila-provider".into(),
            message: "chat completion returned no usable content".into(),
        })?;

    if choice.finish_reason.as_deref() == Some("length") {
        return Err(AppError::ExternalService {
            service: "mila-provider".into(),
            message: "model response truncated before completion (finish_reason=length)".into(),
        });
    }

    let content = choice.message.content.trim().to_string();
    if content.is_empty() {
        return Err(AppError::ExternalService {
            service: "mila-provider".into(),
            message: "chat completion returned no usable content".into(),
        });
    }

    Ok(content)
}

pub(super) fn usage_from_response(
    response: &ChatCompletionResponse,
    system_prompt: &str,
    user_prompt: &str,
    raw_content: &str,
) -> (Option<i32>, Option<i32>) {
    if let Some(usage) = response.usage.as_ref() {
        let input_tokens = i32::try_from(usage.prompt_tokens).ok();
        let output_tokens = usage
            .completion_tokens
            .and_then(|value| i32::try_from(value).ok())
            .or_else(|| i32::try_from(usage.total_tokens.saturating_sub(usage.prompt_tokens)).ok());
        return (input_tokens, output_tokens);
    }

    let prompt_tokens =
        approximate_token_count(system_prompt).saturating_add(approximate_token_count(user_prompt));
    let completion_tokens = approximate_token_count(raw_content);

    (
        i32::try_from(prompt_tokens).ok(),
        i32::try_from(completion_tokens).ok(),
    )
}
