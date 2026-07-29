package app.indelible.api

import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class OnboardingParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun getOnboardingStatusSendsGetToOnboarding() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(onboardingStatusJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getOnboardingStatus()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/onboarding", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun completeOnboardingStepSendsPost() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(onboardingStatusJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.completeOnboardingStep(1)

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/onboarding/steps/1/complete", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun skipOnboardingSendsPost() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(onboardingStatusJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.skipOnboarding()

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/onboarding/skip", capturedPath)
            assertTrue(result.isSuccess)
        }

    private fun onboardingStatusJson() =
        """
        {
            "completed": false,
            "current_step": 1,
            "steps": [{"step": 1, "name": "profile", "completed": false}]
        }
        """.trimIndent()

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
