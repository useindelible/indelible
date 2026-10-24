package app.indelible.reader.repository

import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.json
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs

private const val WHEN = "2026-02-01T00:00:00Z"

class ApiReaderRepositoryOnlineViewTest {
    private suspend fun ReaderOfflineHarness.queueEdits() {
        repository.upsertItemNote(DL_DOC, "mine")
        repository.updateHighlightColor(DL_DOC, "hlt_1", "blue")
        repository.recordProgress(DL_DOC, percent = 60f, sessionId = "s")
    }

    @Test
    fun online_read_of_cached_document_shows_queued_note_highlight_and_progress() =
        runTest {
            val h = readerOfflineHarness()
            h.keepCopy()
            h.queueEdits()

            assertEquals("mine", h.note())
            assertEquals("blue", h.highlights().single().color)
            assertEquals(60, h.document().progressPercent)
        }

    @Test
    fun online_read_of_cached_document_refreshes_the_copy() =
        runTest {
            val h = readerOfflineHarness()
            h.keepCopy()
            h.server.route(NOTE_PATH) {
                json("""{"id":"note_1","body":"edited elsewhere","created_at":"$WHEN","updated_at":"$WHEN"}""")
            }

            assertEquals("edited elsewhere", h.note())
            assertEquals("edited elsewhere", h.store.cachedDocument(DL_SCOPE, DL_DOC)?.noteBody)
        }

    @Test
    fun online_read_of_uncached_document_shows_queued_note_highlight_and_progress() =
        runTest {
            val h = readerOfflineHarness()
            h.queueEdits()

            assertEquals("mine", h.note())
            assertEquals("blue", h.highlights().single().color)
            assertEquals(60, h.document().progressPercent)
        }

    @Test
    fun online_read_with_stale_refresh_returns_existing_local_view() =
        runTest {
            val h = readerOfflineHarness()
            h.keepCopy()
            h.repository.updateHighlightColor(DL_DOC, "hlt_1", "blue")
            h.duringFetchOf(HIGHLIGHTS_PATH) { h.acknowledgeQueued() }

            val highlights = h.highlights()

            assertEquals("blue", highlights.single().color)
            val copied = h.store.cachedHighlights(DL_SCOPE, DL_DOC)
            assertEquals("blue", copied.single().color)
        }

    @Test
    fun staleUncachedHighlightsAreWithheldAfterRetryLimit() =
        runTest {
            val h = readerOfflineHarness()
            h.duringFetchOf(HIGHLIGHTS_PATH) { h.acknowledgeNoteEdit() }

            val result = h.repository.listHighlights(DL_DOC)

            assertIs<StaleReadException>(result.exceptionOrNull())
            assertEquals(3, h.server.count(HIGHLIGHTS_PATH))
        }

    @Test
    fun staleUncachedProgressFallsBackToServer() =
        runTest {
            val h = readerOfflineHarness()
            h.duringFetchOf(READER_PATH) { h.acknowledgeProgress() }

            val document = h.document()

            assertEquals(20, document.progressPercent)
            assertEquals(3, h.server.count(READER_PATH))
        }

    @Test
    fun online_open_requests_auto_cache_once() =
        runTest {
            val h = readerOfflineHarness()

            h.repository.getItem(DL_DOC)
            h.repository.getItem(DL_DOC)
            runCurrent()

            assertFalse(checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC)).pinned)
            assertEquals(1, h.server.count(HTML_PATH))
        }

    @Test
    fun opening_a_copy_marks_it_recently_used() =
        runTest {
            val h = readerOfflineHarness()
            h.keepCopy()
            advanceTimeBy(5_000)

            h.repository.getItem(DL_DOC)

            assertEquals(5_000L, h.store.cachedDocument(DL_SCOPE, DL_DOC)?.lastOpenedAt)
        }
}
