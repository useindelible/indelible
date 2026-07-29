package app.indelible.share

import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.share.viewmodel.ShareUiState
import app.indelible.share.viewmodel.ShareViewModel
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
import kotlin.test.assertIs

@OptIn(ExperimentalCoroutinesApi::class)
class ShareViewModelTest {
    private val validUrl = "https://example.com/article"

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

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
        runTest {
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
            val viewModel = ShareViewModel(useCase)

            assertIs<ShareUiState.Idle>(viewModel.uiState.value)

            viewModel.save(validUrl)
            val terminal = viewModel.uiState.first { it !is ShareUiState.Idle && it !is ShareUiState.Saving }

            assertIs<ShareUiState.Success>(terminal)
        }

    @Test
    fun uiStateAuthRequiredOnNullToken() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()

            val engine = MockEngine { respond("", HttpStatusCode.OK) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val repo = FakePendingSaveRepository()
            val useCase = SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, repo)
            val viewModel = ShareViewModel(useCase)

            viewModel.save(validUrl)
            val terminal = viewModel.uiState.first { it !is ShareUiState.Idle && it !is ShareUiState.Saving }

            assertIs<ShareUiState.AuthRequired>(terminal)
        }

    @Test
    fun uiStateQueuedOnOffline() =
        runTest {
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
            val viewModel = ShareViewModel(useCase)

            viewModel.save(validUrl)
            val terminal = viewModel.uiState.first { it !is ShareUiState.Idle && it !is ShareUiState.Saving }

            assertIs<ShareUiState.Queued>(terminal)
        }
}
