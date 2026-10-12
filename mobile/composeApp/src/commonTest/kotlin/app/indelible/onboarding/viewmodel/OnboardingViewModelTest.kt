package app.indelible.onboarding.viewmodel

import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.core.network.ApiClient
import app.indelible.onboarding.repository.ApiOnboardingRepository
import app.indelible.share.FakePendingSaveRepository
import app.indelible.share.SaveUrlUseCase
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
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
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class OnboardingViewModelTest {
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
    fun initializeFetchesOnboardingStatus() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/onboarding" ->
                            respond(
                                content = onboardingStatusResponse,
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
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

            val state = viewModel.state.value
            assertEquals(5, state.steps.size)
            assertEquals(0, state.currentPage)
            assertFalse(state.isCompleted)
            assertNull(state.error)
        }

    @Test
    fun initializeResumesAtFirstIncompleteStep() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/onboarding" ->
                            respond(
                                content = partiallyCompletedStatusResponse,
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
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

            val state = viewModel.state.value
            assertEquals(4, state.currentPage)
            assertTrue(state.steps[0].completed)
            assertTrue(state.steps[1].completed)
            assertTrue(state.steps[2].completed)
            assertFalse(state.steps[3].completed)
        }

    @Test
    fun initializeHandlesApiFailure() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine =
                MockEngine {
                    respond(
                        content = """{"error": "server_error", "message": "Internal server error"}""",
                        status = HttpStatusCode.InternalServerError,
                        headers = jsonHeaders,
                    )
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

            val state = viewModel.state.value
            assertTrue(state.error != null)
            assertEquals(5, state.steps.size)
        }

    // ---- persistence of accepted onboarding values ----
}
