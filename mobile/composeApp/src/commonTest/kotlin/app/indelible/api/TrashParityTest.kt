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

class TrashParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun listTrashSendsGetToTrash() =
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
                    respond(paginatedTrashItemsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listTrash()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/library/trash", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun getLibraryCountSendsGet() =
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
                    respond("""{"saved_count":3}""", HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getLibraryCount()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/library/count", capturedPath)
            assertTrue(result.isSuccess)
            assertEquals(3L, result.getOrThrow().savedCount)
        }

    @Test
    fun emptyTrashSendsPost() =
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
                    respond("""{"purged":3}""", HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.emptyTrash()

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/library/trash/empty", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun permanentlyDeleteItemSendsPurge() =
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
                    respond("", HttpStatusCode.NoContent, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.permanentlyDeleteItem("lib_01ABC")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/library/lib_01ABC/purge", capturedPath)
            assertTrue(result.isSuccess)
        }

    private fun paginatedTrashItemsJson() =
        """
        {
            "data": [
                {
                    "library_entry_id": "lib_01ABC",
                    "document_id": "doc_01ABC",
                    "object": "library_entry",
                    "title": "Test",
                    "url": "https://ex.com",
                    "canonical_url": "https://ex.com",
                    "source": "https://ex.com",
                    "document_type": "article",
                    "triage_state": "trash",
                    "is_favorite": false,
                    "is_shortlisted": false,
                    "saved_at": "2026-01-01T00:00:00Z",
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z"
                }
            ],
            "page": {"has_more": false, "next_cursor": null}
        }
        """.trimIndent()

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
