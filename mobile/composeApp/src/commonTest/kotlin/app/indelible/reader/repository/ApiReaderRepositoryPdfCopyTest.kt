package app.indelible.reader.repository

import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.FailingWriteFileSystem
import app.indelible.core.offline.FakeServer
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import okio.IOException
import okio.fakefilesystem.FakeFileSystem
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

private const val STORED_PDF = "/bucket/doc_1.pdf"
private const val PDF_FILE = "document.pdf"
private const val PDF_BYTES = 200_000

class ApiReaderRepositoryPdfCopyTest {
    private val body = ByteArray(PDF_BYTES) { 'x'.code.toByte() }.also { "%PDF-1.7".encodeToByteArray().copyInto(it) }

    private suspend fun TestScope.pdf(
        fs: okio.FileSystem = FakeFileSystem(),
        configure: FakeServer.() -> Unit = {},
    ) = readerOfflineHarness(fs) {
        documentType = "pdf"
        available = listOf("pdf")
        configure()
    }

    private fun ReaderOfflineHarness.pdfPath(gen: Long) = (download.generationDir(gen) / PDF_FILE).toString()

    private suspend fun ReaderOfflineHarness.lastState(): PdfCopy = repository.pdfCopy(DL_DOC).toList().last()

    /** Collects the copy states in the background; [runCurrent] drives them. */
    private fun TestScope.collecting(h: ReaderOfflineHarness): Pair<MutableList<PdfCopy>, Job> {
        val seen = mutableListOf<PdfCopy>()
        val job = launch { h.repository.pdfCopy(DL_DOC).toList(seen) }
        return seen to job
    }

    @Test
    fun copyOnDeviceIsReadyWithItsFilePath() =
        runTest {
            val h = pdf()
            h.keepCopy()

            assertEquals(PdfCopy.Ready(h.pdfPath(1)), h.repository.pdfCopy(DL_DOC).first())
        }

    @Test
    fun missingCopyRequestsAReadingDownloadAndReportsItsProgress() =
        runTest {
            val h =
                pdf {
                    route(STORED_PDF) {
                        respond(body, HttpStatusCode.OK, headersOf(HttpHeaders.ContentLength, body.size.toString()))
                    }
                }
            val (seen, job) = collecting(h)
            runCurrent()

            val fractions = seen.filterIsInstance<PdfCopy.Downloading>().mapNotNull { it.fraction }
            assertTrue(fractions.isNotEmpty(), "progress was reported: $seen")
            assertEquals(fractions.sorted(), fractions)
            assertEquals(PdfCopy.Ready(h.pdfPath(1)), seen.last())
            assertTrue(job.isCompleted.not(), "the flow keeps watching the copy")
            job.cancel()
        }

    @Test
    fun installedCopyBecomesReady() =
        runTest {
            val h = pdf()
            val (seen, job) = collecting(h)
            runCurrent()

            assertEquals(PdfCopy.Ready(h.pdfPath(1)), seen.last())
            assertFalse(checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC)).pinned)
            job.cancel()
        }

    @Test
    fun rowWithoutItsPdfFileDownloadsAgainAndOpens() =
        runTest {
            val h = pdf()
            h.keepCopy()
            h.download.fs.delete(h.download.generationDir(1) / PDF_FILE)
            val (seen, job) = collecting(h)
            runCurrent()

            assertIs<PdfCopy.Downloading>(seen.first())
            assertEquals(PdfCopy.Ready(h.pdfPath(2)), seen.last())
            job.cancel()
        }

    @Test
    fun offlineWithoutACopyFailsWithConnection() =
        runTest {
            val h = pdf()
            h.download.online.value = false
            runCurrent()

            assertEquals(PdfCopy.Failed(PdfCopyFailure.CONNECTION), h.lastState())
        }

    @Test
    fun fullDiskFailsWithNoSpace() =
        runTest {
            val full = FailingWriteFileSystem(FakeFileSystem()) { IOException("write failed: ENOSPC") }
            val h = pdf(fs = full)

            assertEquals(PdfCopy.Failed(PdfCopyFailure.NO_SPACE), h.lastState())
        }

    @Test
    fun removingTheCopyWhileOpenReportsRemoved() =
        runTest {
            val h = pdf()
            h.keepCopy()
            val (seen, job) = collecting(h)
            runCurrent()
            assertEquals(PdfCopy.Ready(h.pdfPath(1)), seen.last())

            h.download.manager.removeFromDevice(DL_SCOPE, DL_DOC)
            runCurrent()

            assertEquals(PdfCopy.Removed, seen.last())
            assertTrue(job.isCompleted)
        }

    @Test
    fun documentIsHeldOnlyWhileCollected() =
        runTest {
            val h = pdf()
            h.keepCopy()
            val (_, job) = collecting(h)
            runCurrent()
            assertTrue(h.download.manager.isHeld(DL_SCOPE, DL_DOC))

            job.cancel()
            runCurrent()

            assertFalse(h.download.manager.isHeld(DL_SCOPE, DL_DOC))
        }
}
