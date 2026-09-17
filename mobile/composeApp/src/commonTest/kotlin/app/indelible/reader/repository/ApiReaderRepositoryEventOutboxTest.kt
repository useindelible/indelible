package app.indelible.reader.repository

import app.indelible.core.offline.OutboxKind
import app.indelible.core.offline.OutboxPayload
import io.ktor.http.HttpMethod
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

private const val SESSION_ID = "ses_1"

class ApiReaderRepositoryEventOutboxTest {
    @Test
    fun progress_enqueues_one_reading_event_in_basis_points() =
        runTest {
            val harness = readerOutboxHarness()

            harness.repository.recordProgress(OFFLINE_DOCUMENT_ID, 42.5f, SESSION_ID)

            val row = harness.store.rowsOf(OFFLINE_SCOPE, OutboxKind.READING_EVENT).single()
            assertEquals(OFFLINE_DOCUMENT_ID, row.entityId)
            assertEquals(OFFLINE_DOCUMENT_ID, row.documentId)
            val payload = assertIs<OutboxPayload.ReadingEvent>(row.payload)
            assertEquals("progress", payload.kind)
            assertEquals(4250, payload.progressBasisPoints)
            assertEquals(0L, payload.originSeq)
            assertTrue(payload.eventId.startsWith("rev_"))
            assertTrue(payload.recordedAtEpochMs > 0)
        }

    @Test
    fun progress_basis_points_are_clamped_to_the_valid_range() =
        runTest {
            val harness = readerOutboxHarness()

            harness.repository.recordProgress(OFFLINE_DOCUMENT_ID, 150f, SESSION_ID)
            harness.repository.recordProgress(OFFLINE_DOCUMENT_ID, -4f, SESSION_ID)

            val points =
                harness.store
                    .rowsOf(OFFLINE_SCOPE, OutboxKind.READING_EVENT)
                    .map { (it.payload as OutboxPayload.ReadingEvent).progressBasisPoints }
            assertEquals(listOf(10_000, 0), points)
        }

    @Test
    fun successive_events_take_successive_origin_seqs() =
        runTest {
            val harness = readerOutboxHarness()

            harness.repository.recordProgress(OFFLINE_DOCUMENT_ID, 10f, SESSION_ID)
            harness.repository.recordProgress(OFFLINE_DOCUMENT_ID, 20f, SESSION_ID)

            assertEquals(
                listOf(0L, 1L),
                harness.store
                    .rowsOf(OFFLINE_SCOPE, OutboxKind.READING_EVENT)
                    .map { (it.payload as OutboxPayload.ReadingEvent).originSeq },
            )
        }

    @Test
    fun an_opened_event_carries_no_progress_fields() =
        runTest {
            val harness = readerOutboxHarness()

            harness.repository.recordOpened(OFFLINE_DOCUMENT_ID, SESSION_ID)

            val payload =
                assertIs<OutboxPayload.ReadingEvent>(
                    harness.store
                        .rowsOf(OFFLINE_SCOPE, OutboxKind.READING_EVENT)
                        .single()
                        .payload,
                )
            assertEquals("opened", payload.kind)
            assertEquals(SESSION_ID, payload.sessionId)
            assertNull(payload.progressBasisPoints)
            assertNull(payload.positionJson)
            assertNull(payload.activeMs)
        }

    @Test
    fun item_note_enqueues_a_document_note() =
        runTest {
            val harness = readerOutboxHarness()

            val saved = harness.repository.upsertItemNote(OFFLINE_DOCUMENT_ID, "a note").getOrThrow()

            assertEquals("a note", saved)
            val row = harness.store.rowsOf(OFFLINE_SCOPE, OutboxKind.DOCUMENT_NOTE).single()
            assertEquals(OFFLINE_DOCUMENT_ID, row.entityId)
            assertEquals(OutboxPayload.DocumentNote("a note", null), row.payload)
        }

    @Test
    fun without_a_scope_writes_go_straight_to_the_network() =
        runTest {
            val harness = readerOutboxHarness(scope = null)

            harness.repository.recordProgress(OFFLINE_DOCUMENT_ID, 42.5f, SESSION_ID)
            harness.repository.createHighlight(OFFLINE_DOCUMENT_ID, "Yellow", "quoted", 0, 5)
            harness.repository.deleteHighlight(OFFLINE_DOCUMENT_ID, "hlt_1")
            harness.repository.updateHighlightColor(OFFLINE_DOCUMENT_ID, "hlt_1", "Blue")
            harness.repository.upsertHighlightNote(OFFLINE_DOCUMENT_ID, "hlt_1", "note")
            harness.repository.deleteHighlightNote(OFFLINE_DOCUMENT_ID, "hlt_1")
            harness.repository.setHighlightTags(OFFLINE_DOCUMENT_ID, "hlt_1", listOf("t"))
            harness.repository.upsertItemNote(OFFLINE_DOCUMENT_ID, "a note")

            assertTrue(harness.store.pendingOrdered(OFFLINE_SCOPE).isEmpty())
            assertEquals(
                listOf(
                    HttpMethod.Patch to "/api/v1/documents/$OFFLINE_DOCUMENT_ID/progress",
                    HttpMethod.Post to "/api/v1/documents/$OFFLINE_DOCUMENT_ID/highlights",
                    HttpMethod.Delete to "/api/v1/highlights/hlt_1",
                    HttpMethod.Patch to "/api/v1/highlights/hlt_1",
                    HttpMethod.Put to "/api/v1/highlights/hlt_1/note",
                    HttpMethod.Delete to "/api/v1/highlights/hlt_1/note",
                    HttpMethod.Put to "/api/v1/highlights/hlt_1/tags",
                    HttpMethod.Put to "/api/v1/documents/$OFFLINE_DOCUMENT_ID/note",
                ),
                harness.requests.map { it.method to it.path },
            )
        }
}
