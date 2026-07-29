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
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class SearchParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun searchSendsGetWithQuery() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            var capturedQuery: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    capturedQuery = request.url.parameters["q"]
                    respond(searchResultsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val query = "foo & bar#\u96ea"
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.search(query)

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/search", capturedPath)
            assertEquals(query, capturedQuery)
            assertTrue(result.isSuccess)
        }

    @Test
    fun searchIncludesAuthHeader() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedAuthHeader: String? = null
            val engine =
                MockEngine { request ->
                    capturedAuthHeader = request.headers["Authorization"]
                    respond(searchResultsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            apiClient.search("kotlin")

            assertNotNull(capturedAuthHeader)
            assertTrue(capturedAuthHeader!!.startsWith("Bearer "))
        }

    @Test
    fun searchResponseDeserializes() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            val engine = MockEngine { respond(searchResultsJson(), HttpStatusCode.OK, jsonHeaders) }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.search("test")

            assertTrue(result.isSuccess)
        }

    @Test
    fun suggestionsSendsGetWithQuery() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedPath: String? = null
            var capturedQuery: String? = null
            val engine =
                MockEngine { request ->
                    capturedPath = request.url.encodedPath
                    capturedQuery = request.url.parameters["q"]
                    respond(suggestionsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.suggestions("kot")

            assertEquals("/api/v1/search/suggestions", capturedPath)
            assertNotNull(capturedQuery)
            assertTrue(result.isSuccess)
        }

    @Test
    fun listRecentSearchesSendsGet() =
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
                    respond(recentSearchListJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listRecentSearches()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/search/recent", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun deleteRecentSearchSendsDelete() =
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
            val result = apiClient.deleteRecentSearch("rs_01ABC")

            assertEquals(HttpMethod.Delete, capturedMethod)
            assertEquals("/api/v1/search/recent/rs_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun clearRecentSearchesSendsDelete() =
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
            val result = apiClient.clearRecentSearches()

            assertEquals(HttpMethod.Delete, capturedMethod)
            assertEquals("/api/v1/search/recent", capturedPath)
            assertTrue(result.isSuccess)
        }

    private fun searchResultsJson() =
        """
        {
            "query": "test",
            "has_more": false,
            "results": []
        }
        """.trimIndent()

    private fun suggestionsJson() =
        """
        {
            "query": "kot",
            "suggestions": [
                {
                    "kind": "query",
                    "label": "kotlin",
                    "insert_text": "kotlin"
                }
            ]
        }
        """.trimIndent()

    private fun recentSearchListJson() =
        """
        {
            "items": [
                {
                    "id": "rs_01ABC",
                    "query": "kotlin",
                    "normalized_query": "kotlin",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z",
                    "last_searched_at": "2024-01-01T00:00:00Z"
                }
            ]
        }
        """.trimIndent()

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
