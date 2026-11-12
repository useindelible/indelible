package app.indelible.core.offline

import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.test.runTest
import kotlinx.datetime.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

private const val BASE_MS = 1_000L
private const val NEWER_MS = 2_000L
private const val NOTE_PATH = "/api/v1/documents/$SENDER_DOCUMENT/note"

private fun noteJson(
    body: String,
    updatedAtMs: Long,
): String {
    val at = Instant.fromEpochMilliseconds(updatedAtMs).toString()
    return """{"id":"note_1","body":"$body","created_at":"$at","updated_at":"$at"}"""
}

private fun note(
    body: String = "mine",
    base: Long? = BASE_MS,
) = senderRow(OutboxKind.DOCUMENT_NOTE, OutboxPayload.DocumentNote(body, base), SENDER_DOCUMENT)

class ApiOutboxSenderNoteTest {
    private suspend fun ApiOutboxSender.sendNote(row: OutboxRow) = send(SENDER_SESSION, SENDER_CLIENT_ID, listOf(row))

    /** Answers the GET with [current] (null is 404) and a PUT with the note as saved at [savedAtMs]. */
    private suspend fun noteServer(
        capture: SenderCapture,
        current: String?,
        savedAtMs: Long = NEWER_MS,
    ) = senderWith(capture) { request ->
        val saved = noteJson("mine", savedAtMs)
        when {
            request.method != HttpMethod.Get -> respond(saved, HttpStatusCode.OK, senderJsonHeaders)
            current == null -> respond("", HttpStatusCode.NotFound)
            else -> respond(current, HttpStatusCode.OK, senderJsonHeaders)
        }
    }

    @Test
    fun aDocumentNoteIsCheckedAgainstTheServerBeforeItIsPut() =
        runTest {
            val capture = SenderCapture()
            val sender = noteServer(capture, noteJson("old", BASE_MS))

            val outcome = sender.sendNote(note())

            assertEquals(SendOutcome.Success(NEWER_MS), outcome)
            assertEquals(listOf(HttpMethod.Get, HttpMethod.Put), capture.methods)
            assertEquals(listOf(NOTE_PATH, NOTE_PATH), capture.paths)
        }

    @Test
    fun aNewerServerNoteIsAConflictAndNothingIsPut() =
        runTest {
            val capture = SenderCapture()
            val sender = noteServer(capture, noteJson("theirs", NEWER_MS))

            val outcome = sender.sendNote(note())

            assertEquals(SendOutcome.NoteConflict("theirs", NEWER_MS), outcome)
            assertEquals(listOf(HttpMethod.Get), capture.methods)
        }

    @Test
    fun noServerNoteMeansNoConflict() =
        runTest {
            val capture = SenderCapture()
            val sender = noteServer(capture, current = null)

            assertEquals(SendOutcome.Success(NEWER_MS), sender.sendNote(note()))
            assertEquals(listOf(HttpMethod.Get, HttpMethod.Put), capture.methods)
        }

    @Test
    fun aNullBaseSkipsTheCheck() =
        runTest {
            val capture = SenderCapture()
            val sender = noteServer(capture, noteJson("theirs", NEWER_MS))

            assertEquals(SendOutcome.Success(NEWER_MS), sender.sendNote(note(base = null)))
            assertEquals(listOf(HttpMethod.Put), capture.methods)
        }

    @Test
    fun aNoteWrittenSinceTheCopyHadNoneIsAConflict() =
        runTest {
            val capture = SenderCapture()
            val sender = noteServer(capture, noteJson("theirs", NEWER_MS))

            assertEquals(SendOutcome.NoteConflict("theirs", NEWER_MS), sender.sendNote(note(base = 0L)))
            assertEquals(listOf(HttpMethod.Get), capture.methods)
        }

    @Test
    fun aLostPutResponseIsSeenAsAppliedOnRetry() =
        runTest {
            val capture = SenderCapture()
            val sender = noteServer(capture, noteJson("mine", NEWER_MS))

            assertEquals(SendOutcome.Success(NEWER_MS), sender.sendNote(note("mine")))
            assertEquals(listOf(HttpMethod.Get), capture.methods)
        }

    @Test
    fun aPutSuccessCarriesTheServersUpdatedAt() =
        runTest {
            val sender = noteServer(SenderCapture(), current = null, savedAtMs = 3_000L)

            assertEquals(SendOutcome.Success(3_000L), sender.sendNote(note()))
        }

    @Test
    fun aPutAnsweredWithoutANoteCarriesNoTimestamp() =
        runTest {
            val sender = senderWith { respond("{}", HttpStatusCode.OK, senderJsonHeaders) }

            assertEquals(SendOutcome.Success(), sender.sendNote(note(base = null)))
        }

    @Test
    fun aFailedCheckIsTheChecksOutcomeAndNothingIsPut() =
        runTest {
            val capture = SenderCapture()
            val sender = senderWith(capture) { respond("down", HttpStatusCode.ServiceUnavailable) }

            val outcome = assertIs<SendOutcome.Http>(sender.sendNote(note()))

            assertEquals(503, outcome.status)
            assertEquals(listOf(HttpMethod.Get), capture.methods)
        }
}
