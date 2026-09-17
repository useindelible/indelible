package app.indelible.api

import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.offline.ApiOutboxSender
import app.indelible.core.offline.OutboxKind
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.OutboxRow
import app.indelible.core.offline.OutboxState
import app.indelible.core.offline.SendOutcome
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

private const val SCOPE = "http://localhost:38473|usr_1"
private const val DOCUMENT_ID = "doc_01ABC"
private const val CLIENT_ID = "cli_parity"

class ReadingEventsParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    private class Sent {
        var method: HttpMethod? = null
        var path: String? = null
        var authorization: String? = null
        var body: String? = null
    }

    private fun readingEventRow(
        originSeq: Long,
        recordedAtEpochMs: Long,
    ) = OutboxRow(
        seq = originSeq,
        scope = SCOPE,
        id = "obx_$originSeq",
        kind = OutboxKind.READING_EVENT,
        entityId = DOCUMENT_ID,
        documentId = DOCUMENT_ID,
        payload =
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
                recordedAtEpochMs = recordedAtEpochMs,
            ),
        createdAt = 0,
        attempts = 0,
        lastAttemptAt = null,
        nextAttemptAt = 0,
        state = OutboxState.PENDING,
        lastError = null,
    )

    @Test
    fun readingEventBatchMatchesTheAppendReadingEventsWireContract() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            val sent = Sent()
            val engine =
                MockEngine { request ->
                    sent.method = request.method
                    sent.path = request.url.encodedPath
                    sent.authorization = request.headers[HttpHeaders.Authorization]
                    sent.body = (request.body as TextContent).text
                    respond("""{"accepted":2,"replayed":0}""", HttpStatusCode.Accepted, jsonHeaders)
                }
            val sender = ApiOutboxSender(AuthenticatedApiTransport(tokenStorage, engine = engine))

            val outcome =
                sender.send(
                    SCOPE,
                    CLIENT_ID,
                    listOf(
                        readingEventRow(originSeq = 1, recordedAtEpochMs = 1_767_225_600_000L),
                        readingEventRow(originSeq = 2, recordedAtEpochMs = 1_767_225_601_000L),
                    ),
                )

            assertEquals(SendOutcome.Success, outcome)
            assertEquals(HttpMethod.Post, sent.method)
            assertEquals("/api/v1/documents/$DOCUMENT_ID/reading-events", sent.path)
            assertEquals("Bearer test-token", sent.authorization)

            val body = assertNotNull(sent.body)
            assertTrue(body.contains(""""client_id":"$CLIENT_ID""""), body)
            assertTrue(body.contains(""""progress_basis_points":4237"""), body)
            assertTrue(body.contains(""""recorded_at":"2026-01-01T00:00:00Z""""), body)
            assertTrue(body.contains(""""recorded_at":"2026-01-01T00:00:01Z""""), body)

            val seqs = Regex(""""origin_seq":(\d+)""").findAll(body).map { it.groupValues[1].toLong() }.toList()
            assertEquals(listOf(1L, 2L), seqs)
            assertTrue(seqs.zipWithNext().all { (a, b) -> b > a }, body)
        }

    private companion object {
        const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
