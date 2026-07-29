package app.indelible.onboarding.repository

import app.indelible.core.model.OnboardingStatusResponse
import app.indelible.core.model.StepData
import app.indelible.core.network.OnboardingApiService

class ApiOnboardingRepository(
    private val onboardingApiService: OnboardingApiService,
) : OnboardingRepository {
    override suspend fun getOnboardingStatus(): Result<OnboardingStatusResponse> = onboardingApiService.getOnboardingStatus()

    override suspend fun completeOnboardingStep(
        step: Int,
        data: StepData,
    ): Result<OnboardingStatusResponse> = onboardingApiService.completeOnboardingStep(step, data)

    override suspend fun skipOnboarding(): Result<OnboardingStatusResponse> = onboardingApiService.skipOnboarding()
}
