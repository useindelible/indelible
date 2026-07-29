package app.indelible.onboarding.viewmodel

import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.core.model.OnboardingStatusResponse
import app.indelible.core.model.StepData
import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.onboarding.repository.OnboardingRepository
import app.indelible.share.FakePendingSaveRepository
import app.indelible.share.SaveUrlUseCase
import kotlin.test.Test
import kotlin.test.assertNotNull

class OnboardingViewModelRepositoryBoundaryTest {
    @Test
    fun onboardingViewModelIsConstructedFromRepository() {
        val tokenStorage = InMemoryTokenStorage()
        val apiClient = ApiClient(tokenStorage)
        val viewModel =
            OnboardingViewModel(
                FakeOnboardingRepository(),
                ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
            )

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
