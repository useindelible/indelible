package app.indelible.api

import app.indelible.api.generated.models.CreateSmartListBody
import app.indelible.api.generated.models.UpdateSmartListBody
import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class SmartListsParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun listSmartListsSendsGetToSmartLists() =
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
                    respond(paginatedSmartListsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listSmartLists()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/smart-lists", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun createSmartListSendsPost() =
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
                    respond(smartListJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result =
                apiClient.createSmartList(
                    CreateSmartListBody(
                        name = "Test Smart List",
                        filterExpression =
                            buildJsonObject {
                                put("type", "and")
                                putJsonArray("conditions") {}
                            },
                    ),
                )

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/smart-lists", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun getSmartListSendsGetWithId() =
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
                    respond(smartListJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getSmartList("sl_01ABC")

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/smart-lists/sl_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun updateSmartListSendsPatch() =
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
                    respond(smartListJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.updateSmartList("sl_01ABC", UpdateSmartListBody(name = "Updated"))

            assertEquals(HttpMethod.Patch, capturedMethod)
            assertEquals("/api/v1/smart-lists/sl_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun deleteSmartListSendsDelete() =
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
            val result = apiClient.deleteSmartList("sl_01ABC")

            assertEquals(HttpMethod.Delete, capturedMethod)
            assertEquals("/api/v1/smart-lists/sl_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun listSmartListItemsSendsGet() =
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
            val result = apiClient.listSmartListItems("sl_01ABC")

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/smart-lists/sl_01ABC/entries", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun pinSmartListSendsPatch() =
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
                    respond(smartListJson(isPinned = true), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.pinSmartList("sl_01ABC", isPinned = true)

            assertEquals(HttpMethod.Patch, capturedMethod)
            assertEquals("/api/v1/smart-lists/sl_01ABC/pin", capturedPath)
            assertTrue(result.isSuccess)
            assertTrue(result.getOrThrow().isPinned)
        }

    private fun smartListJson(isPinned: Boolean = false) =
        """
        {
            "id": "sl_01ABC",
            "object": "smart_list",
            "name": "Test Smart List",
            "filter_expression": {"type": "and", "conditions": []},
            "is_pinned": $isPinned,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
        """.trimIndent()

    private fun paginatedSmartListsJson() =
        """
        {
            "data": [
                {
                    "id": "sl_01ABC",
                    "object": "smart_list",
                    "name": "Test Smart List",
                    "filter_expression": {"type": "and", "conditions": []},
                    "is_pinned": false,
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
