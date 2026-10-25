package app.indelible.core.offline

import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

private const val READER_PATH = "/api/v1/documents/$DL_DOC"

class DownloadPinnedBookTest {
    private suspend fun TestScope.book(chapterList: String): DownloadHarness =
        downloadHarness {
            documentType = "book"
            available = listOf("epub")
            route(EPUB_TOC_PATH) { json(epubToc(chapters = null, chapterList = chapterList)) }
        }

    @Test
    fun pinnedBookWithAPendingListWaitsInsteadOfFailing() =
        runTest {
            val h = book("pending")

            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()

            assertEquals(Acquisition.Waiting, h.acquisition())
            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
        }

    @Test
    fun reopeningAPendingPinnedBookRetriesAndKeepsThePin() =
        runTest {
            val h = book("pending")
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
    fun pendingPinRetriesWhenTheConnectionReturns() =
        runTest {
            val h = book("pending")
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()
            h.server.route(EPUB_TOC_PATH) { json(epubToc()) }

            h.online.value = false
            runCurrent()
            h.online.value = true
            runCurrent()

            assertTrue(checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC)).pinned)
        }

    @Test
    fun pinOnABookThatNeedsReuploadFailsButKeepsThePin() =
        runTest {
            val h = book("needs_reupload")
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()
            assertEquals(Acquisition.Failed(DownloadFailure.SERVER), h.acquisition())

            h.server.route(EPUB_TOC_PATH) { json(epubToc()) }
            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()

            assertTrue(checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC)).pinned)
        }

    @Test
    fun pinOnAServerTooOldIsNotParked() =
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
            val fetched = h.server.count(READER_PATH)

            h.online.value = false
            runCurrent()
            h.online.value = true
            runCurrent()

            assertEquals(fetched, h.server.count(READER_PATH), "a reconnect must not retry a server too old")
        }
}
