package app.indelible.reader.repository

import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertTrue

class ReaderNoSessionTest {
    @Test
    fun outboxOwnedWritesFailWithoutASessionInsteadOfGoingDirect() =
        runTest {
            val harness = readerOutboxHarness(scope = null)

            val note = harness.repository.upsertItemNote(OFFLINE_DOCUMENT_ID, "body")
            val highlight = harness.repository.createHighlight(OFFLINE_DOCUMENT_ID, "yellow", "text", 0, 4)
            val color = harness.repository.updateHighlightColor(OFFLINE_DOCUMENT_ID, "hlt_1", "blue")

            assertTrue(note.isFailure)
            assertTrue(highlight.isFailure)
            assertTrue(color.isFailure)
            assertTrue(harness.requests.isEmpty(), harness.requests.joinToString { it.path })
        }

    @Test
    fun readingEventsAreDroppedWithoutASession() =
        runTest {
            val harness = readerOutboxHarness(scope = null)

            harness.repository.recordOpened(OFFLINE_DOCUMENT_ID, "session_1")
            harness.repository.recordProgress(OFFLINE_DOCUMENT_ID, 0.5f, "session_1")

            assertTrue(harness.requests.isEmpty(), harness.requests.joinToString { it.path })
        }
}
