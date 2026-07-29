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
        chat_request.temperature = Some(0.0);

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

fn provider_test_message(error: AiError) -> String {
    match error {
        AiError::ProviderUnreachable { .. } => "Could not reach the provider. Check the provider URL and that the server is running.".into(),
        AiError::AuthenticationFailed { message } | AiError::ProviderError { message, .. } => message,
        AiError::MalformedResponse { .. } => "The provider returned an invalid response.".into(),
        AiError::StreamTerminatedUnexpectedly { .. } => "The provider connection ended unexpectedly.".into(),
        AiError::EndpointDisallowed { .. } => "This endpoint is not allowed: it must be an https URL that does not resolve to a private or internal address.".into(),
    }
}
