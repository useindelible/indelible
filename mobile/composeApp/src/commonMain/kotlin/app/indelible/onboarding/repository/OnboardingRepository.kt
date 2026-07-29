package app.indelible.onboarding.repository

import app.indelible.core.model.OnboardingStatusResponse
import app.indelible.core.model.StepData

interface OnboardingRepository {
    suspend fun getOnboardingStatus(): Result<OnboardingStatusResponse>

    suspend fun completeOnboardingStep(
        step: Int,
        data: StepData = StepData(),
    ): Result<OnboardingStatusResponse>

    suspend fun skipOnboarding(): Result<OnboardingStatusResponse>
}
