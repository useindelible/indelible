package app.indelible.core.offline

import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.request.HttpRequestData
import io.ktor.client.request.HttpResponseData
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

private const val SCOPE = "http://localhost:38473|usr_1"
private const val DOCUMENT_ID = "doc_01"
private const val HIGHLIGHT_ID = "hlt_01"
private const val CLIENT_ID = "cli_test"
private val SESSION = Session(epoch = 0, origin = "http://localhost:38473", scope = SCOPE)

class ApiOutboxSenderTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    private suspend fun boundRegistry(): SessionRegistry = SessionRegistry().apply { publish(SessionState(0, SESSION)) }

    private class Capture {
        val methods = mutableListOf<HttpMethod>()
        val paths = mutableListOf<String>()
        val bodies = mutableListOf<String?>()
    }

    private suspend fun senderWith(
        capture: Capture = Capture(),
        handler: MockRequestHandleScope.(HttpRequestData) -> HttpResponseData,
    ): ApiOutboxSender {
        val tokenStorage = InMemoryTokenStorage()
        tokenStorage.saveToken("test-token")
        tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
        val engine =
            MockEngine { request ->
                capture.methods += request.method
                capture.paths += request.url.encodedPath
                capture.bodies += (request.body as? TextContent)?.text
                handler(request)
            }
        return ApiOutboxSender(AuthenticatedApiTransport(tokenStorage, engine = engine, registry = boundRegistry()))
    }

    private fun row(
        kind: OutboxKind,
        payload: OutboxPayload,
        entityId: String = HIGHLIGHT_ID,
    ) = OutboxRow(
        seq = 1,
        scope = SCOPE,
        id = "obx_1",
        kind = kind,
        entityId = entityId,
        documentId = DOCUMENT_ID,
        payload = payload,
        createdAt = 0,
        attempts = 0,
        lastAttemptAt = null,
        nextAttemptAt = 0,
        state = OutboxState.PENDING,
        lastError = null,
    )

    private fun noteRow() = row(OutboxKind.DOCUMENT_NOTE, OutboxPayload.DocumentNote("b", null))

    private fun readingEventRow(originSeq: Long) =
        row(
            OutboxKind.READING_EVENT,
            OutboxPayload.ReadingEvent(
                eventId = "rev_0$originSeq",
                originSeq = originSeq,
                kind = "progress",
                progressBasisPoints = 4237,
                cause = "scroll",
                sessionId = "ses_1",
                attempt = 0,
                positionJson = null,
                assetKind = "epub",
                activeMs = 1_000,
                recordedAtEpochMs = 1_767_225_600_000L,
            ),
            entityId = DOCUMENT_ID,
        )

    @Test
    fun readingEventBatchPostsClientIdAndOriginSeqs() =
        runTest {
            val capture = Capture()
            val accepted = """{"accepted":2,"replayed":0}"""
            val sender = senderWith(capture) { respond(accepted, HttpStatusCode.Accepted) }

            val outcome = sender.send(SESSION, CLIENT_ID, listOf(readingEventRow(1), readingEventRow(2)))

            assertEquals(SendOutcome.Success, outcome)
            assertEquals(HttpMethod.Post, capture.methods.single())
            assertEquals("/api/v1/documents/doc_01/reading-events", capture.paths.single())
            val body = assertNotNull(capture.bodies.single())
            assertTrue(body.contains("\"client_id\":\"$CLIENT_ID\""), body)
            assertTrue(body.contains("\"origin_seq\":1"), body)
            assertTrue(body.contains("\"origin_seq\":2"), body)
            assertTrue(body.contains("\"recorded_at\":\"2026-01-01T00:00:00Z\""), body)
            assertTrue(body.contains("\"progress_basis_points\":4237"), body)
        }

    @Test
    fun highlightCreateMaps201ToSuccessAnd200ToReplaySuccess() =
        runTest {
            val payload =
                OutboxPayload.HighlightCreate(
                    highlightId = HIGHLIGHT_ID,
                    color = "yellow",
                    textContent = "quoted",
                    locatorJson = "{\"type\":\"html\",\"start_offset\":10,\"end_offset\":50}",
                    sourceLocatorJson = null,
                )
            val capture = Capture()
            val created = senderWith(capture) { respond("{}", HttpStatusCode.Created, jsonHeaders) }
            assertEquals(
                SendOutcome.Success,
                created.send(SESSION, CLIENT_ID, listOf(row(OutboxKind.HIGHLIGHT_CREATE, payload))),
            )
            assertEquals(HttpMethod.Post, capture.methods.single())
            assertEquals("/api/v1/documents/doc_01/highlights", capture.paths.single())
            assertTrue(assertNotNull(capture.bodies.single()).contains("\"id\":\"$HIGHLIGHT_ID\""))

            val replayed = senderWith { respond("{}", HttpStatusCode.OK, jsonHeaders) }
            assertEquals(
                SendOutcome.ReplaySuccess,
                replayed.send(SESSION, CLIENT_ID, listOf(row(OutboxKind.HIGHLIGHT_CREATE, payload))),
            )
        }

    @Test
    fun highlightColorPatchesHighlight() =
        runTest {
            val capture = Capture()
            val sender = senderWith(capture) { respond("{}", HttpStatusCode.OK, jsonHeaders) }

            val payload = OutboxPayload.HighlightColor(HIGHLIGHT_ID, "blue")
            assertEquals(
                SendOutcome.Success,
                sender.send(SESSION, CLIENT_ID, listOf(row(OutboxKind.HIGHLIGHT_COLOR, payload))),
            )
            assertEquals(HttpMethod.Patch, capture.methods.single())
            assertEquals("/api/v1/highlights/$HIGHLIGHT_ID", capture.paths.single())
            assertTrue(assertNotNull(capture.bodies.single()).contains("\"color\":\"blue\""))
        }

    @Test
    fun highlightNotePutsBodyAndDeletesWhenNull() =
        runTest {
            val putCapture = Capture()
            val put = senderWith(putCapture) { respond("{}", HttpStatusCode.OK, jsonHeaders) }
            val withBody = OutboxPayload.HighlightNote(HIGHLIGHT_ID, "note text")
            assertEquals(
                SendOutcome.Success,
                put.send(SESSION, CLIENT_ID, listOf(row(OutboxKind.HIGHLIGHT_NOTE, withBody))),
            )
            assertEquals(HttpMethod.Put, putCapture.methods.single())
            assertEquals("/api/v1/highlights/$HIGHLIGHT_ID/note", putCapture.paths.single())

            val deleteCapture = Capture()
            val delete = senderWith(deleteCapture) { respond("", HttpStatusCode.NotFound) }
            val cleared = OutboxPayload.HighlightNote(HIGHLIGHT_ID, null)
            assertEquals(
                SendOutcome.ReplaySuccess,
                delete.send(SESSION, CLIENT_ID, listOf(row(OutboxKind.HIGHLIGHT_NOTE, cleared))),
            )
            assertEquals(HttpMethod.Delete, deleteCapture.methods.single())
            assertEquals("/api/v1/highlights/$HIGHLIGHT_ID/note", deleteCapture.paths.single())
        }

    @Test
    fun highlightTagsPutsTagList() =
        runTest {
            val capture = Capture()
            val sender = senderWith(capture) { respond("{}", HttpStatusCode.OK, jsonHeaders) }

            val payload = OutboxPayload.HighlightTags(HIGHLIGHT_ID, listOf("a", "b"))
            assertEquals(
                SendOutcome.Success,
                sender.send(SESSION, CLIENT_ID, listOf(row(OutboxKind.HIGHLIGHT_TAGS, payload))),
            )
            assertEquals(HttpMethod.Put, capture.methods.single())
            assertEquals("/api/v1/highlights/$HIGHLIGHT_ID/tags", capture.paths.single())
            assertTrue(assertNotNull(capture.bodies.single()).contains("\"tags\":[\"a\",\"b\"]"))
        }

    @Test
    fun highlightDeleteMaps404ToReplaySuccess() =
        runTest {
            val capture = Capture()
            val sender = senderWith(capture) { respond("", HttpStatusCode.NotFound) }

            val payload = OutboxPayload.HighlightDelete(HIGHLIGHT_ID)
            assertEquals(
                SendOutcome.ReplaySuccess,
                sender.send(SESSION, CLIENT_ID, listOf(row(OutboxKind.HIGHLIGHT_DELETE, payload))),
            )
            assertEquals(HttpMethod.Delete, capture.methods.single())
            assertEquals("/api/v1/highlights/$HIGHLIGHT_ID", capture.paths.single())
        }

    @Test
    fun documentNotePutsDocumentNote() =
        runTest {
            val capture = Capture()
            val sender = senderWith(capture) { respond("{}", HttpStatusCode.OK, jsonHeaders) }

            val payload = OutboxPayload.DocumentNote("note body", null)
            assertEquals(
                SendOutcome.Success,
                sender.send(SESSION, CLIENT_ID, listOf(row(OutboxKind.DOCUMENT_NOTE, payload, entityId = DOCUMENT_ID))),
            )
            assertEquals(HttpMethod.Put, capture.methods.single())
            assertEquals("/api/v1/documents/doc_01/note", capture.paths.single())
        }

    @Test
    fun unauthorizedThenSuccessRefreshesAndSucceeds() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("stale-token")
            tokenStorage.saveRefreshToken("refresh-old")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            val authorizations = mutableListOf<String?>()
            var noteCalls = 0
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/refresh" ->
                            respond(refreshJson, HttpStatusCode.OK, jsonHeaders)
                        else -> {
                            noteCalls++
                            authorizations += request.headers[HttpHeaders.Authorization]
                            if (noteCalls == 1) {
                                respond("nope", HttpStatusCode.Unauthorized)
                            } else {
                                respond("{}", HttpStatusCode.OK, jsonHeaders)
                            }
                        }
                    }
                }
            val sender =
                ApiOutboxSender(AuthenticatedApiTransport(tokenStorage, engine = engine, registry = boundRegistry()))

            val outcome = sender.send(SESSION, CLIENT_ID, listOf(noteRow()))

            assertEquals(SendOutcome.Success, outcome)
            assertEquals(listOf<String?>("Bearer stale-token", "Bearer access-new"), authorizations)
        }

    @Test
    fun unauthorizedTwiceYieldsHttp401NotTransport() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("stale-token")
            tokenStorage.saveRefreshToken("refresh-old")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/refresh" -> respond(refreshJson, HttpStatusCode.OK, jsonHeaders)
                        else -> respond("nope", HttpStatusCode.Unauthorized)
                    }
                }
            val sender =
                ApiOutboxSender(AuthenticatedApiTransport(tokenStorage, engine = engine, registry = boundRegistry()))

            val outcome = sender.send(SESSION, CLIENT_ID, listOf(noteRow()))

            val http = assertIs<SendOutcome.Http>(outcome)
            assertEquals(401, http.status)
            assertEquals(Classification.AuthBlocked, classify(outcome, attempts = 0, now = 0) { false })
        }

    @Test
    fun transportFailureYieldsTransportOutcome() =
        runTest {
            val sender = senderWith { throw FakeNetworkFailure() }

            val outcome = sender.send(SESSION, CLIENT_ID, listOf(noteRow()))

            assertIs<SendOutcome.Transport>(outcome)
        }

    private val refreshJson =
        """{"access_token":"access-new","refresh_token":"refresh-new","expires_at":$FAR_FUTURE_EXPIRY}"""

    private companion object {
        const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
