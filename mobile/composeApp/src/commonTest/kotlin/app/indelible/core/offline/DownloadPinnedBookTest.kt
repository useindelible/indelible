package app.indelible.core.offline

import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

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
            val asked = h.server.count(CAPABILITIES)

            h.online.value = false
            runCurrent()
            h.online.value = true
            runCurrent()

            assertEquals(asked, h.server.count(CAPABILITIES))
        }

    @Test
    fun unpinnedRequestForTheRunningDownloadAddsNoSecondDownload() =
        runTest {
            val release = CompletableDeferred<Unit>()
            val h =
                downloadHarness {
                    documentType = "book"
                    available = listOf("epub")
                    route(chapterPath(2)) {
                        release.await()
                        respond("<h1>2</h1>", HttpStatusCode.OK)
                    }
                }
            h.capBytes = 1
            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()

            h.manager.cacheOnOpen(h.session, DL_DOC)
            release.complete(Unit)
            runCurrent()

            assertEquals(1, h.server.count(EPUB_TOC_PATH))
            assertNull(h.acquisition())
        }
}
