package app.indelible.reader.ui.pdf

import app.indelible.reader.pdf.FakePdfDocument
import app.indelible.reader.pdf.PdfPageSize
import app.indelible.reader.pdf.PdfTone
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

class PdfPageBitmapsTest {
    private val document = FakePdfDocument(List(4) { PdfPageSize(100.0, 100.0) })
    private val pageBytes = 100L * 100L * 4L

    @Test
    fun samePageAndWidthRendersOnce() =
        runTest {
            val bitmaps = PdfPageBitmaps(document)

            val first = bitmaps.bitmap(0, 100, PdfTone.NONE)
            val again = bitmaps.bitmap(0, 100, PdfTone.NONE)

            assertEquals(1, document.renders.size)
            assertEquals(first, again)
        }

    @Test
    fun aToneChangeRendersAgain() =
        runTest {
            val bitmaps = PdfPageBitmaps(document)

            bitmaps.bitmap(0, 100, PdfTone.NONE)
            bitmaps.bitmap(0, 100, PdfTone.DARK)

            assertEquals(listOf(PdfTone.NONE, PdfTone.DARK), document.renders.map { it.third })
        }

    @Test
    fun budgetEvictsTheLeastRecentlyUsed() =
        runTest {
            val bitmaps = PdfPageBitmaps(document, budgetBytes = pageBytes * 2)

            bitmaps.bitmap(0, 100, PdfTone.NONE)
            bitmaps.bitmap(1, 100, PdfTone.NONE)
            bitmaps.bitmap(0, 100, PdfTone.NONE)
            bitmaps.bitmap(2, 100, PdfTone.NONE)
            bitmaps.bitmap(0, 100, PdfTone.NONE)
            bitmaps.bitmap(1, 100, PdfTone.NONE)

            assertEquals(listOf(0, 1, 2, 1), document.renders.map { it.first })
        }

    @Test
    fun twoMissesOnOneKeyCountItsBytesOnce() =
        runTest {
            val gate = CompletableDeferred<Unit>()
            document.beforeRender = { gate.await() }
            val bitmaps = PdfPageBitmaps(document, budgetBytes = pageBytes * 2)

            val racing = List(2) { async { bitmaps.bitmap(0, 100, PdfTone.NONE) } }
            runCurrent()
            gate.complete(Unit)
            racing.awaitAll()
            bitmaps.bitmap(1, 100, PdfTone.NONE)
            bitmaps.bitmap(0, 100, PdfTone.NONE)

            assertEquals(2, document.renders.count { it.first == 0 })
        }
}
