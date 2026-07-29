package app.indelible.onboarding.viewmodel

import app.indelible.core.model.StepData
import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.onboarding.repository.ApiOnboardingRepository
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

    private fun createTokenStorageWithToken(): InMemoryTokenStorage {
        val storage = InMemoryTokenStorage()
        kotlinx.coroutines.runBlocking { storage.saveToken("test-token") }
        return storage
    }

    private val onboardingStatusResponse =
        """
        {
            "current_step": 0,
            "completed": false,
            "steps": [
                {"step": 1, "name": "Account Setup", "completed": false},
                {"step": 2, "name": "Add Content", "completed": false},
                {"step": 3, "name": "RSS Feeds", "completed": false},
                {"step": 4, "name": "AI Configuration", "completed": false},
                {"step": 5, "name": "Complete", "completed": false}
            ]
        }
        """.trimIndent()

    private val partiallyCompletedStatusResponse =
        """
        {
            "current_step": 3,
            "completed": false,
            "steps": [
                {"step": 1, "name": "Account Setup", "completed": true},
                {"step": 2, "name": "Add Content", "completed": true},
                {"step": 3, "name": "RSS Feeds", "completed": true},
                {"step": 4, "name": "AI Configuration", "completed": false},
                {"step": 5, "name": "Complete", "completed": false}
            ]
        }
        """.trimIndent()

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
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

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
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

            viewModel.initialize()
            viewModel.state.first { !it.isLoading }

            val state = viewModel.state.value
            assertEquals(4, state.currentPage)
            assertTrue(state.steps[0].completed)
            assertTrue(state.steps[1].completed)
            assertTrue(state.steps[2].completed)
            assertFalse(state.steps[3].completed)
        }

    private val step1CompletedResponse =
        """
        {
            "current_step": 1,
            "completed": false,
            "steps": [
                {"step": 1, "name": "Account Setup", "completed": true},
                {"step": 2, "name": "Add Content", "completed": false},
                {"step": 3, "name": "RSS Feeds", "completed": false},
                {"step": 4, "name": "AI Configuration", "completed": false},
                {"step": 5, "name": "Complete", "completed": false}
            ]
        }
        """.trimIndent()

    private val allCompletedResponse =
        """
        {
            "current_step": 5,
            "completed": true,
            "steps": [
                {"step": 1, "name": "Account Setup", "completed": true},
                {"step": 2, "name": "Add Content", "completed": true},
                {"step": 3, "name": "RSS Feeds", "completed": true},
                {"step": 4, "name": "AI Configuration", "completed": true},
                {"step": 5, "name": "Complete", "completed": true}
            ]
        }
        """.trimIndent()

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
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

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
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

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
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

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
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

            viewModel.initialize()
            viewModel.state.first { !it.isLoading }

            val state = viewModel.state.value
            assertTrue(state.error != null)
            assertEquals(5, state.steps.size)
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
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

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
    fun updateDisplayNameUpdatesState() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine = MockEngine { respond("", HttpStatusCode.NotFound) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

            viewModel.updateDisplayName("Test User")

            assertEquals("Test User", viewModel.state.value.displayName)
        }

    @Test
    fun updateSelectedThemeUpdatesState() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine = MockEngine { respond("", HttpStatusCode.NotFound) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

            viewModel.updateSelectedTheme(ThemeChoice.DARK)

            assertEquals(ThemeChoice.DARK, viewModel.state.value.selectedTheme)
        }

    @Test
    fun toggleFeedAddsAndRemoves() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine = MockEngine { respond("", HttpStatusCode.NotFound) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

            val feedUrl = "https://example.com/feed"

            viewModel.toggleFeed(feedUrl)
            assertTrue(
                viewModel.state.value.selectedFeeds
                    .contains(feedUrl),
            )

            viewModel.toggleFeed(feedUrl)
            assertFalse(
                viewModel.state.value.selectedFeeds
                    .contains(feedUrl),
            )
        }

    @Test
    fun updateAiProviderUpdatesState() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine = MockEngine { respond("", HttpStatusCode.NotFound) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

            viewModel.updateSelectedAiProvider(AiProvider.OLLAMA)

            assertEquals(AiProvider.OLLAMA, viewModel.state.value.selectedAiProvider)
        }

    @Test
    fun clearErrorResetsError() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine =
                MockEngine {
                    respond(
                        content = """{"error": "server_error"}""",
                        status = HttpStatusCode.InternalServerError,
                        headers = jsonHeaders,
                    )
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

            viewModel.initialize()
            viewModel.state.first { !it.isLoading }
            assertTrue(viewModel.state.value.error != null)

            viewModel.clearError()
            assertNull(viewModel.state.value.error)
        }

    @Test
    fun defaultStepsReturnsFivePersistedSteps() {
        val steps = OnboardingViewModel.defaultSteps()
        assertEquals(5, steps.size)
        assertEquals(1, steps.first().number)
        assertEquals("Account Setup", steps.first().name)
        assertEquals(5, steps.last().number)
        assertEquals("Ready", steps.last().name)
        assertTrue(steps.none { it.completed })
    }

    @Test
    fun updateCurrentPageUpdatesState() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine = MockEngine { respond("", HttpStatusCode.NotFound) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

            viewModel.updateCurrentPage(3)

            assertEquals(3, viewModel.state.value.currentPage)
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
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

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
            val viewModel = OnboardingViewModel(ApiOnboardingRepository(apiClient.onboardingApiService))

            viewModel.initialize()
            viewModel.state.first { !it.isLoading }

            viewModel.completeStep(2, StepData(displayName = "Test", theme = "dark"))
            viewModel.state.first { !it.isStepLoading }

            assertTrue(requestBody != null)
            assertTrue(requestBody!!.contains("display_name"))
            assertTrue(requestBody!!.contains("Test"))
        }
}
