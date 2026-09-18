package app.indelible.reader.repository

import app.indelible.core.network.ApiException
import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.json
import app.indelible.reader.model.ArticleTocStatus
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.test.currentTime
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

private const val READY_TOC =
    """{"entries":[{"id":"h-1","title":"First","depth":0,"source_heading_index":0,"word_count":10}],""" +
        """"status":"ready","truncated":false}"""

class ApiReaderRepositoryOfflineReadTest {
    @Test
    fun get_item_falls_back_when_unreachable() =
        runTest {
            val h = readerOfflineHarness()
            h.keepCopy()
            h.repository.recordProgress(DL_DOC, percent = 60f, sessionId = "s")
            h.goOffline()

            val document = h.document()

            assertEquals("Doc", document.title)
            assertEquals(60, document.progressPercent)
        }

    @Test
    fun falls_back_on_5xx_and_timeout_not_on_404() =
        runTest {
            val h = readerOfflineHarness()
            h.keepCopy()

            h.server.route(READER_PATH) { respond("boom", HttpStatusCode.InternalServerError) }
            assertEquals("Doc", h.document().title)
            h.server.route(READER_PATH) { awaitCancellation() }
            assertEquals("Doc", h.document().title)
            h.server.route(READER_PATH) { respond("gone", HttpStatusCode.NotFound) }
            val gone = h.repository.getItem(DL_DOC).exceptionOrNull()
            assertEquals(404, (gone as? ApiException)?.statusCode)
        }

    @Test
    fun gives_up_after_fallback_budget_when_cached() =
        runTest {
            val h = readerOfflineHarness()
            h.keepCopy()
            h.server.route(NOTE_PATH) { awaitCancellation() }
            val start = currentTime

            val note = h.note()

            assertEquals("server note", note)
            assertEquals(READ_BUDGET_MS, currentTime - start)
        }

    @Test
    fun readable_html_served_from_file() =
        runTest {
            val h = readerOfflineHarness()
            h.keepCopy()
            h.goOffline()

            assertEquals("<p>hello</p>", h.repository.fetchReadableHtml(DL_DOC).getOrThrow())
        }

    @Test
    fun toc_served_from_file() =
        runTest {
            val h = readerOfflineHarness { route(TOC_PATH) { json(READY_TOC) } }
            h.keepCopy()
            h.goOffline()

            val toc = h.repository.getArticleToc(DL_DOC).getOrThrow()

            assertEquals(ArticleTocStatus.READY, toc.status)
            assertEquals(listOf("First"), toc.entries.map { it.title })
        }

    @Test
    fun missing_toc_offline_answers_none() =
        runTest {
            val h = readerOfflineHarness { available = listOf("readable_html") }
            h.keepCopy()
            h.goOffline()

            val toc = h.repository.getArticleToc(DL_DOC).getOrThrow()

            assertEquals(ArticleTocStatus.NONE, toc.status)
        }

    @Test
    fun offline_reopen_after_stale_online_refresh_keeps_local_progress() =
        runTest {
            val h = readerOfflineHarness()
            h.keepCopy()
            h.repository.recordProgress(DL_DOC, percent = 60f, sessionId = "s")
            h.duringFetchOf(READER_PATH) { h.acknowledgeQueued() }
            h.repository.getItem(DL_DOC)

            h.goOffline()

            assertEquals(60, h.document().progressPercent)
        }

    @Test
    fun session_ending_mid_read_answers_from_the_copy() =
        runTest {
            val h = readerOfflineHarness()
            h.keepCopy()
            h.duringFetchOf(READER_PATH) { h.signOut() }

            assertEquals("Doc", h.document().title)
        }

    @Test
    fun uncached_offline_keeps_original_error() =
        runTest {
            val h = readerOfflineHarness()
            h.goOffline()

            val failure = h.repository.getItem(DL_DOC).exceptionOrNull()

            assertEquals("unreachable", failure?.message)
        }

    @Test
    fun no_session_no_fallback() =
        runTest {
            val h = readerOfflineHarness()
            h.keepCopy()
            h.signOut()
            h.goOffline()

            assertTrue(h.repository.getItem(DL_DOC).isFailure)
        }
}
