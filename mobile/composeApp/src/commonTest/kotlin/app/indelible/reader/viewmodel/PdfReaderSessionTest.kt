package app.indelible.reader.viewmodel

import app.indelible.core.offline.PagePosition
import app.indelible.core.offline.ReadingPosition
import app.indelible.reader.model.PdfStatus
import app.indelible.reader.pdf.PdfOpenException
import app.indelible.reader.repository.PdfCopy
import app.indelible.reader.repository.PdfCopyFailure
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertSame
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class PdfReaderSessionTest {
    @Test
    fun downloadProgressShowsWhileWaitingForTheCopy() =
        runTest {
            val copies = MutableSharedFlow<PdfCopy>()
            val h = pdfHarness(copy = copies)
            h.session.open()
            advanceUntilIdle()

            copies.emit(PdfCopy.Downloading(0.4f))
            advanceUntilIdle()
            assertEquals(PdfStatus.WAITING_FOR_COPY, h.state.status)
            assertEquals(0.4f, h.state.download)

            copies.emit(PdfCopy.Downloading(null))
            advanceUntilIdle()
            assertNull(h.state.download)
        }

    @Test
    fun copyArrivingOpensTheDocumentAtTheRestore() =
        runTest {
            val h =
                pdfHarness(
                    resumeAt = PagePosition(12, 4200),
                    copy = flowOf(PdfCopy.Downloading(0.9f), PdfCopy.Ready(PDF_PATH)),
                )

            h.session.open()
            advanceUntilIdle()

            assertEquals(PdfStatus.READY, h.state.status)
            assertSame(h.document, h.state.document)
            assertEquals(11, h.state.restore?.pageIndex)
            assertEquals(0.42f, h.state.restore?.pageOffset)
        }

    @Test
    fun unreadableFileIsUnreadable() =
        runTest {
            val h = pdfHarness(opener = { throw PdfOpenException("damaged") })

            h.session.open()
            advanceUntilIdle()

            assertEquals(PdfStatus.UNREADABLE, h.state.status)
            assertNull(h.state.document)
        }

    @Test
    fun connectionFailureOffersRetryAndRetryAsksAgain() =
        runTest {
            val h = pdfHarness(copy = flowOf(PdfCopy.Failed(PdfCopyFailure.CONNECTION)))
            h.session.open()
            advanceUntilIdle()
            assertEquals(PdfStatus.CONNECTION_FAILED, h.state.status)

            h.session.retry()
            advanceUntilIdle()

            assertEquals(2, h.copyRequests)
        }

    @Test
    fun noSpaceIsReported() =
        runTest {
            val h = pdfHarness(copy = flowOf(PdfCopy.Failed(PdfCopyFailure.NO_SPACE)))

            h.session.open()
            advanceUntilIdle()

            assertEquals(PdfStatus.NO_SPACE, h.state.status)
        }

    @Test
    fun serverFailureIsADownloadFailure() =
        runTest {
            val h = pdfHarness(copy = flowOf(PdfCopy.Failed(PdfCopyFailure.SERVER)))

            h.session.open()
            advanceUntilIdle()

            assertEquals(PdfStatus.DOWNLOAD_FAILED, h.state.status)
        }

    @Test
    fun removedCopyClosesTheDocument() =
        runTest {
            val h = pdfHarness(copy = flowOf(PdfCopy.Ready(PDF_PATH), PdfCopy.Removed))

            h.session.open()
            advanceUntilIdle()

            assertEquals(PdfStatus.COPY_REMOVED, h.state.status)
            assertNull(h.state.document)
            assertEquals(1, h.document.closes)
        }

    @Test
    fun removedCopyNeverPublishesReadyWithoutADocument() =
        runTest {
            val h = pdfHarness(copy = flowOf(PdfCopy.Ready(PDF_PATH), PdfCopy.Removed))
            val states = mutableListOf<PdfReaderState>()
            backgroundScope.launch(UnconfinedTestDispatcher(testScheduler)) { h.session.state.toList(states) }

            h.session.open()
            advanceUntilIdle()

            assertFalse(states.isEmpty())
            assertTrue(states.none { it.status == PdfStatus.READY && it.document == null })
            assertEquals(PdfStatus.COPY_REMOVED, states.last().status)
            assertNull(states.last().document)
        }

    @Test
    fun samplesBeforePositionedWriteNothing() =
        runTest {
            val h = pdfHarness()
            h.session.open()
            advanceUntilIdle()

            h.session.onSample(PdfScrollSample(page = 3, pageOffset = 0.25f, fraction = 0.125f, scrollTopDp = 0f))
            advanceUntilIdle()

            assertTrue(h.written.isEmpty())
        }

    @Test
    fun sampleSchedulesPercentAndPagePosition() =
        runTest {
            val h = pdfHarness()
            h.session.open()
            advanceUntilIdle()
            h.session.onPositioned()

            h.session.onSample(PdfScrollSample(page = 3, pageOffset = 0.25f, fraction = 0.125f, scrollTopDp = 0f))
            advanceUntilIdle()

            assertEquals(listOf(12.5f to PagePosition(3, 2500) as ReadingPosition?), h.written)
            assertEquals(12.5f, h.state.percent)
        }

    @Test
    fun captureAndFlushWritesTheLastSample() =
        runTest {
            val h = pdfHarness()
            h.session.open()
            advanceUntilIdle()
            h.session.onPositioned()
            h.session.onSample(PdfScrollSample(page = 1, pageOffset = 0f, fraction = 0f, scrollTopDp = 0f))
            h.session.onSample(PdfScrollSample(page = 7, pageOffset = 0.5f, fraction = 0.325f, scrollTopDp = 0f))
            assertTrue(h.written.isEmpty())

            h.session.captureAndFlush()

            assertEquals(32.5f to PagePosition(7, 5000) as ReadingPosition?, h.written.last())
        }

    @Test
    fun nonFiniteSampleIsIgnored() =
        runTest {
            val h = pdfHarness()
            h.session.open()
            advanceUntilIdle()
            h.session.onPositioned()

            val broken = PdfScrollSample(page = 1, pageOffset = Float.NaN, fraction = Float.NaN, scrollTopDp = 0f)
            h.session.onSample(broken)
            h.session.captureAndFlush()

            assertTrue(h.written.isEmpty())
            assertNull(h.state.percent)
        }

    @Test
    fun closeDuringTheOpenStillClosesTheDocument() =
        runTest {
            val gate = CompletableDeferred<Unit>()
            val document = twentyPages()
            val h =
                pdfHarness(document = document) {
                    gate.await()
                    document
                }
            h.session.open()
            runCurrent()

            h.session.close()
            gate.complete(Unit)
            advanceUntilIdle()

            assertEquals(1, document.closes)
            assertNull(h.state.document)
        }

    @Test
    fun closeClosesTheDocumentOnce() =
        runTest {
            val h = pdfHarness(copy = flow { emit(PdfCopy.Ready(PDF_PATH)) })
            h.session.open()
            advanceUntilIdle()

            h.session.close()
            h.session.close()

            assertEquals(1, h.document.closes)
            assertNull(h.state.document)
        }
}
