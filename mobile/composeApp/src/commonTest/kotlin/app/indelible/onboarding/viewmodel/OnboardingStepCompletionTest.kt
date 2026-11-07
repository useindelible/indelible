package app.indelible.onboarding.viewmodel

import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.core.model.StepData
import app.indelible.core.network.ApiClient
import app.indelible.onboarding.repository.ApiOnboardingRepository
import app.indelible.share.FakePendingSaveRepository
import app.indelible.share.SaveUrlUseCase
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class OnboardingStepCompletionTest {
    private val testDispatcher = UnconfinedTestDispatcher()

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun completeStepCallsApiAndUpdatesState() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            var completedStepUrl: String? = null

            val engine =
                MockEngine { request ->
                    when {
                        request.url.encodedPath == "/api/v1/onboarding" ->
                            respond(
                                content = onboardingStatusResponse,
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        request.url.encodedPath.startsWith("/api/v1/onboarding/steps/") &&
                            request.method == HttpMethod.Post -> {
                            completedStepUrl = request.url.encodedPath
                            respond(
                                content = step1CompletedResponse,
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        }
                        else -> respond("", HttpStatusCode.NotFound)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel =
                OnboardingViewModel(
                    ApiOnboardingRepository(apiClient.onboardingApiService),
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
                )

            viewModel.initialize()
            viewModel.state.first { !it.isLoading }

            viewModel.completeStep(1)
            viewModel.state.first { !it.isStepLoading }

            assertEquals("/api/v1/onboarding/steps/1/complete", completedStepUrl)
            assertTrue(
                viewModel.state.value.steps[0]
                    .completed,
            )
            assertFalse(viewModel.state.value.isStepLoading)
        }

    @Test
    fun completingAlreadyCompletedStepIsNoOp() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            var apiCallCount = 0

            val engine =
                MockEngine { request ->
                    when {
                        request.url.encodedPath == "/api/v1/onboarding" ->
                            respond(
                                content = partiallyCompletedStatusResponse,
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        request.url.encodedPath.startsWith("/api/v1/onboarding/steps/") -> {
                            apiCallCount++
                            respond(
                                content = step1CompletedResponse,
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        }
                        else -> respond("", HttpStatusCode.NotFound)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel =
                OnboardingViewModel(
                    ApiOnboardingRepository(apiClient.onboardingApiService),
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
                )

            viewModel.initialize()
            viewModel.state.first { !it.isLoading }

            viewModel.completeStep(1)

            assertEquals(0, apiCallCount)
        }

    @Test
    fun skipAllCallsApiAndMarksAllComplete() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            var skipCalled = false

            val engine =
                MockEngine { request ->
                    when {
                        request.url.encodedPath == "/api/v1/onboarding" ->
                            respond(
                                content = onboardingStatusResponse,
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        request.url.encodedPath == "/api/v1/onboarding/skip" &&
                            request.method == HttpMethod.Post -> {
                            skipCalled = true
                            respond(
                                content = allCompletedResponse,
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        }
                        else -> respond("", HttpStatusCode.NotFound)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel =
                OnboardingViewModel(
                    ApiOnboardingRepository(apiClient.onboardingApiService),
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
                )

            viewModel.initialize()
            viewModel.state.first { !it.isLoading }

            viewModel.skipAll()
            viewModel.state.first { !it.isStepLoading }

            assertTrue(skipCalled)
            assertTrue(viewModel.state.value.isCompleted)
            assertTrue(
                viewModel.state.value.steps
                    .all { it.completed },
            )
        }

    @Test
    fun completeStepHandlesApiFailure() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine =
                MockEngine { request ->
                    when {
                        request.url.encodedPath == "/api/v1/onboarding" ->
                            respond(
                                content = onboardingStatusResponse,
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        else ->
                            respond(
                                content = """{"error": "server_error", "message": "Failed"}""",
                                status = HttpStatusCode.InternalServerError,
                                headers = jsonHeaders,
                            )
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel =
                OnboardingViewModel(
                    ApiOnboardingRepository(apiClient.onboardingApiService),
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
                )

            viewModel.initialize()
            viewModel.state.first { !it.isLoading }

            viewModel.completeStep(1)
            viewModel.state.first { !it.isStepLoading }

            assertFalse(
                viewModel.state.value.steps[0]
                    .completed,
            )
            assertTrue(viewModel.state.value.error != null)
        }

    @Test
    fun skipAllHandlesApiFailure() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine =
                MockEngine { request ->
                    when {
                        request.url.encodedPath == "/api/v1/onboarding" ->
                            respond(
                                content = onboardingStatusResponse,
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        else ->
                            respond(
                                content = """{"error": "server_error", "message": "Failed"}""",
                                status = HttpStatusCode.InternalServerError,
                                headers = jsonHeaders,
                            )
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel =
                OnboardingViewModel(
                    ApiOnboardingRepository(apiClient.onboardingApiService),
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
                )

            viewModel.initialize()
            viewModel.state.first { !it.isLoading }

            viewModel.skipAll()
            viewModel.state.first { !it.isStepLoading }

            assertFalse(viewModel.state.value.isCompleted)
            assertTrue(viewModel.state.value.error != null)
        }

    @Test
    fun completeStepWithDataSendsData() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            var requestBody: String? = null

            val engine =
                MockEngine { request ->
                    when {
                        request.url.encodedPath == "/api/v1/onboarding" ->
                            respond(
                                content = onboardingStatusResponse,
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        request.url.encodedPath.startsWith("/api/v1/onboarding/steps/") -> {
                            requestBody = (request.body as io.ktor.http.content.TextContent).text
                            respond(
                                content = step1CompletedResponse,
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        }
                        else -> respond("", HttpStatusCode.NotFound)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel =
                OnboardingViewModel(
                    ApiOnboardingRepository(apiClient.onboardingApiService),
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
                )

            viewModel.initialize()
            viewModel.state.first { !it.isLoading }

            viewModel.completeStep(2, StepData(displayName = "Test", theme = "dark"))
            viewModel.state.first { !it.isStepLoading }

            assertTrue(requestBody != null)
            assertTrue(requestBody!!.contains("display_name"))
            assertTrue(requestBody!!.contains("Test"))
        }
}
