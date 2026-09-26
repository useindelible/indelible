package app.indelible.core.offline

import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import okio.IOException
import okio.fakefilesystem.FakeFileSystem
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

private const val HTML = "/api/v1/assets/documents/$DL_DOC/readable_html"

class DownloadRaceTest {
    /** A harness whose readable HTML response is held until [release] completes. */
    private suspend fun TestScope.gated(release: CompletableDeferred<Unit>): DownloadHarness =
        downloadHarness {
            route(HTML) {
                release.await()
                respond("<p>late</p>", HttpStatusCode.OK)
            }
        }

    /** A harness whose row removals each wait for the next gate before deleting. */
    private suspend fun TestScope.removalHeld(vararg gates: CompletableDeferred<Unit>): DownloadHarness {
        val pending = ArrayDeque(gates.toList())
        return downloadHarness(
            store = { real ->
                object : OfflineStore by real {
                    override suspend fun removeCachedDocument(
                        scope: String,
                        documentId: String,
                    ) {
                        pending.removeFirst().await()
                        real.removeCachedDocument(scope, documentId)
                    }
                }
            },
        )
    }

    @Test
    fun sessionChangeMidDownloadLeavesNoRowsOrFiles() =
        runTest {
            val release = CompletableDeferred<Unit>()
            val h = gated(release)
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()

            h.signedIn.switchTo("http://b.test|u2")
            release.complete(Unit)
            runCurrent()

            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertFalse(h.fs.exists(h.files.documentDir(DL_SCOPE, DL_DOC) / "1"))
            assertNull(h.acquisition())
        }

    @Test
    fun purgeCancelsInFlightDownloadBeforeDeletingTree() =
        runTest {
            val release = CompletableDeferred<Unit>()
            val h = gated(release)
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()

            ScopePurger(h.store) { h.manager.removeAllDownloads(it) }.purge(DL_SCOPE)
            release.complete(Unit)
            runCurrent()

            assertFalse(h.fs.exists(h.files.scopeDir(DL_SCOPE)))
            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
        }

    @Test
    fun removeCancelsActiveDownloadAndNothingReinstalls() =
        runTest {
            val release = CompletableDeferred<Unit>()
            val h = gated(release)
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()

            h.manager.removeFromDevice(DL_SCOPE, DL_DOC)
            release.complete(Unit)
            runCurrent()

            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertFalse(h.fs.exists(h.files.documentDir(DL_SCOPE, DL_DOC)))
            assertNull(h.acquisition())
        }

    @Test
    fun removeAllCancelsQueuedRequests() =
        runTest {
            val release = CompletableDeferred<Unit>()
            val h = gated(release)
            h.manager.keepOffline(h.session, DL_DOC)
            h.manager.cacheOnOpen(h.session, "doc_2")
            runCurrent()

            h.manager.removeAllDownloads(DL_SCOPE)
            release.complete(Unit)
            runCurrent()

            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertEquals(0, h.server.count("/api/v1/documents/doc_2"))
            assertFalse(h.fs.exists(h.files.scopeDir(DL_SCOPE)))
        }

    @Test
    fun noSpaceFailsAndPausesAutoCache() =
        runTest {
            val full = FailingWriteFileSystem(FakeFileSystem()) { IOException("write failed: ENOSPC") }
            val h = downloadHarness(fs = full)
            h.signedIn.enqueue(OutboxPayload.DocumentNote("queued", null))

            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()
            h.manager.cacheOnOpen(h.session, "doc_2")
            runCurrent()

            assertEquals(Acquisition.Failed(DownloadFailure.NO_SPACE), h.acquisition())
            assertEquals(0, h.server.count("/api/v1/documents/doc_2"))
            assertEquals(1, h.store.pendingOrdered(DL_SCOPE).size)
            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
        }

    @Test
    fun duplicateRequestIsDeduplicated() =
        runTest {
            val h = downloadHarness()

            h.manager.cacheOnOpen(h.session, DL_DOC)
            h.manager.cacheOnOpen(h.session, DL_DOC)
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()

            assertEquals(1, h.server.count(HTML))
            assertTrue(checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC)).pinned)
            assertNull(h.acquisition())
        }

    @Test
    fun pinWithoutCopyWaitsForConnection() =
        runTest {
            val h = downloadHarness()
            h.online.value = false
            runCurrent()

            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()
            assertEquals(Acquisition.WaitingForConnection, h.acquisition())
            assertTrue(h.server.calls.isEmpty())

            h.online.value = true
            runCurrent()

            assertTrue(checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC)).pinned)
            assertNull(h.acquisition())
        }

    @Test
    fun staleFirstInstallThreeTimesInstallsNothingAndWaits() =
        runTest {
            lateinit var h: DownloadHarness
            h =
                downloadHarness {
                    route("/api/v1/documents/$DL_DOC/highlights") {
                        h.signedIn.run { store.remove(scope, enqueue(OutboxPayload.DocumentNote("mine", null))) }
                        json("""{"count":0,"highlights":[]}""")
                    }
                }

            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()

            assertEquals(3, h.server.count("/api/v1/documents/$DL_DOC/highlights"))
            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertFalse(h.fs.exists(h.generationDir(1)))
            assertEquals(Acquisition.Waiting, h.acquisition())
        }

    @Test
    fun pinOfADocumentWithoutReadableContentWaitsInsteadOfClaimingACopy() =
        runTest {
            val h = downloadHarness { available = emptyList() }

            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()

            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertEquals(Acquisition.Waiting, h.acquisition())
            assertEquals(0, h.server.count(HTML))
        }

    @Test
    fun freeingSpaceResumesAutoCaching() =
        runTest {
            val full = FailingWriteFileSystem(FakeFileSystem()) { IOException("write failed: ENOSPC") }
            val h = downloadHarness(fs = full)
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()
            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()
            assertEquals(1, h.server.count(HTML))

            h.manager.removeFromDevice(DL_SCOPE, DL_DOC)
            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()

            assertEquals(2, h.server.count(HTML))
        }

    @Test
    fun aPinThatInstallsResumesAutoCaching() =
        runTest {
            val full = FailingWriteFileSystem(FakeFileSystem()) { IOException("write failed: ENOSPC") }
            val h = downloadHarness(fs = full)
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()
            h.manager.cacheOnOpen(h.session, "doc_2")
            runCurrent()
            assertEquals(0, h.server.count("/api/v1/documents/doc_2"))

            full.failing = false
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()
            h.manager.cacheOnOpen(h.session, "doc_2")
            runCurrent()

            assertEquals(1, h.server.count("/api/v1/documents/doc_2"))
        }

    @Test
    fun aDownloadDuringRemovalNeverLeavesARowWithoutItsFiles() =
        runTest {
            val removed = CompletableDeferred<Unit>()
            val resume = CompletableDeferred<Unit>()
            val h =
                downloadHarness(
                    store = { real ->
                        object : OfflineStore by real {
                            override suspend fun removeCachedDocument(
                                scope: String,
                                documentId: String,
                            ) {
                                real.removeCachedDocument(scope, documentId)
                                removed.complete(Unit)
                                resume.await()
                            }
                        }
                    },
                )
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()
            val removal = launch { h.manager.removeFromDevice(DL_SCOPE, DL_DOC) }
            removed.await()

            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()
            resume.complete(Unit)
            removal.join()
            runCurrent()

            val copy = checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertTrue(h.fs.exists(h.files.generationDir(DL_SCOPE, DL_DOC, copy.generation)))
        }

    @Test
    fun aRequestWaitsForTheLastOfOverlappingRemovals() =
        runTest {
            val first = CompletableDeferred<Unit>()
            val second = CompletableDeferred<Unit>()
            val h = removalHeld(first, second)
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()
            val removals = List(2) { launch { h.manager.removeFromDevice(DL_SCOPE, DL_DOC) } }
            runCurrent()

            first.complete(Unit)
            runCurrent()
            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()
            second.complete(Unit)
            removals.forEach { it.join() }
            runCurrent()

            val copy = checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertTrue(h.fs.exists(h.files.generationDir(DL_SCOPE, DL_DOC, copy.generation)))
        }

    @Test
    fun keepOfflineDuringRemovalLeavesAPinnedCopy() =
        runTest {
            val gate = CompletableDeferred<Unit>()
            val h = removalHeld(gate)
            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()
            val removal = launch { h.manager.removeFromDevice(DL_SCOPE, DL_DOC) }
            runCurrent()

            h.manager.keepOffline(h.session, DL_DOC)
            gate.complete(Unit)
            removal.join()
            runCurrent()

            val copy = checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertTrue(copy.pinned)
            assertTrue(h.fs.exists(h.files.generationDir(DL_SCOPE, DL_DOC, copy.generation)))
        }
}
