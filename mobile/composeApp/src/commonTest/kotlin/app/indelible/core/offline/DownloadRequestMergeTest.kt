package app.indelible.core.offline

import app.indelible.core.storage.DEFAULT_OFFLINE_CAP_BYTES
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull

class DownloadRequestMergeTest {
    /** A book over the cap whose last chapter waits for [release]; unpinned, it ends as Skipped after the download. */
    private suspend fun TestScope.overCapBook(
        release: CompletableDeferred<Unit>,
        store: (OfflineStore) -> OfflineStore = { it },
    ): DownloadHarness =
        downloadHarness(store = store) {
            documentType = "book"
            available = listOf("epub")
            route(chapterPath(2)) {
                release.await()
                respond("<h1>2</h1>", HttpStatusCode.OK)
            }
        }.also { it.capBytes = 1 }

    @Test
    fun unpinnedRequestForTheRunningDownloadAddsNoSecondDownload() =
        runTest {
            val release = CompletableDeferred<Unit>()
            val h = overCapBook(release)
            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()

            h.manager.cacheOnOpen(h.session, DL_DOC)
            release.complete(Unit)
            runCurrent()

            assertEquals(1, h.server.count(EPUB_TOC_PATH))
            assertNull(h.acquisition())
        }

    @Test
    fun requestDuringTheRemovalOfTheRunningDownloadIsKept() =
        runTest {
            val release = CompletableDeferred<Unit>()
            val cleanup = CompletableDeferred<Unit>()
            var holdReads = false
            val h =
                overCapBook(release) { real ->
                    object : OfflineStore by real {
                        override suspend fun cachedDocument(
                            scope: String,
                            documentId: String,
                        ): CachedDocumentRow? {
                            if (holdReads) cleanup.await()
                            return real.cachedDocument(scope, documentId)
                        }
                    }
                }
            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()
            holdReads = true
            val removal = launch { h.manager.removeFromDevice(DL_SCOPE, DL_DOC) }
            runCurrent()

            h.manager.cacheOnOpen(h.session, DL_DOC)
            holdReads = false
            cleanup.complete(Unit)
            release.complete(Unit)
            removal.join()
            h.capBytes = DEFAULT_OFFLINE_CAP_BYTES
            runCurrent()

            assertNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC), "the request made during the removal acquires anew")
        }
}
