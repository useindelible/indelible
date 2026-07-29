package app.indelible.api

import app.indelible.api.generated.models.CreateTagBody
import app.indelible.api.generated.models.MergeTagsBody
import app.indelible.api.generated.models.UpdateTagBody
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
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class TagsParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun listTagsSendsGetToTags() =
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
                    respond(paginatedTagsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listTags()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/tags", capturedPath)
            assertTrue(result.isSuccess)
            assertNotNull(result.getOrThrow())
            assertTrue(result.getOrThrow() is List)
        }

    @Test
    fun createTagSendsPost() =
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
                    respond(tagJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.createTag(CreateTagBody(name = "kotlin"))

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/tags", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun getTagSendsGetWithId() =
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
                    respond(tagJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getTag("tag_01ABC")

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/tags/tag_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun updateTagSendsPatch() =
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
                    respond(tagJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.updateTag("tag_01ABC", UpdateTagBody(name = "kotlin-updated"))

            assertEquals(HttpMethod.Patch, capturedMethod)
            assertEquals("/api/v1/tags/tag_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun deleteTagSendsDelete() =
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
            val result = apiClient.deleteTag("tag_01ABC")

            assertEquals(HttpMethod.Delete, capturedMethod)
            assertEquals("/api/v1/tags/tag_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun mergeTagsSendsPost() =
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
                    respond(tagJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.mergeTags(MergeTagsBody(sourceIds = listOf("tag_01ABC"), targetId = "tag_02DEF"))

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/tags/merge", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun listTagHighlightsSendsGet() =
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
                    respond(paginatedHighlightsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listTagHighlights("tag_01ABC")

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/tags/tag_01ABC/highlights", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun listTagItemsSendsGet() =
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
                    respond(paginatedItemsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listTagItems("tag_01ABC")

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/tags/tag_01ABC/entries", capturedPath)
            assertTrue(result.isSuccess)
        }

    private fun tagJson() =
        """
        {"id":"tag_01ABC","object":"tag","name":"kotlin","aliases":[],"highlight_count":3,"item_count":10,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}
        """.trimIndent()

    private fun paginatedTagsJson() =
        """
        {"data":[{"id":"tag_01ABC","object":"tag","name":"kotlin","aliases":[],"highlight_count":3,"item_count":10,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}],"page":{"has_more":false}}
        """.trimIndent()

    private fun paginatedHighlightsJson() =
        """
        {
            "data": [
                {
                    "id": "hl_01ABC",
                    "document_id": "doc_01ABC",
                    "color": "yellow",
                    "text_content": "some highlighted text",
                    "locator": {"type": "html", "start_offset": 10, "end_offset": 30},
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z"
                }
            ],
            "page": {"has_more": false, "next_cursor": null}
        }
        """.trimIndent()

    private fun paginatedItemsJson() =
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
                    "triage_state": "inbox",
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
