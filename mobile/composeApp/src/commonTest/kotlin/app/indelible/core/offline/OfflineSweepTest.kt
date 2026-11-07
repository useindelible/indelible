package app.indelible.core.offline

import io.ktor.utils.io.ByteReadChannel
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import okio.Path
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

private const val FOREIGN = "http://b.test|u2"

class OfflineSweepTest {
    private fun DownloadHarness.fileOf(
        documentId: String,
        generation: Long = 1,
    ): Path = files.generationDir(DL_SCOPE, documentId, generation) / "readable.html"

    private suspend fun DownloadHarness.writeFile(
        path: Path,
        text: String,
    ) {
        files.write(path, ByteReadChannel(text))
    }

    private suspend fun DownloadHarness.createHighlight(
        id: String,
        documentId: String,
    ): String {
        store.enqueue(session, OutboxKind.HIGHLIGHT_CREATE, id, documentId) {
            val payload = OutboxPayload.HighlightCreate(id, "yellow", "quoted", null, null)
            applyToCachedHighlight(id, documentId, payload, at = 0L)
            payload to Unit
        }
        val rows = store.observeOutbox(DL_SCOPE).first()
        return rows.last().id
    }

    @Test
    fun sweepRemovesUnreferencedGenerations() =
        runTest {
            val h = downloadHarness()
            h.seed(DL_DOC, bytes = 10)
            h.writeFile(h.fileOf(DL_DOC, generation = 0), "old")
            h.writeFile(h.fileOf(DL_DOC, generation = 2), "partial")

            h.manager.cleanupAtLaunch()

            assertTrue(h.fs.exists(h.fileOf(DL_DOC)))
            assertFalse(h.fs.exists(h.generationDir(0)))
            assertFalse(h.fs.exists(h.generationDir(2)))
        }

    @Test
    fun sweepDropsRowsWithMissingOrTruncatedFiles() =
        runTest {
            val h = downloadHarness()
            h.seed("missing", bytes = 10)
            h.seed("truncated", bytes = 10)
            h.seed("intact", bytes = 10)
            h.fs.delete(h.fileOf("missing"))
            h.writeFile(h.fileOf("truncated"), "short")

            h.manager.cleanupAtLaunch()

            assertEquals(setOf("intact"), h.copyIds())
            assertFalse(h.fs.exists(h.files.documentDir(DL_SCOPE, "missing")))
            assertFalse(h.fs.exists(h.files.documentDir(DL_SCOPE, "truncated")))
            assertTrue(h.fs.exists(h.fileOf("intact")))
        }

    @Test
    fun sweepDeletesDirsWithoutRows() =
        runTest {
            val h = downloadHarness()
            h.seed("kept", bytes = 10)
            h.writeFile(h.fileOf(DL_DOC), "orphan")

            h.manager.cleanupAtLaunch()

            assertFalse(h.fs.exists(h.files.documentDir(DL_SCOPE, DL_DOC)))
            assertTrue(h.fs.exists(h.fileOf("kept")))
        }

    @Test
    fun sweepDeletesUnknownScopeDirs() =
        runTest {
            val h = downloadHarness()
            h.seed(DL_DOC, bytes = 10)
            val signedOut = h.files.scopeDir("http://gone.test|u9")
            h.writeFile(signedOut / DL_DOC / "1" / "readable.html", "left behind")
            h.writeFile(h.files.root / "stray" / "file", "unknown")

            h.manager.cleanupAtLaunch()

            assertFalse(h.fs.exists(signedOut))
            assertFalse(h.fs.exists(h.files.root / "stray"))
            assertTrue(h.fs.exists(h.fileOf(DL_DOC)))
        }

    @Test
    fun sweepKeepsPurgePendingScopeDir() =
        runTest {
            val h = downloadHarness()
            h.store.clientIdentity(FOREIGN)
            h.store.upsertCachedDocument(FOREIGN, catalogRow("foreign", bytes = 5L))
            h.store.setPurgePending(FOREIGN, true)
            val retained = h.files.scopeDir(FOREIGN) / "unlisted" / "1" / "readable.html"
            h.writeFile(retained, "the purge owns this")

            h.manager.cleanupAtLaunch()

            assertTrue(h.fs.exists(retained))
            assertEquals("foreign", h.store.cachedDocument(FOREIGN, "foreign")?.documentId)
        }

    @Test
    fun sweepDropsOrphanCachedHighlights() =
        runTest {
            val h = downloadHarness()
            val acknowledged = h.createHighlight("hlt_done", "uncached")
            h.createHighlight("hlt_queued", "uncached")
            h.store.remove(DL_SCOPE, acknowledged)

            h.manager.cleanupAtLaunch()

            assertEquals(listOf("hlt_queued"), h.store.cachedHighlights(DL_SCOPE, "uncached").map { it.id })
        }

    @Test
    fun sweepWaitsForTheActiveDownload() =
        runTest {
            val release = CompletableDeferred<Unit>()
            val h =
                downloadHarness {
                    route("/api/v1/documents/$DL_DOC/toc") {
                        release.await()
                        json("""{"entries":[],"status":"none","truncated":false}""")
                    }
                }
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()
            assertTrue(h.fs.exists(h.fileOf(DL_DOC)))

            val sweep = launch { h.manager.cleanupAtLaunch() }
            runCurrent()
            assertFalse(sweep.isCompleted)
            release.complete(Unit)
            sweep.join()

            assertEquals("<p>hello</p>", h.files.readText(h.fileOf(DL_DOC)))
            assertEquals(1L, h.store.cachedDocument(DL_SCOPE, DL_DOC)?.generation)
        }
}
