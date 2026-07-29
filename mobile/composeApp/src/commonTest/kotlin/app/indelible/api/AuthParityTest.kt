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
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class AuthParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun loginSendsPostToAuthLogin() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(authResponseJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.login("user@example.com", "password")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/auth/login", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun loginResponseDeserializesTokens() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val engine = MockEngine { respond(authResponseJson(), HttpStatusCode.OK, jsonHeaders) }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.login("user@example.com", "password")

            assertTrue(result.isSuccess)
            val response = result.getOrThrow()
            assertEquals("tok_123", response.accessToken)
            assertEquals("indr_123", response.refreshToken)
        }

    @Test
    fun registerSendsPostToAuthRegister() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(authResponseJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.register("Test User", "user@example.com", "password")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/auth/register", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun logoutSendsPostToAuthLogout() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveRefreshToken("indr_123")
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedPath = request.url.encodedPath
                    respond("", HttpStatusCode.NoContent, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            apiClient.logout()

            assertEquals("/api/v1/auth/logout", capturedPath)
        }

    @Test
    fun forgotPasswordSendsPost() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond("""{"message":"Reset link sent"}""", HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.forgotPassword("user@example.com")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/auth/password/forgot", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun resetPasswordSendsPost() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(authResponseJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.resetPassword("reset-token", "new-password")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/auth/password/reset", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun resendVerificationSendsPost() =
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
            apiClient.resendVerification()

            assertEquals("/api/v1/auth/email/resend", capturedPath)
        }

    @Test
    fun getOAuthProvidersReturnsProviders() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val engine =
                MockEngine { request ->
                    respond(oauthProvidersJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getOAuthProviders()

            assertTrue(result.isSuccess)
            val response = result.getOrThrow()
            assertNotNull(response.providers)
            assertTrue(response.providers.isNotEmpty())
            assertFalse(response.signupsEnabled)
            assertFalse(response.setupRequired)
        }

    private fun authResponseJson() =
        """
        {
            "id": "usr_01ABC",
            "object": "user",
            "email": "user@example.com",
            "display_name": "Test User",
            "email_verified": true,
            "onboarding_completed": true,
            "access_token": "tok_123",
            "refresh_token": "indr_123",
            "expires_at": $FAR_FUTURE_EXPIRY
        }
        """.trimIndent()

    private fun oauthProvidersJson() =
        """
        {"providers": [{"id": "google", "name": "Google", "enabled": true}], "signups_enabled": false, "setup_required": false}
        """.trimIndent()

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
