use ind_application::AppError;
use ind_application::event_intents::ai_output_failed;
use ind_application::repos::event::MutationSideEffects;
use ind_domain::{AiPromptAction, AiRunId};

use crate::content::{chat_provider_from_config, map_ai_error};
use crate::{ChatCompletionRequest, ChatMessage, ReasoningEffort};

use super::model::{ModelResult, first_choice_content, usage_from_response};
use super::runner::{ActionTarget, AiActionRunner, PreparedAction};

impl AiActionRunner {
    pub(super) async fn run_model_completion(
        &self,
        prepared: &PreparedAction,
        action: AiPromptAction,
    ) -> Result<ModelResult, AppError> {
        let provider =
            chat_provider_from_config(&prepared.config, self.credential_cipher.as_deref())?;
        let system_prompt = action_system_prompt(prepared, action);
        let mut request = action_request(prepared, action, &system_prompt);
        let response = self
            .ai_client
            .chat_completion(&provider, request.clone())
            .await
            .map_err(map_ai_error)?;
        let first_usage = response_usage(&response, &system_prompt, &prepared.user_prompt);
        let first_was_truncated = response
            .choices
            .first()
            .and_then(|choice| choice.finish_reason.as_deref())
            == Some("length");

        let (response, input_tokens, output_tokens) = if first_was_truncated {
            let mut retry_prompt = system_prompt.clone();
            retry_prompt.push_str("\n\n");
            retry_prompt.push_str(compact_retry_instruction(action));
            request.messages = vec![
                ChatMessage::system(retry_prompt.as_str()),
                ChatMessage::user(prepared.user_prompt.clone()),
            ];
            request.max_completion_tokens = Some(compact_retry_budget(action));
            let response = self
                .ai_client
                .chat_completion(&provider, request)
                .await
                .map_err(map_ai_error)?;
            let retry_usage = response_usage(&response, &retry_prompt, &prepared.user_prompt);
            (
                response,
                sum_optional(first_usage.0, retry_usage.0),
                sum_optional(first_usage.1, retry_usage.1),
            )
        } else {
            (response, first_usage.0, first_usage.1)
        };

        let raw_content = first_choice_content(&response)?;
        Ok(ModelResult {
            raw_content,
            input_tokens,
            output_tokens,
        })
    }

    pub(super) async fn mark_run_failed(
        &self,
        run_id: AiRunId,
        prepared: &PreparedAction,
        target: ActionTarget,
        action: AiPromptAction,
        err: &AppError,
    ) -> Result<(), AppError> {
        let message = err.to_string();
        self.store
            .fail_run(
                run_id,
                message.clone(),
                MutationSideEffects::with_event(ai_output_failed(
                    prepared.user_id,
                    target.run_document_id(),
                    action,
                    run_id,
                    &message,
                )),
            )
            .await
    }

    pub(super) async fn finish_run(
        &self,
        run_id: AiRunId,
        prepared: &PreparedAction,
        target: ActionTarget,
        action: AiPromptAction,
        model: &ModelResult,
        result: Result<(), AppError>,
    ) -> Result<(), AppError> {
        match result {
            Ok(()) => {
                self.store
                    .complete_run(run_id, model.input_tokens, model.output_tokens)
                    .await
            }
            Err(err) => {
                self.mark_run_failed(run_id, prepared, target, action, &err)
                    .await?;
                Err(err)
            }
        }
    }
}

fn action_system_prompt(prepared: &PreparedAction, action: AiPromptAction) -> String {
    let mut prompt = prepared.system_prompt.clone();
    if action == AiPromptAction::Summary {
        prompt.push_str("\n\n");
        prompt.push_str(&format!(
            "Keep the summary to at most {} words.",
            super::budget::SUMMARY_WORD_LIMIT
        ));
    }
    if let Some(instruction) = structured_output_instruction(action) {
        prompt.push_str("\n\n");
        prompt.push_str(instruction);
    }
    prompt
}

fn action_request(
    prepared: &PreparedAction,
    action: AiPromptAction,
    system_prompt: &str,
) -> ChatCompletionRequest {
    let mut request = ChatCompletionRequest::new(
        prepared.config.chat_model.clone(),
        vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(prepared.user_prompt.clone()),
        ],
    );
    request.temperature = Some(match action {
        AiPromptAction::Summary => 0.2,
        AiPromptAction::Tags | AiPromptAction::Entities => 0.1,
        AiPromptAction::Chat | AiPromptAction::Custom => 0.7,
    });
    request.max_completion_tokens = Some(super::budget::output_budget_tokens(action) as u32);
    request.user = Some(prepared.user_id.to_string());
    if prepared.config.supports_structured_output {
        request.response_format = super::schema::response_format_for(action);
    }
    if prepared.config.supports_reasoning_effort
        && matches!(
            action,
            AiPromptAction::Summary | AiPromptAction::Tags | AiPromptAction::Entities
        )
    {
        request.reasoning_effort = Some(ReasoningEffort::None);
    }
    request
}

fn response_usage(
    response: &crate::ChatCompletionResponse,
    system_prompt: &str,
    user_prompt: &str,
) -> (Option<i32>, Option<i32>) {
    let content = response
        .choices
        .first()
        .map(|choice| choice.message.content.as_str())
        .unwrap_or_default();
    usage_from_response(response, system_prompt, user_prompt, content)
}

fn sum_optional(first: Option<i32>, second: Option<i32>) -> Option<i32> {
    match (first, second) {
        (Some(first), Some(second)) => Some(first.saturating_add(second)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn compact_retry_budget(action: AiPromptAction) -> u32 {
    match action {
        AiPromptAction::Summary | AiPromptAction::Tags => 512,
        AiPromptAction::Entities => 1024,
        AiPromptAction::Chat | AiPromptAction::Custom => 512,
    }
}

fn compact_retry_instruction(action: AiPromptAction) -> &'static str {
    match action {
        AiPromptAction::Summary => {
            "Your previous response exceeded the output limit. Return only the JSON object with a summary of at most 80 words."
        }
        AiPromptAction::Tags => {
            "Your previous response exceeded the output limit. Return only the JSON object with 3 to 5 concise tags."
        }
        AiPromptAction::Entities => {
            "Your previous response exceeded the output limit. Return only the JSON object with at most 8 essential entities; use null descriptions and empty aliases when needed."
        }
        AiPromptAction::Chat | AiPromptAction::Custom => {
            "Your previous response exceeded the output limit. Answer as concisely as possible."
        }
    }
}

fn structured_output_instruction(action: AiPromptAction) -> Option<&'static str> {
    match action {
        AiPromptAction::Summary => Some(
            "Return the summary as a JSON object: {\"summary\": \"...\"}. Output only that JSON object.",
        ),
        AiPromptAction::Tags => Some(
            "Return tags as a JSON object: {\"tags\": [\"tag1\", \"tag2\"]}. Output only that JSON object.",
        ),
        AiPromptAction::Entities => Some(
            "Return entities as a JSON object: {\"entities\": [{\"name\": \"...\", \"entity_type\": \"person|organization|location|event|work\", \"description\": null, \"mention_count\": 1, \"aliases\": [\"...\"]}]}. aliases lists the entity's well-known alternate names (synonyms or acronyms, e.g. for Meta include \"Facebook\"), or [] if none. Include at most 16 of the most important entities. Output only that JSON object.",
        ),
        AiPromptAction::Chat | AiPromptAction::Custom => None,
    }
}
