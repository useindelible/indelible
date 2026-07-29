package app.indelible.onboarding.viewmodel

import app.indelible.core.model.OnboardingStatusResponse
import app.indelible.core.model.StepData
import app.indelible.onboarding.repository.OnboardingRepository
import kotlin.test.Test
import kotlin.test.assertNotNull

class OnboardingViewModelRepositoryBoundaryTest {
    @Test
    fun onboardingViewModelIsConstructedFromRepository() {
        val viewModel = OnboardingViewModel(FakeOnboardingRepository())

        assertNotNull(viewModel)
    }
}

private class FakeOnboardingRepository : OnboardingRepository {
    override suspend fun getOnboardingStatus(): Result<OnboardingStatusResponse> = unused()

    override suspend fun completeOnboardingStep(
        step: Int,
        data: StepData,
    ): Result<OnboardingStatusResponse> = unused()

    override suspend fun skipOnboarding(): Result<OnboardingStatusResponse> = unused()
}

private fun <T> unused(): Result<T> = Result.failure(UnsupportedOperationException("not used"))
