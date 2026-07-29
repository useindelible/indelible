package app.indelible.api

import app.indelible.api.generated.models.CreateApiTokenRequest
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

class TokensParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun listApiTokensSendsGet() =
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
                    respond(tokenListJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listApiTokens()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/tokens", capturedPath)
            assertTrue(result.isSuccess)
            assertEquals(1, result.getOrThrow().size)
        }

    @Test
    fun createApiTokenSendsPost() =
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
                    respond(createApiTokenResponseJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result =
                apiClient.createApiToken(
                    CreateApiTokenRequest(name = "My Token", scopes = listOf("read")),
                )

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/tokens", capturedPath)
            assertTrue(result.isSuccess)
            assertEquals("ind_secret_token", result.getOrThrow().rawToken)
        }

    @Test
    fun deleteApiTokenSendsDelete() =
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
            val result = apiClient.deleteApiToken("tok_01ABC")

            assertEquals(HttpMethod.Delete, capturedMethod)
            assertEquals("/api/v1/tokens/tok_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    private fun tokenListJson() =
        """
        {"data":[{"id":"tok_01ABC","object":"api_token","name":"My Token","prefix":"ind_","scopes":["read"],"created_at":"2026-01-01T00:00:00Z"}]}
        """.trimIndent()

    private fun createApiTokenResponseJson() =
        """
        {"id":"tok_01ABC","object":"api_token","name":"My Token","prefix":"ind_","scopes":["read"],"created_at":"2026-01-01T00:00:00Z","raw_token":"ind_secret_token"}
        """.trimIndent()

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
