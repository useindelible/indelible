package app.indelible.onboarding.viewmodel

import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.core.network.ApiClient
import app.indelible.onboarding.repository.ApiOnboardingRepository
import app.indelible.share.FakePendingSaveRepository
import app.indelible.share.SaveUrlUseCase
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.onboarding_step_account
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
class OnboardingLocalStateTest {
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
    fun updateDisplayNameUpdatesState() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine = MockEngine { respond("", HttpStatusCode.NotFound) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel =
                OnboardingViewModel(
                    ApiOnboardingRepository(apiClient.onboardingApiService),
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
                )

            viewModel.updateDisplayName("Test User")

            assertEquals("Test User", viewModel.state.value.displayName)
        }

    @Test
    fun updateSelectedThemeUpdatesState() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine = MockEngine { respond("", HttpStatusCode.NotFound) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel =
                OnboardingViewModel(
                    ApiOnboardingRepository(apiClient.onboardingApiService),
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
                )

            viewModel.updateSelectedTheme(ThemeChoice.DARK)

            assertEquals(ThemeChoice.DARK, viewModel.state.value.selectedTheme)
        }

    @Test
    fun toggleFeedAddsAndRemoves() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine = MockEngine { respond("", HttpStatusCode.NotFound) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel =
                OnboardingViewModel(
                    ApiOnboardingRepository(apiClient.onboardingApiService),
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
                )

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
            val viewModel =
                OnboardingViewModel(
                    ApiOnboardingRepository(apiClient.onboardingApiService),
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
                )

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
            val viewModel =
                OnboardingViewModel(
                    ApiOnboardingRepository(apiClient.onboardingApiService),
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
                )

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
        assertEquals(5, steps.last().number)
        assertTrue(steps.none { it.completed })
        assertEquals(Res.string.onboarding_step_account, OnboardingPage.ACCOUNT_SETUP.labelRes)
    }

    @Test
    fun updateCurrentPageUpdatesState() =
        runTest {
            val tokenStorage = createTokenStorageWithToken()
            val engine = MockEngine { respond("", HttpStatusCode.NotFound) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel =
                OnboardingViewModel(
                    ApiOnboardingRepository(apiClient.onboardingApiService),
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
                )

            viewModel.updateCurrentPage(3)

            assertEquals(3, viewModel.state.value.currentPage)
        }
}
