package app.indelible.core.offline

import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class DownloadPinnedBookTest {
    private suspend fun TestScope.pendingBook(): DownloadHarness =
        downloadHarness {
            documentType = "book"
            available = listOf("epub")
            route(EPUB_TOC_PATH) { json(epubToc(chapters = null, chapterList = "pending")) }
        }

    @Test
    fun pinnedBookWithAPendingListWaitsInsteadOfFailing() =
        runTest {
            val h = pendingBook()

            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()

            assertEquals(Acquisition.Waiting, h.acquisition())
            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
        }

    @Test
    fun reopeningAPendingPinnedBookRetriesAndKeepsThePin() =
        runTest {
            val h = pendingBook()
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()

            h.server.route(EPUB_TOC_PATH) { json(epubToc()) }
            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()

            val copy = checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertTrue(copy.pinned, "the pin made while the list was pending must survive the retry")
            assertNull(h.acquisition())
        }

    @Test
    fun pinOnAServerTooOldStillFails() =
        runTest {
            val h =
                downloadHarness {
                    documentType = "book"
                    available = listOf("epub")
                    route(CAPABILITIES) { json("""{"capabilities":[]}""") }
                }

            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()

            assertEquals(Acquisition.Failed(DownloadFailure.SERVER_TOO_OLD), h.acquisition())
        }
}
