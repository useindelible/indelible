package app.indelible.core.network

import app.indelible.core.platform.platformClientType
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class ApiClientTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun loginReturnsUserOnSuccess() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val engine =
                MockEngine {
                    respond(
                        content = authResponseJson(accessToken = "tok_123", refreshToken = "indr_123"),
                        status = HttpStatusCode.OK,
                        headers = jsonHeaders,
                    )
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.login("user@example.com", "password")

            assertTrue(result.isSuccess)
            val response = result.getOrThrow()
            assertEquals("tok_123", response.accessToken)
            assertEquals("indr_123", response.refreshToken)
            assertEquals("usr_01ABCDEF", response.id)
            assertEquals("Test", response.displayName)
        }

    @Test
    fun loginReturnsErrorOnFailure() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val engine =
                MockEngine {
                    respond(
                        content = """{"error":"invalid_credentials","message":"Bad password"}""",
                        status = HttpStatusCode.Unauthorized,
                        headers = jsonHeaders,
                    )
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.login("user@example.com", "wrong")

            assertTrue(result.isFailure)
            val exception = result.exceptionOrNull()
            assertIs<ApiException>(exception)
            assertEquals(401, exception.statusCode)
            assertEquals("Bad password", exception.message)
        }

    @Test
    fun getSessionIncludesAuthHeader() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("my-bearer-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)

            var capturedAuthHeader: String? = null
            val engine =
                MockEngine { request ->
                    capturedAuthHeader = request.headers["Authorization"]
                    respond(
                        content = sessionProfileJson(),
                        status = HttpStatusCode.OK,
                        headers = jsonHeaders,
                    )
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            apiClient.getSession()

            assertEquals("Bearer my-bearer-token", capturedAuthHeader)
        }

    @Test
    fun getSessionFailsWithNoTokens() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val engine = MockEngine { respond("", HttpStatusCode.OK) }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getSession()

            assertTrue(result.isFailure)
            val exception = result.exceptionOrNull()
            assertIs<ApiException>(exception)
            assertEquals(401, exception.statusCode)
        }

    @Test
    fun logoutSendsRefreshTokenInBody() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveRefreshToken("refresh-logout")
            var requestBody: String? = null

            val engine =
                MockEngine { request ->
                    if (request.url.encodedPath == "/api/v1/auth/logout") {
                        requestBody = (request.body as TextContent).text
                    }
                    respond("", HttpStatusCode.NoContent, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.logout()

            assertTrue(result.isSuccess)
            assertNotNull(requestBody)
            assertTrue(requestBody.contains("refresh-logout"))
        }

    @Test
    fun logoutReturnsFailureWhenRevocationFails() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveRefreshToken("refresh-logout")
            val engine =
                MockEngine {
                    respond(
                        content = """{"error":"server_error","message":"boom"}""",
                        status = HttpStatusCode.InternalServerError,
                        headers = jsonHeaders,
                    )
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.logout()

            assertTrue(result.isFailure)
            assertIs<ApiException>(result.exceptionOrNull())
        }

    @Test
    fun registerReturnsUserOnSuccess() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val engine =
                MockEngine {
                    respond(
                        content =
                            authResponseJson(
                                accessToken = "reg-token",
                                refreshToken = "reg-refresh",
                                emailVerified = false,
                                onboardingCompleted = false,
                            ),
                        status = HttpStatusCode.OK,
                        headers = jsonHeaders,
                    )
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.register("New User", "new@example.com", "password123")

            assertTrue(result.isSuccess)
            val response = result.getOrThrow()
            assertEquals("reg-token", response.accessToken)
            assertEquals("reg-refresh", response.refreshToken)
            assertFalse(response.emailVerified)
        }

    @Test
    fun forgotPasswordSucceeds() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val engine =
                MockEngine {
                    respond(
                        content = """{"message":"Reset link sent"}""",
                        status = HttpStatusCode.OK,
                        headers = jsonHeaders,
                    )
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.forgotPassword("user@example.com")

            assertTrue(result.isSuccess)
        }

    @Test
    fun requestsIncludeClientTypeHeader() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            var capturedClientType: String? = null

            val engine =
                MockEngine { request ->
                    capturedClientType = request.headers["X-Client-Type"]
                    respond(
                        content = authResponseJson(accessToken = "tok", refreshToken = "indr_tok"),
                        status = HttpStatusCode.OK,
                        headers = jsonHeaders,
                    )
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            apiClient.login("u@e.com", "p")

            assertEquals(platformClientType(), capturedClientType)
        }

    @Test
    fun usesCustomServerUrl() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveServerUrl("  https://my.server.com/  ")

            var capturedUrl: String? = null
            val engine =
                MockEngine { request ->
                    capturedUrl = request.url.toString()
                    respond(
                        content = authResponseJson(accessToken = "tok", refreshToken = "indr_tok"),
                        status = HttpStatusCode.OK,
                        headers = jsonHeaders,
                    )
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            apiClient.login("u@e.com", "p")

            assertEquals("https://my.server.com/api/v1/auth/login", capturedUrl)
        }

    @Test
    fun usesDefaultServerUrlWhenNotConfigured() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()

            var capturedUrl: String? = null
            val engine =
                MockEngine { request ->
                    capturedUrl = request.url.toString()
                    respond(
                        content = authResponseJson(accessToken = "tok", refreshToken = "indr_tok"),
                        status = HttpStatusCode.OK,
                        headers = jsonHeaders,
                    )
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            apiClient.login("u@e.com", "p")

            assertNotNull(capturedUrl)
            assertTrue(capturedUrl.startsWith("${ApiClient.DEFAULT_SERVER_URL}/"))
        }

    private fun authResponseJson(
        accessToken: String,
        refreshToken: String,
        emailVerified: Boolean = true,
        onboardingCompleted: Boolean = true,
        expiresAt: Long = FAR_FUTURE_EXPIRY,
    ) = """
        {
            "id": "usr_01ABCDEF",
            "object": "user",
            "email": "user@example.com",
            "display_name": "Test",
            "email_verified": $emailVerified,
            "onboarding_completed": $onboardingCompleted,
            "access_token": "$accessToken",
            "refresh_token": "$refreshToken",
            "expires_at": $expiresAt
        }
        """.trimIndent()

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
