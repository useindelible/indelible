use axum::Router;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use ind_application::AppError;
use ind_application::ports::{OnboardingStatus, OnboardingStepInfo, UpdateMilaConfigRequest};
use ind_domain::DomainError;
use ind_domain::ai::MILA_EMBEDDING_DIM;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{ApiError, FieldError};
use crate::extract::Json;
use crate::middleware::{Principal, RequireVerifiedUserAccessJwt};
use crate::response::ApiResponse;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct OnboardingResponse {
    pub current_step: i16,
    pub completed: bool,
    pub steps: Vec<OnboardingStepResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OnboardingStepResponse {
    pub step: i16,
    pub name: String,
    pub completed: bool,
}

impl From<OnboardingStepInfo> for OnboardingStepResponse {
    fn from(s: OnboardingStepInfo) -> Self {
        Self {
            step: s.step,
            name: s.name,
            completed: s.completed,
        }
    }
}

impl From<OnboardingStatus> for OnboardingResponse {
    fn from(status: OnboardingStatus) -> Self {
        Self {
            current_step: status.current_step,
            completed: status.completed,
            steps: status
                .steps
                .into_iter()
                .map(OnboardingStepResponse::from)
                .collect(),
        }
    }
}

/// Freeform step-completion payload. All fields are optional; each step
/// uses only the fields relevant to it.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct StepData {
    pub display_name: Option<String>,
    pub theme: Option<String>,
    pub source: Option<String>,
    pub feed_urls: Option<Vec<String>>,
    pub chat_provider: Option<String>,
    pub chat_api_key: Option<String>,
    pub chat_endpoint: Option<String>,
    pub embedding_provider: Option<String>,
    pub embedding_api_key: Option<String>,
    pub embedding_endpoint: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompleteStepRequest {
    #[schema(value_type = StepData)]
    pub data: serde_json::Value,
}

#[utoipa::path(
    get,
    path = "/api/v1/onboarding",
    responses(
        (status = 200, description = "Onboarding status", body = OnboardingResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Verified user access JWT from a supported client and verified email required"),
    ),
    security(("bearer" = [])),
    tag = "Onboarding",
)]
pub async fn get_onboarding(
    RequireVerifiedUserAccessJwt(Principal { user_id, .. }): RequireVerifiedUserAccessJwt,
    State(state): State<AppState>,
) -> Result<ApiResponse<OnboardingResponse>, ApiError> {
    let status = state
        .onboarding_ops
        .get_onboarding(user_id)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(OnboardingResponse::from(status)))
}

#[utoipa::path(
    post,
    path = "/api/v1/onboarding/steps/{step}/complete",
    params(
        ("step" = i16, Path, description = "Step number to complete"),
    ),
    request_body = CompleteStepRequest,
    responses(
        (status = 200, description = "Step completed, updated onboarding status", body = OnboardingResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Verified user access JWT from a supported client and verified email required"),
        (status = 422, description = "Invalid step or feed URL"),
    ),
    security(("bearer" = [])),
    tag = "Onboarding",
)]
pub async fn complete_step(
    RequireVerifiedUserAccessJwt(Principal { user_id, .. }): RequireVerifiedUserAccessJwt,
    State(state): State<AppState>,
    Path(step): Path<i16>,
    Json(body): Json<CompleteStepRequest>,
) -> Result<ApiResponse<OnboardingResponse>, ApiError> {
    let mila_request = if step == 4 {
        mila_config_from_ai_step_data(&body.data)
    } else {
        None
    };

    if step == 3 {
        subscribe_feed_step(&state, user_id, &body.data).await?;
    }

    if let Some(request) = mila_request {
        let mila_ops = state
            .mila_config_ops
            .as_deref()
            .ok_or(ApiError::ServiceUnavailable {
                message: "mila service not configured".into(),
            })?;
        mila_ops
            .upsert_config(user_id, request)
            .await
            .map_err(ApiError::from)?;
    }

    let status = state
        .onboarding_ops
        .complete_step(user_id, step, body.data)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(OnboardingResponse::from(status)))
}

#[utoipa::path(
    post,
    path = "/api/v1/onboarding/skip",
    responses(
        (status = 200, description = "Onboarding skipped", body = OnboardingResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Verified user access JWT from a supported client and verified email required"),
    ),
    security(("bearer" = [])),
    tag = "Onboarding",
)]
pub async fn skip_onboarding(
    RequireVerifiedUserAccessJwt(Principal { user_id, .. }): RequireVerifiedUserAccessJwt,
    State(state): State<AppState>,
) -> Result<ApiResponse<OnboardingResponse>, ApiError> {
    let status = state
        .onboarding_ops
        .skip_onboarding(user_id)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(OnboardingResponse::from(status)))
}

async fn subscribe_feed_step(
    state: &AppState,
    user_id: ind_domain::UserId,
    data: &serde_json::Value,
) -> Result<(), ApiError> {
    let urls = feed_urls_from_step_data(data)?;
    if urls.is_empty() {
        return Ok(());
    }

    let feed_ops = state
        .feed_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "feed service not configured".into(),
        })?;

    for url in urls {
        match feed_ops.subscribe(user_id, url.clone(), None, None).await {
            Ok(_) => {}
            Err(err) if is_already_subscribed(&err) => {}
            Err(err) => return Err(feed_subscription_error(&url, err)),
        }
    }

    Ok(())
}

fn feed_urls_from_step_data(data: &serde_json::Value) -> Result<Vec<String>, ApiError> {
    let step_data: StepData =
        serde_json::from_value(data.clone()).map_err(|e| ApiError::ValidationError {
            errors: vec![FieldError {
                field: "data".into(),
                message: format!("invalid onboarding step data: {e}"),
            }],
        })?;

    let mut seen = std::collections::BTreeSet::new();
    let urls = step_data
        .feed_urls
        .unwrap_or_default()
        .into_iter()
        .filter_map(|url| {
            let url = url.trim();
            if url.is_empty() || !seen.insert(url.to_string()) {
                None
            } else {
                Some(url.to_string())
            }
        })
        .collect();

    Ok(urls)
}

fn is_already_subscribed(err: &AppError) -> bool {
    matches!(
        err,
        AppError::Domain(DomainError::Conflict {
            entity: "FeedSubscription",
            message,
        }) if message.contains("already subscribed")
    )
}

fn feed_subscription_error(url: &str, err: AppError) -> ApiError {
    match err {
        AppError::Domain(DomainError::Validation { field, message }) => ApiError::ValidationError {
            errors: vec![FieldError {
                field: format!("data.feed_urls.{field}"),
                message,
            }],
        },
        AppError::ExternalService { message, .. } => ApiError::ValidationError {
            errors: vec![FieldError {
                field: "data.feed_urls".into(),
                message: format!("could not subscribe to {url}: {message}"),
            }],
        },
        other => ApiError::from(other),
    }
}

fn mila_config_from_ai_step_data(data: &serde_json::Value) -> Option<UpdateMilaConfigRequest> {
    let chat_provider = data.get("chat_provider")?.as_str()?;
    let embedding_provider = data.get("embedding_provider")?.as_str()?;
    let chat_api_key = optional_text(data, "chat_api_key");
    let embedding_api_key = optional_text(data, "embedding_api_key");

    // model_context_window seeds the known context window of the chat model this step provisions,
    // so the action input budget is right-sized from the start. Users adjust it in settings later.
    let (chat_api_base, chat_model, model_context_window, supports_structured_output) =
        match chat_provider {
            "ollama" => (
                local_openai_base(data, "chat_endpoint", "http://host.docker.internal:11434"),
                optional_text(data, "chat_model")?,
                16_000_i32,
                false,
            ),
            "openai" => (
                "https://api.openai.com/v1".into(),
                "gpt-5.4-mini".into(),
                400_000_i32,
                true,
            ),
            _ => return None,
        };

    let (embedding_api_base, embedding_model, embedding_dim) = match embedding_provider {
        "ollama" => (
            local_openai_base(
                data,
                "embedding_endpoint",
                "http://host.docker.internal:11434",
            ),
            optional_text(data, "embedding_model")?,
            768_i32,
        ),
        "openai" => (
            "https://api.openai.com/v1".into(),
            "text-embedding-3-small".into(),
            MILA_EMBEDDING_DIM,
        ),
        _ => return None,
    };

    Some(UpdateMilaConfigRequest {
        chat_api_base,
        chat_api_key,
        clear_chat_api_key: false,
        chat_model,
        embedding_api_base,
        embedding_api_key,
        clear_embedding_api_key: false,
        embedding_model,
        embedding_dim,
        model_context_window,
        chat_context_pct: 70,
        top_k: 6,
        cross_item_top_k: 20,
        cross_item_max_per_item: 3,
        enabled: true,
        byo_enabled: Some(true),
        supports_structured_output: Some(supports_structured_output),
        supports_reasoning_effort: Some(false),
    })
}

fn optional_text(data: &serde_json::Value, field: &str) -> Option<String> {
    data.get(field)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn local_openai_base(data: &serde_json::Value, field: &str, default_endpoint: &str) -> String {
    let endpoint = data
        .get(field)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(default_endpoint);
    let endpoint = endpoint.trim_end_matches('/');
    if endpoint.ends_with("/v1") {
        endpoint.to_string()
    } else {
        format!("{endpoint}/v1")
    }
}

pub fn onboarding_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/onboarding", get(get_onboarding))
        .route(
            "/api/v1/onboarding/steps/{step}/complete",
            post(complete_step),
        )
        .route("/api/v1/onboarding/skip", post(skip_onboarding))
}

#[cfg(test)]
mod tests {
    use super::mila_config_from_ai_step_data;

    /// The shipped stack runs the API inside Docker, where `localhost` is the
    /// container itself: the default local-AI endpoint must name the host
    /// gateway or the advertised quickstart cannot work.
    #[test]
    fn local_ai_defaults_to_the_docker_reachable_host_gateway() {
        let data = serde_json::json!({
            "chat_provider": "ollama",
            "chat_model": "llama3",
            "embedding_provider": "ollama",
            "embedding_model": "nomic-embed-text"
        });
        let config = mila_config_from_ai_step_data(&data).expect("config must build");
        assert_eq!(config.chat_api_base, "http://host.docker.internal:11434/v1");
        assert_eq!(
            config.embedding_api_base,
            "http://host.docker.internal:11434/v1"
        );
    }

    #[test]
    fn an_explicit_endpoint_still_wins_over_the_default() {
        let data = serde_json::json!({
            "chat_provider": "ollama",
            "chat_model": "llama3",
            "chat_endpoint": "http://192.168.1.20:11434",
            "embedding_provider": "ollama",
            "embedding_model": "nomic-embed-text",
            "embedding_endpoint": "http://192.168.1.20:11434"
        });
        let config = mila_config_from_ai_step_data(&data).expect("config must build");
        assert_eq!(config.chat_api_base, "http://192.168.1.20:11434/v1");
        assert_eq!(config.embedding_api_base, "http://192.168.1.20:11434/v1");
    }
}
