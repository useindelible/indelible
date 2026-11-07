package app.indelible.onboarding.viewmodel

import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.core.i18n.UiMessage
import app.indelible.core.network.ApiClient
import app.indelible.onboarding.repository.ApiOnboardingRepository
import app.indelible.share.FakePendingSaveRepository
import app.indelible.share.SaveUrlUseCase
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.onboarding_add_content_invalid_url
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
class OnboardingStepPersistenceTest {
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

    private data class RecordedRequest(
        val method: HttpMethod,
        val path: String,
        val body: String,
    )

    private fun profileJson() =
        """
        {
            "id": "usr_01ABCDEF",
            "object": "user",
            "email": "user@example.com",
            "display_name": "Momo",
            "email_verified": true,
            "onboarding_completed": false,
            "has_password": true,
            "locale": "en",
            "theme": "dark",
            "timezone": "UTC",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }
        """.trimIndent()

    private fun savedItemJson() =
        """
        {
            "library_entry_id": "lib_01ABC",
            "document_id": "doc_01ABC",
            "object": "library_entry",
            "progress_percent": null,
            "max_progress_percent": null,
            "last_read_at": null,
            "finished_at": null,
            "title": "Test Article",
            "url": "https://example.com/article",
            "canonical_url": "https://example.com/article",
            "source": "manual",
            "document_type": "article",
            "triage_state": "inbox",
            "is_favorite": false,
            "is_shortlisted": false,
            "saved_at": "2026-03-25T12:00:00Z",
            "created_at": "2026-03-25T12:00:00Z",
            "updated_at": "2026-03-25T12:00:00Z"
        }
        """.trimIndent()

    /**
     * Requests land from the engine's own dispatcher, so recording is
     * synchronized and every assertion reads a snapshot.
     */
    private class RequestLog {
        private val entries = mutableListOf<RecordedRequest>()

        fun record(entry: RecordedRequest) {
            synchronized(entries) { entries.add(entry) }
        }

        fun snapshot(): List<RecordedRequest> = synchronized(entries) { entries.toList() }
    }

    private fun routedEngine(
        log: RequestLog,
        profileStatus: HttpStatusCode = HttpStatusCode.OK,
    ) = MockEngine { request ->
        val body = (request.body as? io.ktor.http.content.TextContent)?.text ?: ""
        log.record(RecordedRequest(request.method, request.url.encodedPath, body))
        when (request.url.encodedPath) {
            "/api/v1/me" -> respond(profileJson(), profileStatus, jsonHeaders)
            "/api/v1/library" -> respond(savedItemJson(), HttpStatusCode.Accepted, jsonHeaders)
            else -> respond(onboardingStatusResponse, HttpStatusCode.OK, jsonHeaders)
        }
    }

    private suspend fun buildViewModel(engine: MockEngine): OnboardingViewModel {
        // A token with no recorded expiry sends the transport down the refresh
        // path before any request lands, so pin one far in the future.
        val tokenStorage = createTokenStorageWithToken()
        tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
        val apiClient = ApiClient(tokenStorage, engine = engine)
        val viewModel =
            OnboardingViewModel(
                ApiOnboardingRepository(apiClient.onboardingApiService),
                ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository()),
            )
        viewModel.initialize()
        viewModel.state.first { !it.isLoading }
        return viewModel
    }

    @Test
    fun accountStepPersistsDisplayNameAndThemeBeforeAdvancing() =
        runTest {
            val log = RequestLog()
            val viewModel = buildViewModel(routedEngine(log))
            viewModel.updateDisplayName("Momo")
            viewModel.updateSelectedTheme(ThemeChoice.DARK)
            var advanced = false

            viewModel.completeAccountStep { advanced = true }
            viewModel.state.first { !it.isStepLoading }

            val recorded = log.snapshot()
            val profile =
                recorded.firstOrNull { it.method == HttpMethod.Patch && it.path == "/api/v1/me" }
                    ?: throw AssertionError(
                        "no profile PATCH; recorded=" + recorded.map { it.method.value + " " + it.path },
                    )
            assertTrue(profile.body.contains("\"display_name\":\"Momo\""), profile.body)
            assertTrue(profile.body.contains("\"theme\":\"dark\""), profile.body)
            assertFalse(
                profile.body.contains("\"avatar_url\":\"Momo\""),
                "display name must never ride the avatar field: " + profile.body,
            )
            val stepIndex =
                recorded.indexOfFirst { it.path == "/api/v1/onboarding/steps/1/complete" }
            assertTrue(stepIndex > recorded.indexOf(profile), "profile must persist before the step advances")
            assertTrue(advanced)
        }

    @Test
    fun accountStepMapsAutoThemeToSystem() =
        runTest {
            val log = RequestLog()
            val viewModel = buildViewModel(routedEngine(log))
            viewModel.updateSelectedTheme(ThemeChoice.AUTO)

            viewModel.completeAccountStep()
            viewModel.state.first { !it.isStepLoading }

            val profile =
                log.snapshot().first { it.method == HttpMethod.Patch && it.path == "/api/v1/me" }
            assertTrue(profile.body.contains("\"theme\":\"system\""), profile.body)
        }

    @Test
    fun accountStepFailureStaysVisibleAndDoesNotAdvance() =
        runTest {
            val log = RequestLog()
            val viewModel =
                buildViewModel(routedEngine(log, profileStatus = HttpStatusCode.InternalServerError))
            viewModel.updateDisplayName("Momo")
            var advanced = false

            viewModel.completeAccountStep { advanced = true }
            val state = viewModel.state.first { !it.isStepLoading && it.error != null }

            assertTrue(state.error != null, "a failed profile save must stay visible")
            assertFalse(advanced)
            assertTrue(log.snapshot().none { it.path == "/api/v1/onboarding/steps/1/complete" })
        }

    @Test
    fun addContentStepSavesTheUrlBeforeAdvancing() =
        runTest {
            val log = RequestLog()
            val viewModel = buildViewModel(routedEngine(log))
            viewModel.updateUrlInput("https://example.com/article")
            var advanced = false

            viewModel.completeAddContentStep { advanced = true }
            viewModel.state.first { !it.isStepLoading }

            val recorded = log.snapshot()
            val save =
                recorded.firstOrNull { it.method == HttpMethod.Post && it.path == "/api/v1/library" }
                    ?: throw AssertionError(
                        "no library save; recorded=" + recorded.map { it.method.value + " " + it.path },
                    )
            assertTrue(save.body.contains("https://example.com/article"), save.body)
            val stepIndex =
                recorded.indexOfFirst { it.path == "/api/v1/onboarding/steps/2/complete" }
            assertTrue(stepIndex > recorded.indexOf(save), "the save must land before the step advances")
            assertTrue(advanced)
        }

    @Test
    fun addContentStepWithBlankUrlJustAdvances() =
        runTest {
            val log = RequestLog()
            val viewModel = buildViewModel(routedEngine(log))
            var advanced = false

            viewModel.completeAddContentStep { advanced = true }
            viewModel.state.first { !it.isStepLoading }

            val recorded = log.snapshot()
            assertTrue(recorded.none { it.path == "/api/v1/library" })
            assertTrue(recorded.any { it.path == "/api/v1/onboarding/steps/2/complete" })
            assertTrue(advanced)
        }

    @Test
    fun addContentStepWithInvalidUrlStaysVisibleAndDoesNotAdvance() =
        runTest {
            val log = RequestLog()
            val viewModel = buildViewModel(routedEngine(log))
            viewModel.updateUrlInput("not a url")
            var advanced = false

            viewModel.completeAddContentStep { advanced = true }
            val state = viewModel.state.first { !it.isStepLoading && it.error != null }

            assertEquals(
                UiMessage(Res.string.onboarding_add_content_invalid_url),
                state.error,
                "an unsaveable URL must stay visible",
            )
            assertFalse(advanced)
            val recorded = log.snapshot()
            assertTrue(recorded.none { it.path == "/api/v1/library" })
            assertTrue(recorded.none { it.path == "/api/v1/onboarding/steps/2/complete" })
        }

    private companion object {
        const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
