use ind_domain::TtsProvider;

use crate::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deployment {
    Hosted,
    SelfHosted,
}

pub const MANAGED_MONTHLY_CHARACTER_LIMIT: i64 = 1_000_000;

pub struct TtsEntitlements {
    deployment: Deployment,
    hosted_managed_custom_persona: bool,
}

impl TtsEntitlements {
    pub fn new(deployment: Deployment) -> Self {
        Self {
            deployment,
            hosted_managed_custom_persona: false,
        }
    }

    pub fn with_hosted_managed_custom_persona(mut self, allowed: bool) -> Self {
        self.hosted_managed_custom_persona = allowed;
        self
    }

    pub fn deployment(&self) -> Deployment {
        self.deployment
    }

    pub fn managed_monthly_character_limit(&self) -> i64 {
        MANAGED_MONTHLY_CHARACTER_LIMIT
    }

    pub fn authorize_synthesis(&self, provider: TtsProvider) -> Result<(), AppError> {
        match self.deployment {
            Deployment::SelfHosted => Ok(()),
            Deployment::Hosted if provider.hosted_native_allowed() => Ok(()),
            Deployment::Hosted => Err(AppError::PaymentRequired {
                feature: "tts_native_provider",
            }),
        }
    }

    pub fn authorize_persona_creation(&self, provider: TtsProvider) -> Result<(), AppError> {
        match self.deployment {
            Deployment::SelfHosted => Ok(()),
            Deployment::Hosted if provider == TtsProvider::Mock => Ok(()),
            Deployment::Hosted if self.hosted_managed_custom_persona => Ok(()),
            Deployment::Hosted => Err(AppError::PaymentRequired {
                feature: "tts_custom_persona",
            }),
        }
    }
}

trait HostedNativeProvider {
    fn hosted_native_allowed(self) -> bool;
}

impl HostedNativeProvider for TtsProvider {
    fn hosted_native_allowed(self) -> bool {
        matches!(
            self,
            TtsProvider::Mock | TtsProvider::DashScope | TtsProvider::UnrealSpeech
        )
    }
}
