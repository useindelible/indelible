package app.indelible.core.network

import app.indelible.core.offline.Session
import app.indelible.core.offline.SessionRegistry
import app.indelible.core.offline.SessionState
import app.indelible.core.offline.StaleSessionException
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.request.HttpRequestData
import io.ktor.client.request.HttpResponseData
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.async
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertSame
import kotlin.test.assertTrue

private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
private const val PING = "/api/v1/ping"
private const val REFRESH = "/api/v1/auth/refresh"

class BoundTransportTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")
    private val refreshJson =
        """{"access_token":"access-new","refresh_token":"refresh-new","expires_at":$FAR_FUTURE_EXPIRY}"""

    private class Fixture(
        val registry: SessionRegistry,
        val tokenStorage: InMemoryTokenStorage,
        val transport: AuthenticatedApiTransport,
        val hosts: MutableList<String>,
        val paths: MutableList<String>,
    )

    private suspend fun fixture(
        expired: Boolean = false,
        onUnauthorized: suspend (Long) -> Unit = {},
        handler: suspend MockRequestHandleScope.(HttpRequestData) -> HttpResponseData,
    ): Fixture {
        val registry = SessionRegistry()
        val tokenStorage = InMemoryTokenStorage()
        tokenStorage.saveServerUrl("http://stored.test")
        tokenStorage.saveToken("old")
        tokenStorage.saveRefreshToken("refresh-old")
        tokenStorage.saveExpiresAt(if (expired) 0L else FAR_FUTURE_EXPIRY)
        val hosts = mutableListOf<String>()
        val paths = mutableListOf<String>()
        val engine =
            MockEngine { request ->
                hosts += request.url.host
                paths += request.url.encodedPath
                handler(request)
            }
        val transport = AuthenticatedApiTransport(tokenStorage, onUnauthorized, engine, registry)
        return Fixture(registry, tokenStorage, transport, hosts, paths)
    }

    private suspend fun signedIn(
        registry: SessionRegistry,
        origin: String = "http://a.test",
    ): Session {
        val session = Session(epoch = 0, origin = origin, scope = "$origin|u1")
        registry.publish(SessionState(0, session))
        return session
    }

    private suspend fun ping(
        transport: AuthenticatedApiTransport,
        session: Session,
    ): RawApiResponse =
        transport.rawAuthenticatedRequest(session) { client, baseUrl, token ->
            client.get("$baseUrl$PING") { header(HttpHeaders.Authorization, "Bearer $token") }
        }

    @Test
    fun boundRequestUsesTheSessionOriginNotStorage() =
        runTest {
            val f = fixture { respond("", HttpStatusCode.OK) }
            val session = signedIn(f.registry)

            val response = ping(f.transport, session)

            assertEquals(200, response.status)
            assertEquals(listOf("a.test"), f.hosts)
        }

    @Test
    fun boundRequestIsStaleBeforeDispatchWhenTheSessionChanged() =
        runTest {
            val f = fixture { respond("", HttpStatusCode.OK) }
            val session = signedIn(f.registry)
            f.registry.publish(SessionState(1, Session(1, "http://a.test", "http://a.test|u1")))

            assertFailsWith<StaleSessionException> { ping(f.transport, session) }
            assertTrue(f.paths.isEmpty())
        }

    @Test
    fun requestAlreadySentReturnsItsRealOutcomeAfterTheSessionChanges() =
        runTest {
            lateinit var registry: SessionRegistry
            val f =
                fixture {
                    registry.publish(SessionState(1, null))
                    respond("", HttpStatusCode.OK)
                }
            registry = f.registry
            val session = signedIn(f.registry)

            val response = ping(f.transport, session)

            assertEquals(200, response.status)
        }

    @Test
    fun postRefreshRetryIsStaleWhenTheSessionChangedDuringRefresh() =
        runTest {
            lateinit var registry: SessionRegistry
            val f =
                fixture { request ->
                    when (request.url.encodedPath) {
                        REFRESH -> {
                            registry.publish(SessionState(1, null))
                            respond(refreshJson, HttpStatusCode.OK, jsonHeaders)
                        }
                        else -> respond("nope", HttpStatusCode.Unauthorized)
                    }
                }
            registry = f.registry
            val session = signedIn(f.registry)

            assertFailsWith<StaleSessionException> { ping(f.transport, session) }

            assertEquals(listOf(PING, REFRESH), f.paths)
            assertEquals("old", f.tokenStorage.getToken())
        }

    @Test
    fun boundRequestRefreshesAgainstTheSessionOriginNotStorage() =
        runTest {
            val f =
                fixture(expired = true) { request ->
                    when (request.url.encodedPath) {
                        REFRESH -> respond(refreshJson, HttpStatusCode.OK, jsonHeaders)
                        else -> respond("", HttpStatusCode.OK)
                    }
                }
            val session = signedIn(f.registry)

            assertEquals(200, ping(f.transport, session).status)

            assertEquals(listOf(REFRESH, PING), f.paths)
            assertEquals(listOf("a.test", "a.test"), f.hosts)
        }

    @Test
    fun staleBoundRequestNeverStartsARefresh() =
        runTest {
            val f = fixture(expired = true) { respond("", HttpStatusCode.OK) }
            val session = signedIn(f.registry)
            f.registry.publish(SessionState(1, Session(1, "http://a.test", "http://a.test|u1")))

            assertFailsWith<StaleSessionException> { ping(f.transport, session) }
            assertTrue(f.paths.isEmpty())
        }

    @Test
    fun coldStartRefreshBindsToEpochZeroAndSucceeds() =
        runTest {
            val f =
                fixture(expired = true) { request ->
                    when (request.url.encodedPath) {
                        REFRESH -> respond(refreshJson, HttpStatusCode.OK, jsonHeaders)
                        else -> respond("", HttpStatusCode.OK)
                    }
                }

            val result =
                f.transport.directAuthenticatedRequest { client, baseUrl, token ->
                    client.get("$baseUrl$PING") { header(HttpHeaders.Authorization, "Bearer $token") }.status.value
                }

            assertEquals(200, result.getOrThrow())
            assertEquals("access-new", f.tokenStorage.getToken())
            assertEquals(0L, f.registry.current.value.epoch)
            assertNull(f.registry.current.value.session)
        }

    @Test
    fun refreshAnswered401InvalidatesAndReportsTheEpochWithoutClearingStorage() =
        runTest {
            val reported = mutableListOf<Long>()
            val f =
                fixture(expired = true, onUnauthorized = { reported += it }) { request ->
                    when (request.url.encodedPath) {
                        REFRESH -> respond("revoked", HttpStatusCode.Unauthorized)
                        else -> respond("", HttpStatusCode.OK)
                    }
                }
            signedIn(f.registry)

            val result =
                f.transport.directAuthenticatedRequest { client, baseUrl, _ -> client.get("$baseUrl$PING") }

            assertTrue(result.isFailure)
            assertEquals(listOf(0L), reported)
            assertNull(f.registry.current.value.session)
            assertEquals("old", f.tokenStorage.getToken())
        }

    @Test
    fun retryAfter401NeverBorrowsTheNextSessionsToken() =
        runTest {
            lateinit var f: Fixture
            f =
                fixture { request ->
                    when {
                        request.url.encodedPath == PING && f.paths.size == 1 -> {
                            f.tokenStorage.saveToken("token-b")
                            f.registry.publish(SessionState(1, Session(1, "http://b.test", "http://b.test|u2")))
                            respond("expired", HttpStatusCode.Unauthorized)
                        }
                        else -> respond("", HttpStatusCode.OK)
                    }
                }
            signedIn(f.registry)

            val result =
                f.transport.directAuthenticatedRequest { client, baseUrl, token ->
                    val response = client.get("$baseUrl$PING") { header(HttpHeaders.Authorization, "Bearer $token") }
                    if (response.status == HttpStatusCode.Unauthorized) throw ApiException(401, "expired")
                    response.status.value
                }

            assertTrue(result.exceptionOrNull() is StaleSessionException)
            assertEquals(listOf(PING), f.paths)
        }

    @Test
    fun refresh401ThatLandsAfterATransitionDoesNotSignOutTheNewSession() =
        runTest {
            lateinit var f: Fixture
            val reported = mutableListOf<Long>()
            val b = Session(1, "http://b.test", "http://b.test|u2")
            f =
                fixture(expired = true, onUnauthorized = { reported += it }) { request ->
                    when (request.url.encodedPath) {
                        REFRESH -> {
                            f.registry.publish(SessionState(1, b))
                            respond("revoked", HttpStatusCode.Unauthorized)
                        }
                        else -> respond("", HttpStatusCode.OK)
                    }
                }
            signedIn(f.registry)

            val result =
                f.transport.directAuthenticatedRequest { client, baseUrl, _ -> client.get("$baseUrl$PING") }

            assertTrue(result.isFailure)
            assertTrue(reported.isEmpty())
            assertSame(b, f.registry.current.value.session)
        }

    @Test
    fun requestQueuedBehindARefreshIsStaleAfterATransitionAndReadsNoCredentials() =
        runTest {
            lateinit var f: Fixture
            val releaseRefresh = CompletableDeferred<Unit>()
            val reported = mutableListOf<Long>()
            f =
                fixture(expired = true, onUnauthorized = { reported += it }) { request ->
                    when (request.url.encodedPath) {
                        REFRESH -> {
                            releaseRefresh.await()
                            respond(refreshJson, HttpStatusCode.OK, jsonHeaders)
                        }
                        else -> respond("", HttpStatusCode.OK)
                    }
                }
            signedIn(f.registry)
            val ping: suspend (HttpClient, String, String) -> Int =
                { client, baseUrl, _ -> client.get("$baseUrl$PING").status.value }
            val first = async { f.transport.directAuthenticatedRequest(block = ping) }
            val queued = async { f.transport.directAuthenticatedRequest(block = ping) }
            runCurrent()

            val b = Session(1, "http://b.test", "http://b.test|u2")
            f.tokenStorage.saveToken("token-b")
            f.tokenStorage.saveRefreshToken("refresh-b")
            f.registry.publish(SessionState(1, b))
            releaseRefresh.complete(Unit)

            assertTrue(first.await().exceptionOrNull() is StaleSessionException)
            assertTrue(queued.await().exceptionOrNull() is StaleSessionException)
            assertEquals(listOf(REFRESH), f.paths)
            assertEquals("token-b", f.tokenStorage.getToken())
            assertTrue(reported.isEmpty())
            assertSame(b, f.registry.current.value.session)
        }

    @Test
    fun serverChangeDuringRefreshDiscardsTheRefreshResult() =
        runTest {
            lateinit var registry: SessionRegistry
            val reported = mutableListOf<Long>()
            val f =
                fixture(expired = true, onUnauthorized = { reported += it }) { request ->
                    when (request.url.encodedPath) {
                        REFRESH -> {
                            registry.publish(SessionState(1, null))
                            respond(refreshJson, HttpStatusCode.OK, jsonHeaders)
                        }
                        else -> respond("", HttpStatusCode.OK)
                    }
                }
            registry = f.registry

            val result =
                f.transport.directAuthenticatedRequest { client, baseUrl, _ -> client.get("$baseUrl$PING") }

            assertTrue(result.exceptionOrNull() is StaleSessionException)
            assertEquals("old", f.tokenStorage.getToken())
            assertTrue(reported.isEmpty())
        }
}
