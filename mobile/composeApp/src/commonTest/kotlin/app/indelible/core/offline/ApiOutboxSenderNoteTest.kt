package app.indelible.core.offline

import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.request.HttpRequestData
import io.ktor.client.request.HttpResponseData
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.test.runTest
import kotlinx.datetime.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

private const val BASE_MS = 1_000L
private const val NEWER_MS = 2_000L
private const val NOTE_PATH = "/api/v1/documents/$SENDER_DOCUMENT/note"
private const val CAPABILITIES_PATH = "/api/v1/server/capabilities"
private const val CAPABLE = """{"capabilities":["note_precondition"]}"""
private const val OLD_SERVER = """{"capabilities":[]}"""

private fun noteJson(
    body: String,
    updatedAtMs: Long,
): String {
    val at = Instant.fromEpochMilliseconds(updatedAtMs).toString()
    return """{"id":"note_1","body":"$body","created_at":"$at","updated_at":"$at"}"""
}

private fun conflictJson(
    body: String,
    updatedAtMs: Long,
): String {
    val problem = """"type":"about:blank","title":"Conflict","status":409,"detail":"changed","code":"note_conflict""""
    return """{$problem,"note":${noteJson(body, updatedAtMs)}}"""
}

private fun note(
    body: String = "mine",
    base: Long? = BASE_MS,
) = senderRow(OutboxKind.DOCUMENT_NOTE, OutboxPayload.DocumentNote(body, base), SENDER_DOCUMENT)

private fun SenderCapture.noteMethods() = methods.zip(paths).filter { it.second == NOTE_PATH }.map { it.first }

class ApiOutboxSenderNoteTest {
    private suspend fun ApiOutboxSender.sendNote(row: OutboxRow) = send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(row))

    /** Answers the capabilities ask with [capabilities] (null is 503) and the note PUT with [put]. */
    private suspend fun noteServer(
        capture: SenderCapture,
        capabilities: String? = CAPABLE,
        put: MockRequestHandleScope.(HttpRequestData) -> HttpResponseData = {
            respond(noteJson("mine", NEWER_MS), HttpStatusCode.OK, senderJsonHeaders)
        },
    ) = senderWith(capture) { request ->
        when {
            request.url.encodedPath != CAPABILITIES_PATH -> put(request)
            capabilities == null -> respond("down", HttpStatusCode.ServiceUnavailable)
            else -> respond(capabilities, HttpStatusCode.OK, senderJsonHeaders)
        }
    }

    @Test
    fun aCheckedNoteIsPutWithItsBase() =
        runTest {
            val capture = SenderCapture()
            val sender = noteServer(capture)

            val outcome = sender.sendNote(note())

            assertEquals(SendOutcome.Success(NEWER_MS), outcome)
            assertEquals(listOf(HttpMethod.Put), capture.noteMethods())
            val body = capture.bodies.last().orEmpty()
            assertTrue(""""base_updated_at":"1970-01-01T00:00:01Z"""" in body, body)
            assertTrue(""""force":false""" in body, body)
        }

    @Test
    fun aNullBaseIsPutWithoutABaseAndWithoutAskingTheServer() =
        runTest {
            val capture = SenderCapture()
            val sender = noteServer(capture, capabilities = OLD_SERVER)

            assertEquals(SendOutcome.Success(NEWER_MS), sender.sendNote(note(base = null)))
            assertEquals(listOf(NOTE_PATH), capture.paths)
            assertFalse(""""base_updated_at":"""" in capture.bodies.last().orEmpty())
        }

    @Test
    fun aConflictAnswerIsANoteConflict() =
        runTest {
            val sender =
                noteServer(SenderCapture()) {
                    respond(conflictJson("theirs", NEWER_MS), HttpStatusCode.Conflict, senderJsonHeaders)
                }

            assertEquals(SendOutcome.NoteConflict("theirs", NEWER_MS), sender.sendNote(note()))
        }

    @Test
    fun aConflictWithTheSameBodyIsApplied() =
        runTest {
            val sender =
                noteServer(SenderCapture()) {
                    respond(conflictJson("mine", NEWER_MS), HttpStatusCode.Conflict, senderJsonHeaders)
                }

            assertEquals(SendOutcome.Success(NEWER_MS), sender.sendNote(note("mine")))
        }

    @Test
    fun aConflictThatIsNotANoteIsRejected() =
        runTest {
            val sender =
                noteServer(SenderCapture()) {
                    respond("""{"code":"conflict","detail":"taken"}""", HttpStatusCode.Conflict, senderJsonHeaders)
                }

            assertEquals(409, assertIs<SendOutcome.Http>(sender.sendNote(note())).status)
        }

    @Test
    fun aServerWithoutThePreconditionHoldsACheckedNote() =
        runTest {
            val capture = SenderCapture()
            val sender = noteServer(capture, capabilities = OLD_SERVER)

            assertEquals(SendOutcome.Unsupported, sender.sendNote(note()))
            assertEquals(emptyList(), capture.noteMethods())
        }

    @Test
    fun anUnknownCapabilityAnswerWaits() =
        runTest {
            val capture = SenderCapture()
            val sender = noteServer(capture, capabilities = null)

            assertEquals(SendOutcome.NoAnswer, sender.sendNote(note()))
            assertEquals(emptyList(), capture.noteMethods())
        }

    @Test
    fun aPutSuccessCarriesTheServersUpdatedAt() =
        runTest {
            val sender =
                noteServer(SenderCapture()) {
                    respond(noteJson("mine", 3_000L), HttpStatusCode.OK, senderJsonHeaders)
                }

            assertEquals(SendOutcome.Success(3_000L), sender.sendNote(note()))
        }

    @Test
    fun aPutAnsweredWithoutANoteCarriesNoTimestamp() =
        runTest {
            val sender = senderWith { respond("{}", HttpStatusCode.OK, senderJsonHeaders) }

            assertEquals(SendOutcome.Success(), sender.sendNote(note(base = null)))
        }
}
