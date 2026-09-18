package app.indelible.core.network

import app.indelible.core.offline.Session
import app.indelible.core.offline.SessionRegistry
import app.indelible.core.offline.SessionState
import app.indelible.core.offline.StaleSessionException
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.request.HttpRequestData
import io.ktor.client.request.HttpResponseData
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.statement.bodyAsText
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue

private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
private const val PING = "/api/v1/ping"
private const val REFRESH = "/api/v1/auth/refresh"
private const val ASSET = "/api/v1/documents/doc_1/assets/pdf"

class SessionRequestTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")
    private val refreshJson =
        """{"access_token":"access-new","refresh_token":"refresh-new","expires_at":$FAR_FUTURE_EXPIRY}"""

    private class Sent(
        val host: String,
        val path: String,
        val authorization: String?,
        val body: String?,
    )

    private class Fixture(
        val registry: SessionRegistry,
        val tokenStorage: InMemoryTokenStorage,
        val transport: AuthenticatedApiTransport,
        val sent: MutableList<Sent>,
    ) {
        val paths: List<String> get() = sent.map { it.path }
    }

    private suspend fun fixture(
        expired: Boolean = false,
        handler: suspend MockRequestHandleScope.(HttpRequestData) -> HttpResponseData,
    ): Fixture {
        val registry = SessionRegistry()
        val tokenStorage = InMemoryTokenStorage()
        tokenStorage.saveServerUrl("http://stored.test")
        tokenStorage.saveToken("old")
        tokenStorage.saveRefreshToken("refresh-old")
        tokenStorage.saveExpiresAt(if (expired) 0L else FAR_FUTURE_EXPIRY)
        val sent = mutableListOf<Sent>()
        val engine =
            MockEngine { request ->
                sent +=
                    Sent(
                        host = request.url.host,
                        path = request.url.encodedPath,
                        authorization = request.headers[HttpHeaders.Authorization],
                        body = (request.body as? TextContent)?.text,
                    )
                handler(request)
            }
        val transport = AuthenticatedApiTransport(tokenStorage, engine = engine, registry = registry)
        return Fixture(registry, tokenStorage, transport, sent)
    }

    private suspend fun signedIn(registry: SessionRegistry): Session {
        val session = Session(epoch = 0, origin = "http://a.test", scope = "http://a.test|u1")
        registry.publish(SessionState(0, session))
        return session
    }

    private suspend fun fetch(
        transport: AuthenticatedApiTransport,
        session: Session,
        path: String = PING,
    ): String =
        transport.sessionRequest(session) { client, baseUrl, token ->
            val response = client.get("$baseUrl$path") { header(HttpHeaders.Authorization, "Bearer $token") }
            if (response.status == HttpStatusCode.Unauthorized) throw ApiException(401, "expired")
            response.bodyAsText()
        }

    @Test
    fun sessionRequestSendsToSessionOriginAndReturnsBody() =
        runTest {
            val f = fixture { respond("hello", HttpStatusCode.OK) }
            val session = signedIn(f.registry)

            assertEquals("hello", fetch(f.transport, session))
            assertEquals(listOf("a.test"), f.sent.map { it.host })
            assertEquals("Bearer old", f.sent.single().authorization)
        }

    @Test
    fun sessionRequestRefreshesAgainstSessionOriginOn401() =
        runTest {
            val f =
                fixture { request ->
                    when {
                        request.url.encodedPath == REFRESH -> respond(refreshJson, HttpStatusCode.OK, jsonHeaders)
                        request.headers[HttpHeaders.Authorization] == "Bearer old" ->
                            respond("expired", HttpStatusCode.Unauthorized)
                        else -> respond("hello", HttpStatusCode.OK)
                    }
                }
            val session = signedIn(f.registry)

            assertEquals("hello", fetch(f.transport, session))
            assertEquals(listOf(PING, REFRESH, PING), f.paths)
            assertEquals(listOf("a.test", "a.test", "a.test"), f.sent.map { it.host })
            assertEquals("Bearer access-new", f.sent.last().authorization)
        }

    @Test
    fun sessionRequestIsStaleBeforeSend() =
        runTest {
            val f = fixture { respond("hello", HttpStatusCode.OK) }
            val session = signedIn(f.registry)
            f.registry.publish(SessionState(1, Session(1, "http://a.test", "http://a.test|u1")))

            assertFailsWith<StaleSessionException> { fetch(f.transport, session) }
            assertTrue(f.sent.isEmpty())
        }

    @Test
    fun sessionRequestIsStaleBeforePostRefreshRetry() =
        runTest {
            lateinit var registry: SessionRegistry
            val f =
                fixture { request ->
                    when (request.url.encodedPath) {
                        REFRESH -> {
                            registry.invalidate(0)
                            respond(refreshJson, HttpStatusCode.OK, jsonHeaders)
                        }
                        else -> respond("expired", HttpStatusCode.Unauthorized)
                    }
                }
            registry = f.registry
            val session = signedIn(f.registry)

            assertFailsWith<StaleSessionException> { fetch(f.transport, session) }
            assertEquals(listOf(PING, REFRESH), f.paths)
        }

    @Test
    fun staleSessionNeverPostsNextAccountsRefreshTokenToOldOrigin() =
        runTest {
            val releaseRefresh = CompletableDeferred<Unit>()
            val f =
                fixture(expired = true) { request ->
                    when (request.url.encodedPath) {
                        REFRESH -> {
                            releaseRefresh.await()
                            respond(refreshJson, HttpStatusCode.OK, jsonHeaders)
                        }
                        else -> respond("hello", HttpStatusCode.OK)
                    }
                }
            val a = signedIn(f.registry)
            val first = async { runCatching { fetch(f.transport, a) } }
            val queued = async { runCatching { fetch(f.transport, a) } }
            runCurrent()

            f.tokenStorage.saveToken("token-b")
            f.tokenStorage.saveRefreshToken("refresh-b")
            f.registry.publish(SessionState(1, Session(1, "http://b.test", "http://b.test|u2")))
            releaseRefresh.complete(Unit)

            assertTrue(first.await().exceptionOrNull() is StaleSessionException)
            assertTrue(queued.await().exceptionOrNull() is StaleSessionException)
            assertEquals(listOf(REFRESH), f.paths)
            assertTrue(f.sent.none { it.body?.contains("refresh-b") == true })
            assertEquals("token-b", f.tokenStorage.getToken())
        }

    @Test
    fun rawRequestEmitsRequestSucceededOnlyOn2xx() =
        runTest {
            var status = HttpStatusCode.InternalServerError
            val f = fixture { respond("", status) }
            val session = signedIn(f.registry)
            var emitted = 0
            backgroundScope.launch { f.transport.requestSucceeded.collect { emitted++ } }
            runCurrent()

            f.transport.rawAuthenticatedRequest(session) { client, baseUrl, _ -> client.get("$baseUrl$PING") }
            runCurrent()
            assertEquals(0, emitted)

            status = HttpStatusCode.OK
            f.transport.rawAuthenticatedRequest(session) { client, baseUrl, _ -> client.get("$baseUrl$PING") }
            runCurrent()
            assertEquals(1, emitted)

            fetch(f.transport, session)
            runCurrent()
            assertEquals(1, emitted)
        }

    @Test
    fun crossHostRedirectDropsAuthorization() =
        runTest {
            val presigned = headersOf(HttpHeaders.Location, "http://s3.test/bucket/doc_1.pdf")
            val f =
                fixture { request ->
                    when (request.url.host) {
                        "a.test" -> respond("", HttpStatusCode.Found, presigned)
                        else -> respond("%PDF", HttpStatusCode.OK)
                    }
                }
            val session = signedIn(f.registry)

            assertEquals("%PDF", fetch(f.transport, session, ASSET))
            assertEquals(listOf("a.test", "s3.test"), f.sent.map { it.host })
            assertEquals("Bearer old", f.sent.first().authorization)
            assertNull(f.sent.last().authorization)
        }
}
