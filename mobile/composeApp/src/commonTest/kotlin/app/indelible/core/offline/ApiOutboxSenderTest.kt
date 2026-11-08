package app.indelible.core.offline

import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class ApiOutboxSenderTest {
    private fun noteRow() = senderRow(OutboxKind.DOCUMENT_NOTE, OutboxPayload.DocumentNote("b", null))

    private fun readingEventRow(originSeq: Long) =
        senderRow(
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
            entityId = SENDER_DOCUMENT,
        )

    @Test
    fun readingEventBatchPostsClientIdAndOriginSeqs() =
        runTest {
            val capture = SenderCapture()
            val accepted = """{"accepted":2,"replayed":0}"""
            val sender = senderWith(capture) { respond(accepted, HttpStatusCode.Accepted) }

            val outcome = sender.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(readingEventRow(1), readingEventRow(2)))

            assertEquals(SendOutcome.Success(), outcome)
            assertEquals(HttpMethod.Post, capture.methods.single())
            assertEquals("/api/v1/documents/doc_01/reading-events", capture.paths.single())
            val body = assertNotNull(capture.bodies.single())
            assertTrue(body.contains("\"client_id\":\"$SENDER_CLIENT_ID\""), body)
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
                    highlightId = SENDER_HIGHLIGHT,
                    color = "yellow",
                    textContent = "quoted",
                    locatorJson = "{\"type\":\"html\",\"start_offset\":10,\"end_offset\":50}",
                    sourceLocatorJson = null,
                )
            val capture = SenderCapture()
            val created = senderWith(capture) { respond("{}", HttpStatusCode.Created, senderJsonHeaders) }
            assertEquals(
                SendOutcome.Success(),
                created.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(senderRow(OutboxKind.HIGHLIGHT_CREATE, payload))),
            )
            assertEquals(HttpMethod.Post, capture.methods.single())
            assertEquals("/api/v1/documents/doc_01/highlights", capture.paths.single())
            assertTrue(assertNotNull(capture.bodies.single()).contains("\"id\":\"$SENDER_HIGHLIGHT\""))

            val replayed = senderWith { respond("{}", HttpStatusCode.OK, senderJsonHeaders) }
            val replayRow = senderRow(OutboxKind.HIGHLIGHT_CREATE, payload)
            assertEquals(SendOutcome.ReplaySuccess, replayed.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(replayRow)))
        }

    @Test
    fun highlightColorPatchesHighlight() =
        runTest {
            val capture = SenderCapture()
            val sender = senderWith(capture) { respond("{}", HttpStatusCode.OK, senderJsonHeaders) }

            val payload = OutboxPayload.HighlightColor(SENDER_HIGHLIGHT, "blue")
            assertEquals(
                SendOutcome.Success(),
                sender.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(senderRow(OutboxKind.HIGHLIGHT_COLOR, payload))),
            )
            assertEquals(HttpMethod.Patch, capture.methods.single())
            assertEquals("/api/v1/highlights/$SENDER_HIGHLIGHT", capture.paths.single())
            assertTrue(assertNotNull(capture.bodies.single()).contains("\"color\":\"blue\""))
        }

    @Test
    fun highlightNotePutsBodyAndDeletesWhenNull() =
        runTest {
            val putCapture = SenderCapture()
            val put = senderWith(putCapture) { respond("{}", HttpStatusCode.OK, senderJsonHeaders) }
            val withBody = OutboxPayload.HighlightNote(SENDER_HIGHLIGHT, "note text")
            assertEquals(
                SendOutcome.Success(),
                put.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(senderRow(OutboxKind.HIGHLIGHT_NOTE, withBody))),
            )
            assertEquals(HttpMethod.Put, putCapture.methods.single())
            assertEquals("/api/v1/highlights/$SENDER_HIGHLIGHT/note", putCapture.paths.single())

            val deleteCapture = SenderCapture()
            val delete = senderWith(deleteCapture) { respond("", HttpStatusCode.NotFound) }
            val cleared = OutboxPayload.HighlightNote(SENDER_HIGHLIGHT, null)
            assertEquals(
                SendOutcome.ReplaySuccess,
                delete.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(senderRow(OutboxKind.HIGHLIGHT_NOTE, cleared))),
            )
            assertEquals(HttpMethod.Delete, deleteCapture.methods.single())
            assertEquals("/api/v1/highlights/$SENDER_HIGHLIGHT/note", deleteCapture.paths.single())
        }

    @Test
    fun highlightTagsPutsTagList() =
        runTest {
            val capture = SenderCapture()
            val sender = senderWith(capture) { respond("{}", HttpStatusCode.OK, senderJsonHeaders) }

            val payload = OutboxPayload.HighlightTags(SENDER_HIGHLIGHT, listOf("a", "b"))
            assertEquals(
                SendOutcome.Success(),
                sender.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(senderRow(OutboxKind.HIGHLIGHT_TAGS, payload))),
            )
            assertEquals(HttpMethod.Put, capture.methods.single())
            assertEquals("/api/v1/highlights/$SENDER_HIGHLIGHT/tags", capture.paths.single())
            assertTrue(assertNotNull(capture.bodies.single()).contains("\"tags\":[\"a\",\"b\"]"))
        }

    @Test
    fun highlightDeleteMaps404ToReplaySuccess() =
        runTest {
            val capture = SenderCapture()
            val sender = senderWith(capture) { respond("", HttpStatusCode.NotFound) }

            val payload = OutboxPayload.HighlightDelete(SENDER_HIGHLIGHT)
            assertEquals(
                SendOutcome.ReplaySuccess,
                sender.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(senderRow(OutboxKind.HIGHLIGHT_DELETE, payload))),
            )
            assertEquals(HttpMethod.Delete, capture.methods.single())
            assertEquals("/api/v1/highlights/$SENDER_HIGHLIGHT", capture.paths.single())
        }

    @Test
    fun documentNotePutsDocumentNote() =
        runTest {
            val capture = SenderCapture()
            val sender = senderWith(capture) { respond("{}", HttpStatusCode.OK, senderJsonHeaders) }

            val payload = OutboxPayload.DocumentNote("note body", null)
            val noteRow = senderRow(OutboxKind.DOCUMENT_NOTE, payload, SENDER_DOCUMENT)
            assertEquals(SendOutcome.Success(), sender.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(noteRow)))
            assertEquals(HttpMethod.Put, capture.methods.single())
            assertEquals("/api/v1/documents/doc_01/note", capture.paths.single())
        }

    @Test
    fun unauthorizedThenSuccessRefreshesAndSucceeds() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("stale-token")
            tokenStorage.saveRefreshToken("refresh-old")
            tokenStorage.saveExpiresAt(SENDER_TOKEN_EXPIRY)
            val authorizations = mutableListOf<String?>()
            var noteCalls = 0
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/refresh" ->
                            respond(refreshJson, HttpStatusCode.OK, senderJsonHeaders)
                        else -> {
                            noteCalls++
                            authorizations += request.headers[HttpHeaders.Authorization]
                            if (noteCalls == 1) {
                                respond("nope", HttpStatusCode.Unauthorized)
                            } else {
                                respond("{}", HttpStatusCode.OK, senderJsonHeaders)
                            }
                        }
                    }
                }
            val transport = AuthenticatedApiTransport(tokenStorage, engine = engine, registry = boundSenderRegistry())
            val sender = ApiOutboxSender(transport)

            val outcome = sender.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(noteRow()))

            assertEquals(SendOutcome.Success(), outcome)
            assertEquals(listOf<String?>("Bearer stale-token", "Bearer access-new"), authorizations)
        }

    @Test
    fun unauthorizedTwiceYieldsHttp401NotTransport() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("stale-token")
            tokenStorage.saveRefreshToken("refresh-old")
            tokenStorage.saveExpiresAt(SENDER_TOKEN_EXPIRY)
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/refresh" -> respond(refreshJson, HttpStatusCode.OK, senderJsonHeaders)
                        else -> respond("nope", HttpStatusCode.Unauthorized)
                    }
                }
            val transport = AuthenticatedApiTransport(tokenStorage, engine = engine, registry = boundSenderRegistry())
            val sender = ApiOutboxSender(transport)

            val outcome = sender.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(noteRow()))

            val http = assertIs<SendOutcome.Http>(outcome)
            assertEquals(401, http.status)
            val classification = classify(OutboxKind.DOCUMENT_NOTE, outcome, attempts = 0, now = 0) { false }
            assertEquals(Classification.AuthBlocked, classification)
        }

    @Test
    fun transportFailureYieldsTransportOutcome() =
        runTest {
            val sender = senderWith { throw FakeNetworkFailure() }

            val outcome = sender.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(noteRow()))

            assertIs<SendOutcome.Transport>(outcome)
        }

    private val refreshJson =
        """{"access_token":"access-new","refresh_token":"refresh-new","expires_at":$SENDER_TOKEN_EXPIRY}"""

    @Test
    fun problemBodyIsParsedIntoTheOutcome() =
        runTest {
            val body =
                """{"type":"about:blank","title":"Not Found","status":404,"detail":"document gone",""" +
                    """"code":"not_found","errors":[{"field":"document_id","message":"gone"}]}"""
            val sender = senderWith { respond(body, HttpStatusCode.NotFound, senderJsonHeaders) }

            val outcome = assertIs<SendOutcome.Http>(sender.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(noteRow())))

            assertEquals(404, outcome.status)
            assertEquals("not_found", outcome.problem?.code)
            assertEquals("document gone", outcome.problem?.detail)
            assertEquals(listOf(FieldError("document_id", "gone")), outcome.problem?.errors)
        }

    @Test
    fun nonJsonErrorBodyLeavesProblemNull() =
        runTest {
            val sender = senderWith { respond("<html>nope</html>", HttpStatusCode.NotFound) }

            val outcome = assertIs<SendOutcome.Http>(sender.send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(noteRow())))

            assertEquals(null, outcome.problem)
            assertEquals("<html>nope</html>", outcome.message)
        }
}
