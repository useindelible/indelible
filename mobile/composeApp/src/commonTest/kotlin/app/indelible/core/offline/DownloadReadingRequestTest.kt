package app.indelible.core.offline

import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import io.ktor.utils.io.ByteReadChannel
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import okio.FileSystem
import okio.IOException
import okio.fakefilesystem.FakeFileSystem
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

private const val STORED_PDF = "/bucket/doc_1.pdf"
private const val PDF_FILE = "document.pdf"

class DownloadReadingRequestTest {
    private suspend fun TestScope.pdf(
        fs: FileSystem = FakeFileSystem(),
        configure: FakeServer.() -> Unit = {},
    ): DownloadHarness =
        downloadHarness(fs = fs) {
            documentType = "pdf"
            available = listOf("pdf")
            configure()
        }

    @Test
    fun readingRequestInstallsAPdfLargerThanTheCapUnpinned() =
        runTest {
            val h = pdf { sizes = mapOf("pdf" to 1_000L) }
            h.capBytes = 100

            h.manager.requestForReading(h.session, DL_DOC)
            runCurrent()

            val copy = checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertFalse(copy.pinned)
            assertNull(h.acquisition())
        }

    @Test
    fun readingRequestWhileOfflineFailsInsteadOfDropping() =
        runTest {
            val h = pdf()
            h.online.value = false
            runCurrent()

            h.manager.requestForReading(h.session, DL_DOC)
            runCurrent()

            assertEquals(Acquisition.Failed(DownloadFailure.NETWORK), h.acquisition())
        }

    @Test
    fun readingRequestConnectionFailureIsShown() =
        runTest {
            val h =
                downloadHarness {
                    documentType = "book"
                    available = listOf("epub")
                    route(CAPABILITIES) { respond("", HttpStatusCode.ServiceUnavailable) }
                }

            h.manager.requestForReading(h.session, DL_DOC)
            runCurrent()
            assertEquals(Acquisition.Failed(DownloadFailure.NETWORK), h.acquisition())

            val asked = h.server.count(CAPABILITIES)
            h.online.value = false
            runCurrent()
            h.online.value = true
            runCurrent()
            assertEquals(asked, h.server.count(CAPABILITIES), "a reading request is not parked for a reconnect")
        }

    @Test
    fun readingRequestRunsWhileAFullDiskPausedAutoCache() =
        runTest {
            val full = FailingWriteFileSystem(FakeFileSystem()) { IOException("write failed: ENOSPC") }
            val h = pdf(fs = full)
            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()
            assertEquals(Acquisition.Failed(DownloadFailure.NO_SPACE), h.acquisition())
            full.failing = false

            h.manager.requestForReading(h.session, DL_DOC)
            runCurrent()

            assertEquals(1L, h.store.cachedDocument(DL_SCOPE, DL_DOC)?.generation)
        }

    @Test
    fun readingRequestMergesWithAnAutoCache() =
        runTest {
            val h = pdf()

            h.manager.cacheOnOpen(h.session, DL_DOC)
            h.manager.requestForReading(h.session, DL_DOC)
            runCurrent()

            assertEquals(1, h.server.count(STORED_PDF))
            assertEquals(1L, h.store.cachedDocument(DL_SCOPE, DL_DOC)?.generation)
        }

    @Test
    fun readingRequestForADocumentWithNothingToReadFails() =
        runTest {
            val h = pdf { available = emptyList() }

            h.manager.requestForReading(h.session, DL_DOC)
            runCurrent()

            assertEquals(Acquisition.Failed(DownloadFailure.SERVER), h.acquisition())
        }

    @Test
    fun rowWithAMissingPdfFileIsRepairedAndKeepsItsPin() =
        runTest {
            val h = pdf()
            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()
            h.signedIn.enqueue(OutboxPayload.HighlightCreate("hlt_local", "yellow", "words", null, null))
            h.fs.delete(h.generationDir(1) / PDF_FILE)

            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()

            val copy = checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertEquals(2L, copy.generation)
            assertTrue(copy.pinned)
            assertTrue(h.fs.exists(h.generationDir(2) / PDF_FILE))
            assertEquals(1, h.store.pendingOrdered(DL_SCOPE).size)
        }

    @Test
    fun rowWithATruncatedPdfFileIsRepaired() =
        runTest {
            val h = pdf()
            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()
            h.files.write(h.generationDir(1) / PDF_FILE, ByteReadChannel("%PDF"))

            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()

            assertEquals(2L, h.store.cachedDocument(DL_SCOPE, DL_DOC)?.generation)
            assertEquals(2, h.server.count(STORED_PDF))
        }

    @Test
    fun completeRowIsNotDownloadedAgain() =
        runTest {
            val h = pdf()
            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()

            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()

            assertEquals(1, h.server.count(STORED_PDF))
            assertEquals(1L, h.store.cachedDocument(DL_SCOPE, DL_DOC)?.generation)
        }
}
