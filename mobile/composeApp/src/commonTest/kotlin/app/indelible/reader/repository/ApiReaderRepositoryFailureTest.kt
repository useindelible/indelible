package app.indelible.reader.repository

import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertIs
import kotlin.test.assertTrue

class ApiReaderRepositoryFailureTest {
    @Test
    fun a_local_write_that_cannot_commit_fails_the_result_instead_of_throwing() =
        runTest {
            val harness = readerOutboxHarness(backgroundScope, failingStore = true)

            val results =
                listOf(
                    harness.repository.createHighlight(OFFLINE_DOCUMENT_ID, "Yellow", "quoted", 0, 5),
                    harness.repository.deleteHighlight(OFFLINE_DOCUMENT_ID, "hlt_1"),
                    harness.repository.updateHighlightColor(OFFLINE_DOCUMENT_ID, "hlt_1", "Blue"),
                    harness.repository.upsertHighlightNote(OFFLINE_DOCUMENT_ID, "hlt_1", "note"),
                    harness.repository.deleteHighlightNote(OFFLINE_DOCUMENT_ID, "hlt_1"),
                    harness.repository.setHighlightTags(OFFLINE_DOCUMENT_ID, "hlt_1", listOf("t")),
                    harness.repository.upsertItemNote(OFFLINE_DOCUMENT_ID, "a note"),
                )

            assertTrue(results.all { it.isFailure })
            results.forEach { assertIs<EnqueueFailure>(it.exceptionOrNull()) }
        }

    @Test
    fun recording_an_event_swallows_a_local_write_failure() =
        runTest {
            val harness = readerOutboxHarness(backgroundScope, failingStore = true)

            harness.repository.recordOpened(OFFLINE_DOCUMENT_ID, "ses_1")
            harness.repository.recordProgress(OFFLINE_DOCUMENT_ID, 42.5f, "ses_1")
        }
}
