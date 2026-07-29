pub use ind_application::ports::{OnboardingStatus, OnboardingStepInfo};
use ind_domain::{User, UserId};

use crate::error::AuthError;
use crate::service::AuthService;

pub const TOTAL_ONBOARDING_STEPS: i16 = 5;

pub const STEP_NAMES: &[(i16, &str)] = &[
    (1, "Account Setup"),
    (2, "Add Content"),
    (3, "RSS Feeds"),
    (4, "AI Configuration"),
    (5, "Complete"),
];

fn build_onboarding_status(user: &User) -> OnboardingStatus {
    let steps = STEP_NAMES
        .iter()
        .map(|&(step, name)| OnboardingStepInfo {
            step,
            name: name.to_string(),
            completed: step <= user.onboarding_step,
        })
        .collect();

    OnboardingStatus {
        current_step: user.onboarding_step,
        completed: user.onboarding_completed,
        steps,
    }
}

impl AuthService {
    pub async fn get_onboarding_status(
        &self,
        user_id: UserId,
    ) -> Result<OnboardingStatus, AuthError> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::AccountNotFound)?;

        Ok(build_onboarding_status(&user))
    }

    pub async fn advance_onboarding(
        &self,
        user_id: UserId,
        step: i16,
    ) -> Result<OnboardingStatus, AuthError> {
        if !(1..=TOTAL_ONBOARDING_STEPS).contains(&step) {
            return Err(AuthError::ValidationError {
                field: "step".to_string(),
                message: format!("must be between 1 and {}", TOTAL_ONBOARDING_STEPS),
            });
        }

        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::AccountNotFound)?;

        if user.onboarding_completed || step <= user.onboarding_step {
            return Ok(build_onboarding_status(&user));
        }

        let completed = step == TOTAL_ONBOARDING_STEPS;
        let user = self
            .user_repo
            .update_onboarding(user_id, step, completed)
            .await?;

        Ok(build_onboarding_status(&user))
    }

    pub async fn complete_onboarding(
        &self,
        user_id: UserId,
    ) -> Result<OnboardingStatus, AuthError> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::AccountNotFound)?;

        if user.onboarding_completed {
            return Ok(build_onboarding_status(&user));
        }

        let user = self
            .user_repo
            .update_onboarding(user_id, TOTAL_ONBOARDING_STEPS, true)
            .await?;

        Ok(build_onboarding_status(&user))
    }
}
