package app.indelible.share

import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.share.model.PendingItem
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class SaveUrlUseCaseTest {
    private val validUrl = "https://example.com/article"

    private fun buildEngine(handler: (path: String) -> Pair<String, HttpStatusCode>) =
        MockEngine { request ->
            val (body, status) = handler(request.url.encodedPath)
            respond(
                content = body,
                status = status,
                headers = headersOf(HttpHeaders.ContentType, "application/json"),
            )
        }

    private fun saveItemSuccess() =
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
    fun saveValidUrlSuccess() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("valid-token")

            val engine =
                buildEngine { _ ->
                    saveItemSuccess() to HttpStatusCode.Accepted
                }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val repo = FakePendingSaveRepository()
            val useCase = SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, repo)

            val result = useCase.save(validUrl)

            assertIs<SaveResult.Success>(result)
            assertTrue(repo.items.isEmpty())
        }

    @Test
    fun saveInvalidUrlReturnsInvalidUrl() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("valid-token")

            val engine = buildEngine { _ -> "" to HttpStatusCode.OK }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val repo = FakePendingSaveRepository()
            val useCase = SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, repo)

            val result = useCase.save("not a url")

            assertIs<SaveResult.InvalidUrl>(result)
            assertTrue(repo.items.isEmpty())
        }

    @Test
    fun saveNoTokenReturnsAuthRequired() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()

            val engine = buildEngine { _ -> "" to HttpStatusCode.OK }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val repo = FakePendingSaveRepository()
            val useCase = SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, repo)

            val result = useCase.save(validUrl)

            assertIs<SaveResult.AuthRequired>(result)
            assertTrue(repo.items.isEmpty())
        }

    @Test
    fun saveWithOnlyRefreshTokenProceedsToApi() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveRefreshToken("refresh-only")

            var libraryCalls = 0
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/refresh" ->
                            respond(
                                content =
                                    """
                                    {
                                        "access_token": "new-access",
                                        "refresh_token": "new-refresh",
                                        "expires_at": 4102444800
                                    }
                                    """.trimIndent(),
                                status = HttpStatusCode.OK,
                                headers = headersOf(HttpHeaders.ContentType, "application/json"),
                            )
                        "/api/v1/library" -> {
                            libraryCalls++
                            respond(
                                content = saveItemSuccess(),
                                status = HttpStatusCode.Accepted,
                                headers = headersOf(HttpHeaders.ContentType, "application/json"),
                            )
                        }
                        else -> respond("", HttpStatusCode.NotFound)
                    }
                }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val repo = FakePendingSaveRepository()
            val useCase = SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, repo)

            val result = useCase.save(validUrl)

            assertIs<SaveResult.Success>(result)
            assertEquals(1, libraryCalls)
            assertEquals("new-refresh", tokenStorage.getRefreshToken())
        }

    @Test
    fun save401ReturnsAuthRequired() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("expired-token")

            val engine =
                buildEngine { _ ->
                    """{"error":"unauthorized"}""" to HttpStatusCode.Unauthorized
                }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val repo = FakePendingSaveRepository()
            val useCase = SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, repo)

            val result = useCase.save(validUrl)

            assertIs<SaveResult.AuthRequired>(result)
        }

    @Test
    fun save409ReturnsAlreadySaved() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("valid-token")

            val engine =
                buildEngine { _ ->
                    """{"error":"conflict"}""" to HttpStatusCode.Conflict
                }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val repo = FakePendingSaveRepository()
            val useCase = SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, repo)

            val result = useCase.save(validUrl)

            assertIs<SaveResult.AlreadySaved>(result)
        }

    @Test
    fun saveOfflineEnqueues() =
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

            val result = useCase.save(validUrl)

            assertIs<SaveResult.Queued>(result)
            assertEquals(1, repo.items.size)
            assertEquals(validUrl, repo.items.first().url)
        }

    @Test
    fun saveDrainsQueueBeforeSaving() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("valid-token")

            val requestUrls = mutableListOf<String>()
            val engine =
                MockEngine { request ->
                    requestUrls.add(request.url.toString())
                    respond(
                        content = saveItemSuccess(),
                        status = HttpStatusCode.Accepted,
                        headers = headersOf(HttpHeaders.ContentType, "application/json"),
                    )
                }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val repo = FakePendingSaveRepository()
            repo.enqueue(PendingItem("id-1", "https://pending1.com", 1000L))
            repo.enqueue(PendingItem("id-2", "https://pending2.com", 2000L))
            val useCase = SaveUrlUseCase(apiClient.libraryApiService, tokenStorage, repo)

            val result = useCase.save(validUrl)

            assertIs<SaveResult.Success>(result)
            assertEquals(3, requestUrls.size)
            assertTrue(repo.items.isEmpty())
        }

    @Test
    fun drainRequeuesRemainingItemsOnNetworkFailure() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("valid-token")

            val fakeNetworkException = RuntimeException("Connection refused")
            var requestCount = 0
            val engine =
                MockEngine { request ->
                    requestCount++
                    if (requestCount == 2) {
                        throw fakeNetworkException
                    }
                    respond(
                        content = saveItemSuccess(),
                        status = HttpStatusCode.Accepted,
                        headers = headersOf(HttpHeaders.ContentType, "application/json"),
                    )
                }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val repo = FakePendingSaveRepository()
            repo.enqueue(PendingItem("id-1", "https://pending1.com", 1000L))
            repo.enqueue(PendingItem("id-2", "https://pending2.com", 2000L))
            val useCase =
                SaveUrlUseCase(
                    apiClient.libraryApiService,
                    tokenStorage,
                    repo,
                    networkExceptionDetector = { it === fakeNetworkException || it.cause === fakeNetworkException },
                )

            val result = useCase.save(validUrl)

            // Request 1: drain sends pending1 -> success.
            // Request 2: drain sends pending2 -> network exception. drainQueue re-enqueues
            // pending2 (and any items after it) then returns.
            // Request 3: save() sends the new URL -> success (network recovered).
            assertIs<SaveResult.Success>(result)
            assertEquals(3, requestCount)
            assertEquals(1, repo.items.size)
            assertEquals("https://pending2.com", repo.items[0].url)
        }
}
