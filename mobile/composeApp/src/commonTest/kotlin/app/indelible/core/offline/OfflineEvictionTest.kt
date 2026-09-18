package app.indelible.core.offline

import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import okio.fakefilesystem.FakeFileSystem
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

private const val HTML = "/api/v1/assets/documents/$DL_DOC/readable_html"
private const val FOREIGN = "http://b.test|u2"

class OfflineEvictionTest {
    @Test
    fun evictsUnpinnedLruAndStopsAtPinned() =
        runTest {
            val h = downloadHarness()
            h.seed("kept", bytes = 100, openedAt = 0, pinned = true)
            h.seed("oldest", bytes = 100, openedAt = 1)
            h.seed("older", bytes = 100, openedAt = 2)
            h.seed("newest", bytes = 100, openedAt = 3)

            h.capBytes = 250
            h.copies.enforceCap(DL_SCOPE, keep = null)
            assertEquals(setOf("kept", "newest"), h.copyIds())
            assertFalse(h.fs.exists(h.files.documentDir(DL_SCOPE, "oldest")))
            assertFalse(h.fs.exists(h.files.documentDir(DL_SCOPE, "older")))

            h.capBytes = 50
            h.copies.enforceCap(DL_SCOPE, keep = null)
            assertEquals(setOf("kept"), h.copyIds())
            assertTrue(h.fs.exists(h.files.documentDir(DL_SCOPE, "kept")))
        }

    @Test
    fun neverEvictsJustInstalled() =
        runTest {
            val h = downloadHarness()
            h.seed("kept", bytes = 100, pinned = true)
            h.seed("recent", bytes = 10, openedAt = 5_000)
            h.capBytes = 100

            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()

            assertEquals(setOf("kept", DL_DOC), h.copyIds())
            assertTrue(h.fs.exists(h.generationDir(1)))
        }

    @Test
    fun skipsAutoCacheLargerThanCap() =
        runTest {
            val h = downloadHarness { sizes = mapOf("readable_html" to 1_000L) }
            h.capBytes = 100

            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()
            assertEquals(0, h.server.count(HTML))
            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))

            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()
            assertTrue(checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC)).pinned)
        }

    @Test
    fun skipsAutoCacheThatDownloadsLargerThanCap() =
        runTest {
            val h = downloadHarness()
            h.capBytes = 20

            assertEquals(FetchResult.Skipped, h.fetcher.download(h.session, DL_DOC, pin = false))

            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertFalse(h.fs.exists(h.generationDir(1)))
        }

    @Test
    fun removeDeletesRowsBeforeFiles() =
        runTest {
            val fs = FailingDeleteFileSystem(FakeFileSystem())
            val h = downloadHarness(fs = fs)
            h.seed(DL_DOC, bytes = 10)
            fs.failUnder = h.files.documentDir(DL_SCOPE, DL_DOC)

            h.manager.removeFromDevice(DL_SCOPE, DL_DOC)

            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertTrue(fs.exists(h.generationDir(1)))
        }

    @Test
    fun unpinKeepsCopy() =
        runTest {
            val h = downloadHarness()
            h.seed(DL_DOC, bytes = 100, pinned = true)
            h.capBytes = 10

            h.manager.unpin(DL_SCOPE, DL_DOC)

            val copy = checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertFalse(copy.pinned)
            assertTrue(h.fs.exists(h.generationDir(1) / "readable.html"))
        }

    @Test
    fun unpinWithoutCopyWithdrawsTheRequest() =
        runTest {
            val h = downloadHarness()
            h.online.value = false
            runCurrent()
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()

            h.manager.unpin(DL_SCOPE, DL_DOC)
            h.online.value = true
            runCurrent()

            assertTrue(h.server.calls.isEmpty())
            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertNull(h.acquisition())
        }

    @Test
    fun retainedForeignScopeIsNeverEvictedListedOrCounted() =
        runTest {
            val h = downloadHarness()
            h.store.clientIdentity(FOREIGN)
            h.store.upsertCachedDocument(FOREIGN, catalogRow("foreign", bytes = 500L))
            h.store.setPurgePending(FOREIGN, true)
            h.seed(DL_DOC, bytes = 10)
            h.capBytes = 100

            h.copies.enforceCap(DL_SCOPE, keep = null)

            assertEquals(setOf(DL_DOC), h.copyIds())
            assertEquals("foreign", h.store.cachedDocument(FOREIGN, "foreign")?.documentId)
            val catalog = h.store.observeCatalog(DL_SCOPE).first()
            assertEquals(listOf(DL_DOC), catalog.map { it.documentId })
            assertEquals(10L, h.store.totalBytes(DL_SCOPE))
        }
}
