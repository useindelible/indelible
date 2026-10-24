package app.indelible.share

import androidx.lifecycle.viewModelScope
import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.share.viewmodel.ShareUiState
import app.indelible.share.viewmodel.ShareViewModel
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.share_error
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.job
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestResult
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.withContext
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

@OptIn(ExperimentalCoroutinesApi::class)
class ShareViewModelTest {
    private val validUrl = "https://example.com/article"
    private val viewModels = mutableListOf<ShareViewModel>()

    @BeforeTest
    fun setUp() {
        // The save answers from the engine's own thread; an unconfined Main refuses that resume.
        Dispatchers.setMain(StandardTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private fun vmTest(body: suspend TestScope.() -> Unit): TestResult =
        runTest {
            try {
                body()
            } finally {
                withContext(NonCancellable) {
                    viewModels.forEach { it.viewModelScope.coroutineContext.job.cancelAndJoin() }
                }
            }
        }

    private fun newViewModel(useCase: SaveUrlUseCase) = ShareViewModel(useCase).also(viewModels::add)

    private fun saveItemJson() =
        """
        {
            "library_entry_id": "lib_01ABC",
            "document_id": "doc_01ABC",
            "object": "library_entry",
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

    @Test
    fun uiStateTransitionsLoadingToSuccess() =
        vmTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("valid-token")

            val engine =
                MockEngine { request ->
                    if (request.url.encodedPath == "/api/v1/library") {
                        respond(
                            content = saveItemJson(),
                            status = HttpStatusCode.Accepted,
                            headers = headersOf(HttpHeaders.ContentType, "application/json"),
                        )
                    } else {
                        respond("", HttpStatusCode.OK)
                    }
                }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val repo = FakePendingSaveRepository()
            val useCase = SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, repo)
            val viewModel = newViewModel(useCase)

            assertIs<ShareUiState.Idle>(viewModel.uiState.value)

            viewModel.save(validUrl)
            val terminal = viewModel.uiState.first { it !is ShareUiState.Idle && it !is ShareUiState.Saving }

            assertIs<ShareUiState.Success>(terminal)
        }

    @Test
    fun uiStateAuthRequiredOnNullToken() =
        vmTest {
            val tokenStorage = InMemoryTokenStorage()

            val engine = MockEngine { respond("", HttpStatusCode.OK) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val repo = FakePendingSaveRepository()
            val useCase = SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, repo)
            val viewModel = newViewModel(useCase)

            viewModel.save(validUrl)
            val terminal = viewModel.uiState.first { it !is ShareUiState.Idle && it !is ShareUiState.Saving }

            assertIs<ShareUiState.AuthRequired>(terminal)
        }

    @Test
    fun uiStateQueuedOnOffline() =
        vmTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("valid-token")

            val fakeNetworkException = RuntimeException("Connection refused")
            val engine = MockEngine { throw fakeNetworkException }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val repo = FakePendingSaveRepository()
            val useCase =
                SaveUrlUseCase(
                    apiClient.libraryApiService,
                    tokenStorage,
                    repo,
                    networkExceptionDetector = { it === fakeNetworkException || it.cause === fakeNetworkException },
                )
            val viewModel = newViewModel(useCase)

            viewModel.save(validUrl)
            val terminal = viewModel.uiState.first { it !is ShareUiState.Idle && it !is ShareUiState.Saving }

            assertIs<ShareUiState.Queued>(terminal)
        }

    @Test
    fun uiStateUsesLocalizedFallbackForServerFailure() =
        vmTest {
            val tokenStorage = InMemoryTokenStorage().apply { saveToken("valid-token") }
            val engine = MockEngine { respond("upstream details", HttpStatusCode.InternalServerError) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val useCase = SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, FakePendingSaveRepository())
            val viewModel = newViewModel(useCase)

            viewModel.save(validUrl)
            val terminal = viewModel.uiState.first { it !is ShareUiState.Idle && it !is ShareUiState.Saving }

            val error = assertIs<ShareUiState.Error>(terminal)
            assertEquals(Res.string.share_error, error.message.resource)
        }
}
