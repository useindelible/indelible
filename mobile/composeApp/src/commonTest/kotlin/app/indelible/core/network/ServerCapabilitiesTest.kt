package app.indelible.core.network

import app.indelible.core.offline.Session
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.request.HttpRequestData
import io.ktor.client.request.HttpResponseData
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import okio.IOException
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

private const val ORIGIN = "https://books.example"
private const val CAPABILITIES = "/api/v1/server/capabilities"
private const val HEALTH = "/api/health"
private const val PAGE = "<!doctype html><html><body></body></html>"
private const val FIVE_MINUTES_MS = 5 * 60 * 1000L

class ServerCapabilitiesTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")
    private val htmlHeaders = headersOf(HttpHeaders.ContentType, "text/html")
    private val session = Session(epoch = 0, origin = ORIGIN, scope = "$ORIGIN|usr_1")
    private val listed = """{"capabilities":["epub_chapter_list"]}"""

    private var now = 0L

    private class Fixture(
        val capabilities: ServerCapabilities,
        val sent: MutableList<HttpRequestData>,
    )

    private fun fixture(handler: suspend MockRequestHandleScope.(HttpRequestData) -> HttpResponseData): Fixture {
        val sent = mutableListOf<HttpRequestData>()
        val engine =
            MockEngine { request ->
                sent += request
                handler(request)
            }
        val transport = AuthenticatedApiTransport(InMemoryTokenStorage(), engine = engine)
        return Fixture(ServerCapabilities(transport, clock = { now }), sent)
    }

    @Test
    fun supportedCapabilityIsTrue() =
        runTest {
            val f = fixture { respond(listed, HttpStatusCode.OK, jsonHeaders) }

            assertEquals(true, f.capabilities.supports(session, EPUB_CHAPTER_LIST))
            val request = f.sent.single()
            assertEquals("books.example", request.url.host)
            assertEquals(CAPABILITIES, request.url.encodedPath)
            assertNull(request.headers[HttpHeaders.Authorization])
        }

    @Test
    fun missingCapabilityIsFalse() =
        runTest {
            val f = fixture { respond("""{"capabilities":[]}""", HttpStatusCode.OK, jsonHeaders) }

            assertEquals(false, f.capabilities.supports(session, EPUB_CHAPTER_LIST))
        }

    @Test
    fun endpointAbsentIsFalse() =
        runTest {
            val f = fixture { respond("", HttpStatusCode.NotFound) }

            assertEquals(false, f.capabilities.supports(session, EPUB_CHAPTER_LIST))
        }

    @Test
    fun oldServerServingTheWebAppIsFalse() =
        runTest {
            val f =
                fixture { request ->
                    if (request.url.encodedPath == HEALTH) {
                        respond("""{"status":"healthy"}""", HttpStatusCode.OK, jsonHeaders)
                    } else {
                        respond(PAGE, HttpStatusCode.OK, htmlHeaders)
                    }
                }

            assertEquals(false, f.capabilities.supports(session, EPUB_CHAPTER_LIST))
        }

    @Test
    fun aLoginPageInFrontOfTheServerIsUnknown() =
        runTest {
            val f = fixture { respond(PAGE, HttpStatusCode.OK, htmlHeaders) }

            assertNull(f.capabilities.supports(session, EPUB_CHAPTER_LIST))
            assertNull(f.capabilities.supports(session, EPUB_CHAPTER_LIST))
            assertEquals(listOf(CAPABILITIES, HEALTH, CAPABILITIES, HEALTH), f.sent.map { it.url.encodedPath })
        }

    @Test
    fun jsonWithoutCapabilitiesIsFalse() =
        runTest {
            val f = fixture { respond("""{"status":"ok"}""", HttpStatusCode.OK, jsonHeaders) }

            assertEquals(false, f.capabilities.supports(session, EPUB_CHAPTER_LIST))
        }

    @Test
    fun networkFailureIsUnknown() =
        runTest {
            var calls = 0
            val f =
                fixture {
                    calls++
                    if (calls == 1) throw IOException("unreachable")
                    respond("", HttpStatusCode.ServiceUnavailable)
                }

            assertNull(f.capabilities.supports(session, EPUB_CHAPTER_LIST))
            assertNull(f.capabilities.supports(session, EPUB_CHAPTER_LIST))
            assertEquals(2, f.sent.size)
        }

    @Test
    fun supportIsCachedForTheSession() =
        runTest {
            val f = fixture { respond(listed, HttpStatusCode.OK, jsonHeaders) }

            assertEquals(true, f.capabilities.supports(session, EPUB_CHAPTER_LIST))
            now = FIVE_MINUTES_MS * 12
            assertEquals(true, f.capabilities.supports(session, EPUB_CHAPTER_LIST))
            assertEquals(1, f.sent.size)
        }

    @Test
    fun negativeResultExpiresAfterFiveMinutes() =
        runTest {
            var status = HttpStatusCode.NotFound
            val f = fixture { respond(listed, status, jsonHeaders) }

            assertEquals(false, f.capabilities.supports(session, EPUB_CHAPTER_LIST))
            status = HttpStatusCode.OK
            now = FIVE_MINUTES_MS - 1
            assertEquals(false, f.capabilities.supports(session, EPUB_CHAPTER_LIST))
            assertEquals(1, f.sent.size)
            now = FIVE_MINUTES_MS
            assertEquals(true, f.capabilities.supports(session, EPUB_CHAPTER_LIST))
            assertEquals(2, f.sent.size)
        }

    @Test
    fun sessionTransitionClearsTheCache() =
        runTest {
            var body = listed
            val f = fixture { respond(body, HttpStatusCode.OK, jsonHeaders) }
            val next = Session(epoch = 1, origin = session.origin, scope = session.scope)

            assertEquals(true, f.capabilities.supports(session, EPUB_CHAPTER_LIST))
            body = """{"capabilities":[]}"""
            assertEquals(false, f.capabilities.supports(next, EPUB_CHAPTER_LIST))
            assertEquals(2, f.sent.size)
        }
}
