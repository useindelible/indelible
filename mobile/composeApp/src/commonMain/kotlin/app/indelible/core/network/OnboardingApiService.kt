package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1OnboardingClient
import app.indelible.api.generated.client.ApiV1OnboardingSkipClient
import app.indelible.api.generated.client.ApiV1OnboardingStepsCompleteClient
import app.indelible.core.model.CompleteStepRequest
import app.indelible.core.model.OnboardingStatusResponse
import app.indelible.core.model.StepData

class OnboardingApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun getOnboardingStatus(): Result<OnboardingStatusResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1OnboardingClient(client).getOnboarding(configuration)
        }

    suspend fun completeOnboardingStep(
        step: Int,
        data: StepData = StepData(),
    ): Result<OnboardingStatusResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1OnboardingStepsCompleteClient(client).completeStep(
                completeStepRequest = CompleteStepRequest(data = data),
                step = step,
                apiConfiguration = configuration,
            )
        }

    suspend fun skipOnboarding(): Result<OnboardingStatusResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1OnboardingSkipClient(client).skipOnboarding(configuration)
        }
}
