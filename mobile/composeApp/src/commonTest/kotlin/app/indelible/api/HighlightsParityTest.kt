package app.indelible.api

import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.reader.model.CreateHighlightRequest
import app.indelible.reader.model.HighlightLocator
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

class HighlightsParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }

    private fun locatorJson() =
        """
        {"type": "html", "start_offset": 10, "end_offset": 50}
        """.trimIndent()

    private fun highlightJson() =
        """
        {
            "id": "hlt_01ABC",
            "document_id": "doc_01ABC",
            "color": "yellow",
            "text_content": "Interesting passage",
            "locator": ${locatorJson()},
            "tags": [],
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
        """.trimIndent()

    private fun highlightResponseJson() =
        """
        {
            "id": "hlt_01ABC",
            "document_id": "doc_01ABC",
            "color": "yellow",
            "text_content": "Interesting passage",
            "locator": ${locatorJson()},
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
        """.trimIndent()

    private fun highlightListJson() =
        """
        {"count": 1, "highlights": [${highlightJson()}]}
        """.trimIndent()

    private fun recentHighlightsJson() =
        """
        {"count": 1, "highlights": [${highlightJson()}]}
        """.trimIndent()

    private fun highlightNoteJson() =
        """
        {
            "id": "note_01",
            "highlight_id": "hlt_01ABC",
            "body": "Note text",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
        """.trimIndent()

    private fun createHighlightRequest() =
        CreateHighlightRequest(
            color = "yellow",
            textContent = "Interesting passage",
            locator =
                HighlightLocator(type = "html"),
        )

    @Test
    fun listHighlightsSendsGetWithDocumentId() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedPath = request.url.encodedPath
                    respond(highlightListJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listHighlights("doc_01ABC")

            assertEquals("/api/v1/documents/doc_01ABC/highlights", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun createHighlightSendsPost() =
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
                    respond(highlightResponseJson(), HttpStatusCode.Created, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.createHighlight("doc_01ABC", createHighlightRequest())

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/documents/doc_01ABC/highlights", capturedPath)
            assertTrue(
                result.isSuccess,
                "create should parse the HighlightResponse 201 body: ${result.exceptionOrNull()}",
            )
            assertEquals(emptyList(), result.getOrThrow().tags)
        }

    @Test
    fun deleteHighlightSendsDelete() =
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
            val result = apiClient.deleteHighlight("hlt_01ABC")

            assertEquals(HttpMethod.Delete, capturedMethod)
            assertEquals("/api/v1/highlights/hlt_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun patchHighlightSendsPatch() =
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
                    respond(highlightResponseJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.patchHighlight("hlt_01ABC", "blue")

            assertEquals(HttpMethod.Patch, capturedMethod)
            assertEquals("/api/v1/highlights/hlt_01ABC", capturedPath)
            assertTrue(result.isSuccess, "patch should parse the HighlightResponse body: ${result.exceptionOrNull()}")
        }

    @Test
    fun upsertHighlightNoteSendsPut() =
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
                    respond(highlightNoteJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.upsertHighlightNote("hlt_01ABC", "Note text")

            assertEquals(HttpMethod.Put, capturedMethod)
            assertEquals("/api/v1/highlights/hlt_01ABC/note", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun deleteHighlightNoteSendsDelete() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedPath = request.url.encodedPath
                    respond("", HttpStatusCode.NoContent, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.deleteHighlightNote("hlt_01ABC")

            assertEquals("/api/v1/highlights/hlt_01ABC/note", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun setHighlightTagsSendsPut() =
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
                    respond("""{"tags": ["kotlin"]}""", HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.setHighlightTags("hlt_01ABC", listOf("kotlin"))

            assertEquals(HttpMethod.Put, capturedMethod)
            assertEquals("/api/v1/highlights/hlt_01ABC/tags", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun getHighlightTagsSendsGet() =
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
                    respond("""{"tags": ["kotlin", "mobile"]}""", HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getHighlightTags("hlt_01ABC")

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/highlights/hlt_01ABC/tags", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun listRecentHighlightsSendsGet() =
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
                    respond(recentHighlightsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listRecentHighlights()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/highlights/recent", capturedPath)
            assertTrue(result.isSuccess)
            assertEquals(1, result.getOrThrow().count)
        }
}
