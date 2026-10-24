package app.indelible.core.network

import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class ApiClientTokenRefreshTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun refreshOnExpiredTokenPersistsRotatedToken() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("expired-token")
            tokenStorage.saveRefreshToken("refresh-old")
            tokenStorage.saveExpiresAt(1L)

            val authHeaders = mutableListOf<String?>()
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/refresh" ->
                            respond(
                                content = refreshResponseJson("access-new", "refresh-new", FAR_FUTURE_EXPIRY),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        "/api/v1/me" -> {
                            authHeaders += request.headers["Authorization"]
                            respond(
                                content = sessionProfileJson(),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        }
                        else -> respond("", HttpStatusCode.NotFound)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getSession()

            assertTrue(result.isSuccess)
            assertEquals(listOf<String?>("Bearer access-new"), authHeaders)
            assertEquals("access-new", tokenStorage.getToken())
            assertEquals("refresh-new", tokenStorage.getRefreshToken())
            assertEquals(FAR_FUTURE_EXPIRY, tokenStorage.getExpiresAt())
        }

    @Test
    fun retryOn401RefreshesAndRetriesOnce() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("still-valid-token")
            tokenStorage.saveRefreshToken("refresh-old")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)

            val authHeaders = mutableListOf<String?>()
            var sessionCalls = 0
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/me" -> {
                            sessionCalls++
                            authHeaders += request.headers["Authorization"]
                            if (sessionCalls == 1) {
                                respond(
                                    content = """{"error":"unauthorized"}""",
                                    status = HttpStatusCode.Unauthorized,
                                    headers = jsonHeaders,
                                )
                            } else {
                                respond(
                                    content = sessionProfileJson(),
                                    status = HttpStatusCode.OK,
                                    headers = jsonHeaders,
                                )
                            }
                        }
                        "/api/v1/auth/refresh" ->
                            respond(
                                content = refreshResponseJson("access-new", "refresh-new", FAR_FUTURE_EXPIRY),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        else -> respond("", HttpStatusCode.NotFound)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getSession()

            assertTrue(result.isSuccess)
            assertEquals(
                listOf<String?>("Bearer still-valid-token", "Bearer access-new"),
                authHeaders,
            )
            assertEquals("refresh-new", tokenStorage.getRefreshToken())
        }

    @Test
    fun concurrent401ResponsesShareOneRefresh() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("rejected-token")
            tokenStorage.saveRefreshToken("refresh-old")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)

            val bothRejectedRequestsStarted = CompletableDeferred<Unit>()
            var rejectedRequestCount = 0
            var refreshRequestCount = 0
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/me" -> {
                            if (request.headers["Authorization"] == "Bearer rejected-token") {
                                rejectedRequestCount++
                                if (rejectedRequestCount == 2) {
                                    bothRejectedRequestsStarted.complete(Unit)
                                }
                                bothRejectedRequestsStarted.await()
                                respond(
                                    content = """{"error":"unauthorized"}""",
                                    status = HttpStatusCode.Unauthorized,
                                    headers = jsonHeaders,
                                )
                            } else {
                                respond(
                                    content = sessionProfileJson(),
                                    status = HttpStatusCode.OK,
                                    headers = jsonHeaders,
                                )
                            }
                        }
                        "/api/v1/auth/refresh" -> {
                            refreshRequestCount++
                            respond(
                                content = refreshResponseJson("access-new", "refresh-new", FAR_FUTURE_EXPIRY),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        }
                        else -> respond("", HttpStatusCode.NotFound)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val results =
                listOf(
                    async { apiClient.getSession() },
                    async { apiClient.getSession() },
                ).awaitAll()

            assertTrue(results.all { it.isSuccess })
            assertEquals(2, rejectedRequestCount)
            assertEquals(1, refreshRequestCount)
        }

    @Test
    fun refreshRejectionReportsUnauthorizedAndLeavesStorageToTheSignOut() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("expired-token")
            tokenStorage.saveRefreshToken("refresh-old")
            tokenStorage.saveExpiresAt(1L)
            var callbackCalled = false

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

            val apiClient =
                ApiClient(
                    tokenStorage,
                    onUnauthorized = { callbackCalled = true },
                    engine = engine,
                )
            val result = apiClient.getSession()

            assertTrue(result.isFailure)
            assertTrue(callbackCalled)
            assertNotNull(tokenStorage.getRefreshToken())
        }

    private fun refreshResponseJson(
        accessToken: String,
        refreshToken: String,
        expiresAt: Long,
    ) = """
        {
            "access_token": "$accessToken",
            "refresh_token": "$refreshToken",
            "expires_at": $expiresAt
        }
        """.trimIndent()

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
