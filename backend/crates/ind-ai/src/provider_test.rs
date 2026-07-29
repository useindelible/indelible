use std::future::Future;
use std::sync::Arc;

use ind_application::AppError;
use ind_application::ports::{MilaProviderTestResult, TestMilaConfigRequest};
use ind_auth::CredentialCipher;
use ind_domain::{MilaConfig, UserId};
use secrecy::SecretString;

use crate::{
    AiError, AiProviderClient, AiProviderConfig, ChatCompletionRequest, ChatMessage,
    EmbeddingRequest, chat_provider_from_config, embedding_provider_from_config,
};

#[derive(Clone)]
pub struct MilaProviderTestService {
    ai_client: Arc<dyn AiProviderClient>,
    credential_cipher: Option<Arc<CredentialCipher>>,
}

impl MilaProviderTestService {
    pub fn new(
        ai_client: Arc<dyn AiProviderClient>,
        credential_cipher: Option<Arc<CredentialCipher>>,
    ) -> Self {
        Self {
            ai_client,
            credential_cipher,
        }
    }

    pub async fn test_config<LoadConfig, LoadConfigFuture>(
        &self,
        user_id: UserId,
        request: TestMilaConfigRequest,
        load_config: LoadConfig,
    ) -> Result<MilaProviderTestResult, AppError>
    where
        LoadConfig: FnOnce(UserId) -> LoadConfigFuture,
        LoadConfigFuture: Future<Output = Result<Option<MilaConfig>, AppError>>,
    {
        let saved = if request.chat_api_key.is_none() || request.embedding_api_key.is_none() {
            Some(load_config(user_id).await?.ok_or_else(|| {
                AppError::Domain(ind_domain::DomainError::InvariantViolation {
                    message: format!("missing effective Mila config for user {user_id}"),
                })
            })?)
        } else {
            None
        };
        self.test_with_saved_config(request, saved.as_ref()).await
    }

    async fn test_with_saved_config(
        &self,
        request: TestMilaConfigRequest,
        saved: Option<&MilaConfig>,
    ) -> Result<MilaProviderTestResult, AppError> {
        validate_saved_key_origins(&request, saved)?;
        let chat_key = match request.chat_api_key.clone() {
            Some(key) => Some(SecretString::from(key)),
            None => saved
                .map(|config| chat_provider_from_config(config, self.credential_cipher.as_deref()))
                .transpose()?
                .and_then(|provider| provider.api_key),
        };
        let embedding_key = match request.embedding_api_key.clone() {
            Some(key) => Some(SecretString::from(key)),
            None => saved
                .map(|config| {
                    embedding_provider_from_config(config, self.credential_cipher.as_deref())
                })
                .transpose()?
                .and_then(|provider| provider.api_key),
        };
        let chat_provider = AiProviderConfig::new(request.chat_api_base.clone(), chat_key);
        let embedding_provider =
            AiProviderConfig::new(request.embedding_api_base.clone(), embedding_key);
        let embedding_request = EmbeddingRequest::new(
            request.embedding_model.clone(),
            "Indelible embedding dimension probe",
        )
        .with_dimensions(request.embedding_dim);
        let mut chat_request = ChatCompletionRequest::new(
            request.chat_model.clone(),
            vec![
                ChatMessage::system("You are a connectivity probe."),
                ChatMessage::user("Reply with OK."),
            ],
        );
        chat_request.max_completion_tokens = Some(8);
        if !request.supports_reasoning_effort {
            chat_request.temperature = Some(0.0);
        }

        let (embedding_result, chat_result) = futures::join!(
            self.ai_client
                .embedding(&embedding_provider, embedding_request),
            self.ai_client.chat_completion(&chat_provider, chat_request),
        );
        let (embedding_model_ok, embedding_dim, embedding_error) = match embedding_result {
            Ok(response) => {
                let dimension = i32::try_from(response.embedding.len()).map_err(|_| {
                    AppError::ExternalService {
                        service: "mila-provider".into(),
                        message: "embedding dimension overflow".into(),
                    }
                })?;
                if dimension == request.embedding_dim {
                    (true, Some(dimension), None)
                } else {
                    (
                        false,
                        Some(dimension),
                        Some(format!(
                            "Embedding model returned {dimension} dimensions; expected {}.",
                            request.embedding_dim
                        )),
                    )
                }
            }
            Err(error) => (false, None, Some(provider_test_message(error))),
        };
        let (chat_model_ok, chat_error) = match chat_result {
            Ok(_) => (true, None),
            Err(error) => (false, Some(provider_test_message(error))),
        };
        let error = embedding_error.clone().or_else(|| chat_error.clone());
        Ok(MilaProviderTestResult {
            success: embedding_model_ok && chat_model_ok,
            embedding_dim,
            chat_model_ok,
            embedding_model_ok,
            chat_error,
            embedding_error,
            error,
        })
    }
}

fn validate_saved_key_origins(
    request: &TestMilaConfigRequest,
    saved: Option<&MilaConfig>,
) -> Result<(), AppError> {
    let Some(saved) = saved else {
        return Ok(());
    };

    require_key_for_changed_base(
        "chat_api_key",
        &request.chat_api_base,
        request.chat_api_key.is_some(),
        &saved.chat_api_base,
        saved.chat_api_key_enc.as_deref(),
    )?;
    require_key_for_changed_base(
        "embedding_api_key",
        &request.embedding_api_base,
        request.embedding_api_key.is_some(),
        &saved.embedding_api_base,
        saved.embedding_api_key_enc.as_deref(),
    )
}

fn require_key_for_changed_base(
    field: &'static str,
    request_base: &str,
    has_request_key: bool,
    saved_base: &str,
    saved_key: Option<&[u8]>,
) -> Result<(), AppError> {
    if !has_request_key
        && saved_key.is_some_and(|key| !key.is_empty())
        && request_base != saved_base
    {
        return Err(AppError::Domain(ind_domain::DomainError::Validation {
            field: field.into(),
            message: "a request-supplied key is required when the provider base differs from the saved configuration".into(),
        }));
    }
    Ok(())
}

fn provider_test_message(error: AiError) -> String {
    match error {
        AiError::ProviderUnreachable { .. } => "Could not reach the provider. Check the provider URL and that the server is running.".into(),
        AiError::AuthenticationFailed { message } | AiError::ProviderError { message, .. } => message,
        AiError::MalformedResponse { .. } => "The provider returned an invalid response.".into(),
        AiError::StreamTerminatedUnexpectedly { .. } => "The provider connection ended unexpectedly.".into(),
        AiError::EndpointDisallowed { .. } => "This endpoint is not allowed. Public providers must use https; a local server must be a localhost or host.docker.internal address, with EGRESS_ALLOW_PRIVATE_TARGETS=true set on the server.".into(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use secrecy::ExposeSecret;

    use super::*;
    use crate::{ChatCompletionResponse, ChatCompletionStream, EmbeddingResponse};

    #[derive(Default)]
    struct RecordingProvider {
        chat_request: Mutex<Option<ChatCompletionRequest>>,
        chat_provider: Mutex<Option<AiProviderConfig>>,
        embedding_provider: Mutex<Option<AiProviderConfig>>,
    }

    #[async_trait]
    impl AiProviderClient for RecordingProvider {
        async fn chat_completion(
            &self,
            provider: &AiProviderConfig,
            request: ChatCompletionRequest,
        ) -> Result<ChatCompletionResponse, AiError> {
            *self.chat_provider.lock().unwrap() = Some(provider.clone());
            *self.chat_request.lock().unwrap() = Some(request);
            Ok(ChatCompletionResponse {
                id: "probe".into(),
                model: "reasoning-model".into(),
                choices: Vec::new(),
                usage: None,
            })
        }

        async fn chat_completion_stream(
            &self,
            _: &AiProviderConfig,
            _: ChatCompletionRequest,
        ) -> Result<ChatCompletionStream, AiError> {
            unreachable!()
        }

        async fn embedding(
            &self,
            provider: &AiProviderConfig,
            request: EmbeddingRequest,
        ) -> Result<EmbeddingResponse, AiError> {
            *self.embedding_provider.lock().unwrap() = Some(provider.clone());
            Ok(EmbeddingResponse {
                model: request.model,
                embedding: vec![0.0; 4],
                usage: None,
            })
        }
    }

    fn credential_cipher() -> Arc<CredentialCipher> {
        Arc::new(
            CredentialCipher::from_base64("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").unwrap(),
        )
    }

    fn saved_config(user_id: UserId, cipher: &CredentialCipher) -> MilaConfig {
        let now = chrono::Utc::now();
        MilaConfig {
            user_id,
            chat_api_base: "https://saved-chat.example/v1".into(),
            chat_api_key_enc: Some(cipher.seal(b"saved-chat-secret")),
            chat_model: "saved-chat-model".into(),
            embedding_api_base: "https://saved-embedding.example/v1".into(),
            embedding_api_key_enc: Some(cipher.seal(b"saved-embedding-secret")),
            embedding_model: "saved-embedding-model".into(),
            embedding_dim: 4,
            model_context_window: 8_192,
            chat_context_pct: 70,
            chunk_size: 1_000,
            chunk_overlap: 100,
            top_k: 6,
            cross_item_top_k: 20,
            cross_item_max_per_item: 3,
            enabled: true,
            byo_enabled: true,
            supports_structured_output: true,
            supports_reasoning_effort: false,
            chat_cipher_version: CredentialCipher::version(),
            embedding_cipher_version: CredentialCipher::version(),
            created_at: now,
            updated_at: now,
        }
    }

    fn test_request() -> TestMilaConfigRequest {
        TestMilaConfigRequest {
            chat_api_base: "https://saved-chat.example/v1".into(),
            chat_api_key: None,
            chat_model: "chat-model".into(),
            supports_reasoning_effort: false,
            embedding_api_base: "https://saved-embedding.example/v1".into(),
            embedding_api_key: None,
            embedding_model: "embedding-model".into(),
            embedding_dim: 4,
        }
    }

    fn assert_key_origin_validation(error: AppError, field: &str) {
        let AppError::Domain(ind_domain::DomainError::Validation {
            field: actual_field,
            message,
        }) = &error
        else {
            panic!("expected validation error, got {error:?}");
        };
        assert_eq!(actual_field, field);
        assert!(message.contains("request-supplied key"));
        let diagnostic = format!("{error:?} {error}");
        assert!(!diagnostic.contains("saved-chat-secret"));
        assert!(!diagnostic.contains("saved-embedding-secret"));
    }

    #[tokio::test]
    async fn omitted_saved_chat_key_is_rejected_for_a_different_chat_base_before_requests() {
        let provider = Arc::new(RecordingProvider::default());
        let cipher = credential_cipher();
        let service = MilaProviderTestService::new(provider.clone(), Some(cipher.clone()));
        let user_id = UserId::new();
        let saved = saved_config(user_id, &cipher);
        let mut request = test_request();
        request.chat_api_base = "https://custom-chat.example/v1".into();
        request.embedding_api_key = Some("request-embedding-secret".into());

        let error = service
            .test_config(user_id, request, |_| async { Ok(Some(saved)) })
            .await
            .expect_err("a saved chat key must stay bound to its provider base");

        assert_key_origin_validation(error, "chat_api_key");
        assert!(provider.chat_provider.lock().unwrap().is_none());
        assert!(provider.embedding_provider.lock().unwrap().is_none());
    }

    #[tokio::test]
    async fn omitted_saved_embedding_key_is_rejected_for_a_different_embedding_base_before_requests()
     {
        let provider = Arc::new(RecordingProvider::default());
        let cipher = credential_cipher();
        let service = MilaProviderTestService::new(provider.clone(), Some(cipher.clone()));
        let user_id = UserId::new();
        let saved = saved_config(user_id, &cipher);
        let mut request = test_request();
        request.chat_api_key = Some("request-chat-secret".into());
        request.embedding_api_base = "https://custom-embedding.example/v1".into();

        let error = service
            .test_config(user_id, request, |_| async { Ok(Some(saved)) })
            .await
            .expect_err("a saved embedding key must stay bound to its provider base");

        assert_key_origin_validation(error, "embedding_api_key");
        assert!(provider.chat_provider.lock().unwrap().is_none());
        assert!(provider.embedding_provider.lock().unwrap().is_none());
    }

    #[tokio::test]
    async fn matching_provider_bases_reuse_each_saved_key() {
        let provider = Arc::new(RecordingProvider::default());
        let cipher = credential_cipher();
        let service = MilaProviderTestService::new(provider.clone(), Some(cipher.clone()));
        let user_id = UserId::new();
        let saved = saved_config(user_id, &cipher);

        service
            .test_config(user_id, test_request(), |_| async { Ok(Some(saved)) })
            .await
            .unwrap();

        let chat = provider.chat_provider.lock().unwrap().clone().unwrap();
        assert_eq!(chat.api_base, "https://saved-chat.example/v1");
        assert_eq!(chat.api_key.unwrap().expose_secret(), "saved-chat-secret");
        let embedding = provider.embedding_provider.lock().unwrap().clone().unwrap();
        assert_eq!(embedding.api_base, "https://saved-embedding.example/v1");
        assert_eq!(
            embedding.api_key.unwrap().expose_secret(),
            "saved-embedding-secret"
        );
    }

    #[tokio::test]
    async fn explicit_keys_are_used_with_custom_provider_bases() {
        let provider = Arc::new(RecordingProvider::default());
        let service = MilaProviderTestService::new(provider.clone(), None);
        let mut request = test_request();
        request.chat_api_base = "https://custom-chat.example/v1".into();
        request.chat_api_key = Some("request-chat-secret".into());
        request.embedding_api_base = "https://custom-embedding.example/v1".into();
        request.embedding_api_key = Some("request-embedding-secret".into());

        service
            .test_config(UserId::new(), request, |_| async { unreachable!() })
            .await
            .unwrap();

        let chat = provider.chat_provider.lock().unwrap().clone().unwrap();
        assert_eq!(chat.api_base, "https://custom-chat.example/v1");
        assert_eq!(chat.api_key.unwrap().expose_secret(), "request-chat-secret");
        let embedding = provider.embedding_provider.lock().unwrap().clone().unwrap();
        assert_eq!(embedding.api_base, "https://custom-embedding.example/v1");
        assert_eq!(
            embedding.api_key.unwrap().expose_secret(),
            "request-embedding-secret"
        );
    }

    #[tokio::test]
    async fn custom_bases_without_saved_keys_can_be_tested_without_request_keys() {
        let provider = Arc::new(RecordingProvider::default());
        let cipher = credential_cipher();
        let service = MilaProviderTestService::new(provider.clone(), None);
        let user_id = UserId::new();
        let mut saved = saved_config(user_id, &cipher);
        saved.chat_api_key_enc = None;
        saved.embedding_api_key_enc = Some(Vec::new());
        let mut request = test_request();
        request.chat_api_base = "http://localhost:11434/v1".into();
        request.embedding_api_base = "http://localhost:11435/v1".into();

        service
            .test_config(user_id, request, |_| async { Ok(Some(saved)) })
            .await
            .unwrap();

        let chat = provider.chat_provider.lock().unwrap().clone().unwrap();
        assert_eq!(chat.api_base, "http://localhost:11434/v1");
        assert!(chat.api_key.is_none());
        let embedding = provider.embedding_provider.lock().unwrap().clone().unwrap();
        assert_eq!(embedding.api_base, "http://localhost:11435/v1");
        assert!(embedding.api_key.is_none());
    }

    #[tokio::test]
    async fn reasoning_capable_probe_omits_sampling_parameters() {
        let provider = Arc::new(RecordingProvider::default());
        let service = MilaProviderTestService::new(provider.clone(), None);

        service
            .test_config(
                UserId::new(),
                TestMilaConfigRequest {
                    chat_api_base: "https://api.openai.com/v1".into(),
                    chat_api_key: Some("chat-key".into()),
                    chat_model: "reasoning-model".into(),
                    supports_reasoning_effort: true,
                    embedding_api_base: "https://api.openai.com/v1".into(),
                    embedding_api_key: Some("embedding-key".into()),
                    embedding_model: "embedding-model".into(),
                    embedding_dim: 4,
                },
                |_| async { unreachable!() },
            )
            .await
            .unwrap();

        let request = provider.chat_request.lock().unwrap().clone().unwrap();
        let json = serde_json::to_value(request).unwrap();
        assert!(json.get("temperature").is_none());
        assert!(json.get("top_p").is_none());
        assert!(json.get("reasoning_effort").is_none());
    }

    #[tokio::test]
    async fn sampling_probe_keeps_temperature() {
        let provider = Arc::new(RecordingProvider::default());
        let service = MilaProviderTestService::new(provider.clone(), None);

        service
            .test_config(
                UserId::new(),
                TestMilaConfigRequest {
                    chat_api_base: "https://api.openai.com/v1".into(),
                    chat_api_key: Some("chat-key".into()),
                    chat_model: "sampling-model".into(),
                    supports_reasoning_effort: false,
                    embedding_api_base: "https://api.openai.com/v1".into(),
                    embedding_api_key: Some("embedding-key".into()),
                    embedding_model: "embedding-model".into(),
                    embedding_dim: 4,
                },
                |_| async { unreachable!() },
            )
            .await
            .unwrap();

        let request = provider.chat_request.lock().unwrap().clone().unwrap();
        let json = serde_json::to_value(request).unwrap();
        assert_eq!(json.get("temperature"), Some(&serde_json::json!(0.0)));
        assert!(json.get("reasoning_effort").is_none());
    }
}
