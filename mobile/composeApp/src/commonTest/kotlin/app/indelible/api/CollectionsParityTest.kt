package app.indelible.api

import app.indelible.api.generated.models.CreateCollectionBody
import app.indelible.api.generated.models.UpdateCollectionBody
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

class CollectionsParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun listCollectionsSendsGetToCollections() =
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
                    respond(paginatedCollectionsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listCollections()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/collections", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun listCollectionsPassesCursorParam() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedCursor: String? = null
            val engine =
                MockEngine { request ->
                    capturedCursor = request.url.parameters["cursor"]
                    respond(paginatedCollectionsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            apiClient.listCollections(cursor = "abc")

            assertEquals("abc", capturedCursor)
        }

    @Test
    fun createCollectionSendsPost() =
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
                    respond(collectionJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.createCollection(CreateCollectionBody(name = "Test Collection"))

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/collections", capturedPath)
            assertTrue(result.isSuccess)
            assertEquals("Test Collection", result.getOrThrow().name)
        }

    @Test
    fun getCollectionSendsGetWithId() =
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
                    respond(collectionJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getCollection("col_01ABC")

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/collections/col_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun updateCollectionSendsPatch() =
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
                    respond(collectionJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.updateCollection("col_01ABC", UpdateCollectionBody(name = "Updated"))

            assertEquals(HttpMethod.Patch, capturedMethod)
            assertEquals("/api/v1/collections/col_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun deleteCollectionSendsDelete() =
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
            val result = apiClient.deleteCollection("col_01ABC")

            assertEquals(HttpMethod.Delete, capturedMethod)
            assertEquals("/api/v1/collections/col_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun listCollectionChildrenSendsGet() =
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
                    respond(paginatedCollectionsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listCollectionChildren("col_01ABC")

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/collections/col_01ABC/children", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun listCollectionItemsSendsGet() =
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
            val result = apiClient.listCollectionItems("col_01ABC")

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/collections/col_01ABC/entries", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun addItemToCollectionSendsPost() =
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
            val result = apiClient.addItemToCollection("col_01ABC", "lib_01ABC")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/collections/col_01ABC/entries", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun removeItemFromCollectionSendsDelete() =
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
            val result = apiClient.removeItemFromCollection("col_01ABC", "lib_01ABC")

            assertEquals(HttpMethod.Delete, capturedMethod)
            assertEquals("/api/v1/collections/col_01ABC/entries/lib_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    private fun collectionJson() =
        """
        {
            "id": "col_01ABC",
            "object": "collection",
            "name": "Test Collection",
            "item_count": 0,
            "sort_order": 0,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
        """.trimIndent()

    private fun paginatedCollectionsJson() =
        """
        {
            "data": [
                {
                    "id": "col_01ABC",
                    "object": "collection",
                    "name": "Test Collection",
                    "item_count": 0,
                    "sort_order": 0,
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
