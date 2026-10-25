package app.indelible.core.offline

import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

private const val PDF_BYTES = 200_000
private const val STORED_PDF = "/bucket/doc_1.pdf"
private const val ORIGINAL_UPLOAD = "/api/v1/assets/documents/$DL_DOC/original_upload"

class DownloadPdfTest {
    private val body = ByteArray(PDF_BYTES) { 'x'.code.toByte() }.also { "%PDF-1.7".encodeToByteArray().copyInto(it) }

    private suspend fun TestScope.pdf(
        announced: Long? = null,
        contentLength: Boolean = false,
    ): DownloadHarness =
        downloadHarness {
            documentType = "pdf"
            available = listOf("pdf")
            if (announced != null) sizes = mapOf("pdf" to announced)
            route(STORED_PDF) {
                val length = body.size.toString()
                val headers = if (contentLength) headersOf(HttpHeaders.ContentLength, length) else headersOf()
                respond(body, HttpStatusCode.OK, headers)
            }
        }

    private suspend fun DownloadHarness.fractions(): List<Float> {
        val seen = mutableListOf<Float>()
        assertEquals(FetchResult.Installed, fetcher.download(session, DL_DOC, pin = false) { seen += it })
        return seen
    }

    @Test
    fun pdfDownloadReportsProgressFromContentLength() =
        runTest {
            val seen = pdf(contentLength = true).fractions()

            assertTrue(seen.size > 1, "several steps: $seen")
            assertEquals(seen.sorted(), seen)
            assertEquals(seen.distinct(), seen)
            assertEquals(1f, seen.last())
        }

    @Test
    fun pdfDownloadFallsBackToTheAnnouncedSize() =
        runTest {
            val seen = pdf(announced = PDF_BYTES.toLong()).fractions()

            assertTrue(seen.size > 1, "several steps: $seen")
            assertEquals(seen.sorted(), seen)
            assertEquals(1f, seen.last())
        }

    @Test
    fun pdfDownloadWithoutAnySizeShowsUnknownProgress() =
        runTest {
            val h = pdf()
            val states = mutableListOf<Acquisition?>()
            val board = h.manager.acquisitions.observe(DL_SCOPE)
            backgroundScope.launch { board.collect { states += it[DL_DOC] } }

            h.manager.cacheOnOpen(h.session, DL_DOC)
            runCurrent()

            val downloading = states.filterIsInstance<Acquisition.Downloading>()
            assertEquals(listOf(Acquisition.Downloading(null)), downloading.distinct())
            assertNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
        }

    @Test
    fun progressNeverPassesOne() =
        runTest {
            val seen = pdf(announced = PDF_BYTES / 2L).fractions()

            assertTrue(seen.all { it <= 1f }, "$seen")
            assertEquals(1f, seen.last())
        }

    @Test
    fun pdfDownloadWithALargerAnnouncedSizeStillEndsAtOne() =
        runTest {
            val seen = pdf(announced = PDF_BYTES * 2L).fractions()

            assertEquals(1f, seen.last())
        }

    @Test
    fun pdfKindsPreferPdfThenOriginalUpload() =
        runTest {
            val h =
                downloadHarness {
                    documentType = "pdf"
                    available = listOf("original", "original_upload")
                    route(ORIGINAL_UPLOAD) { respond("%PDF-1.7", HttpStatusCode.OK) }
                }

            h.manager.keepOffline(h.session, DL_DOC)
            runCurrent()

            assertNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertEquals(1, h.server.count(ORIGINAL_UPLOAD))
            assertTrue(h.server.calls.none { it.path.endsWith("/original") }, "original is not a backend kind")
        }
}
