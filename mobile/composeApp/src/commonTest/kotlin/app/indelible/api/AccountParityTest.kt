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

class AccountParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun getSessionSendsGetToMe() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            var capturedAuthHeader: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    capturedAuthHeader = request.headers["Authorization"]
                    respond(profileResponseJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            apiClient.getSession()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/me", capturedPath)
            assertNotNull(capturedAuthHeader)
            assertTrue(capturedAuthHeader!!.startsWith("Bearer "))
        }

    @Test
    fun getSessionReturnsAuthUser() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            val engine = MockEngine { respond(profileResponseJson(), HttpStatusCode.OK, jsonHeaders) }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getSession()

            assertTrue(result.isSuccess)
            val user = result.getOrThrow()
            assertEquals("Test User", user.displayName)
        }

    @Test
    fun updateProfileSendsPatchToMe() =
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
                    respond(profileResponseJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            apiClient.updateProfile("Test User")

            assertEquals(HttpMethod.Patch, capturedMethod)
            assertEquals("/api/v1/me", capturedPath)
        }

    @Test
    fun deleteAccountSendsDeleteToMe() =
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
            val result = apiClient.deleteAccount("delete")

            assertEquals(HttpMethod.Delete, capturedMethod)
            assertEquals("/api/v1/me", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun changeEmailSendsPost() =
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
            val result = apiClient.changeEmail("new@example.com", "password")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/me/email", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun changePasswordSendsPost() =
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
            val result = apiClient.changePassword("old-password", "new-password")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/me/password", capturedPath)
            assertTrue(result.isSuccess)
        }

    private fun profileResponseJson() =
        """
        {
            "id": "usr_01ABCDEF",
            "object": "user",
            "email": "user@example.com",
            "display_name": "Test User",
            "email_verified": true,
            "onboarding_completed": true,
            "has_password": true,
            "locale": "en",
            "theme": "auto",
            "timezone": "UTC",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }
        """.trimIndent()

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
