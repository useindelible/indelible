package app.indelible.core.network

import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

class RefreshFailureTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    private suspend fun requestMe(transport: AuthenticatedApiTransport) =
        transport.directAuthenticatedRequest { client, baseUrl, token ->
            client.get("$baseUrl/api/v1/me") { header("Authorization", "Bearer $token") }
        }

    private suspend fun expiredTokenStorage(): InMemoryTokenStorage =
        InMemoryTokenStorage().apply {
            saveToken("old")
            saveExpiresAt(0L)
            saveRefreshToken("refresh-old")
        }

    @Test
    fun refreshNetworkFailureKeepsCredentialsAndSurfacesTheCause() =
        runTest {
            val tokenStorage = expiredTokenStorage()
            var onUnauthorizedCalls = 0

            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/refresh" -> error("connection reset")
                        else -> respond("", HttpStatusCode.NotFound)
                    }
                }

            val transport =
                AuthenticatedApiTransport(
                    tokenStorage,
                    onUnauthorized = { onUnauthorizedCalls++ },
                    engine = engine,
                )

            val result = requestMe(transport)

            assertTrue(result.isFailure)
            val exception = result.exceptionOrNull()
            assertFalse(exception is ApiException && exception.statusCode == 401)
            assertEquals("refresh-old", tokenStorage.getRefreshToken())
            assertEquals(0, onUnauthorizedCalls)
        }

    @Test
    fun refresh500KeepsCredentialsAndSurfaces500() =
        runTest {
            val tokenStorage = expiredTokenStorage()
            var onUnauthorizedCalls = 0

            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/refresh" ->
                            respond(
                                content = """{"error":"server_error","message":"boom"}""",
                                status = HttpStatusCode.InternalServerError,
                                headers = jsonHeaders,
                            )
                        else -> respond("", HttpStatusCode.NotFound)
                    }
                }

            val transport =
                AuthenticatedApiTransport(
                    tokenStorage,
                    onUnauthorized = { onUnauthorizedCalls++ },
                    engine = engine,
                )

            val result = requestMe(transport)

            assertTrue(result.isFailure)
            val exception = result.exceptionOrNull()
            assertIs<ApiException>(exception)
            assertEquals(500, exception.statusCode)
            assertEquals("old", tokenStorage.getToken())
            assertEquals("refresh-old", tokenStorage.getRefreshToken())
            assertEquals(0, onUnauthorizedCalls)
        }

    @Test
    fun refresh401ClearsTheSession() =
        runTest {
            val tokenStorage = expiredTokenStorage()
            var onUnauthorizedCalls = 0

            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/refresh" ->
                            respond(
                                content = """{"error":"unauthorized","message":"revoked"}""",
                                status = HttpStatusCode.Unauthorized,
                                headers = jsonHeaders,
                            )
                        else -> respond("", HttpStatusCode.NotFound)
                    }
                }

            val transport =
                AuthenticatedApiTransport(
                    tokenStorage,
                    onUnauthorized = { onUnauthorizedCalls++ },
                    engine = engine,
                )

            val result = requestMe(transport)

            assertTrue(result.isFailure)
            val exception = result.exceptionOrNull()
            assertIs<ApiException>(exception)
            assertEquals(401, exception.statusCode)
            assertNull(tokenStorage.getToken())
            assertEquals(1, onUnauthorizedCalls)
        }
}
