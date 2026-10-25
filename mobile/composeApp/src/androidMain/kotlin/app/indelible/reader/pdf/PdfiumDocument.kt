package app.indelible.reader.pdf

import android.graphics.Bitmap
import android.graphics.Color
import android.graphics.RectF
import android.os.ParcelFileDescriptor
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import io.legere.pdfiumandroid.suspend.PdfDocumentKt
import io.legere.pdfiumandroid.suspend.PdfPageKt
import io.legere.pdfiumandroid.suspend.PdfTextPageKt
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import kotlin.math.abs

private const val QUARTER_TURNS = 4
private const val DEGREES_PER_TURN = 90
private const val FULL_TURN = 360
private const val HALF_TURN = 180
private const val NO_CHAR = -1
private const val HIT_TOLERANCE = 8.0
private const val WIDE_HIT_TOLERANCE = 24.0

/** One pdfium document; pages are opened on construction, text pages the first time a page is read. */
internal class PdfiumDocument private constructor(
    private val document: PdfDocumentKt,
    private val descriptor: ParcelFileDescriptor,
    private val pages: List<Page>,
    private val dispatcher: CoroutineDispatcher,
) : PdfDocument {
    private class Page(
        val page: PdfPageKt,
        val box: PageSpaceRect,
        val rotation: Int,
        val size: PdfPageSize,
    ) {
        var text: PdfTextPageKt? = null
        var chars: String? = null
    }

    override val pageCount: Int get() = pages.size
    override val pageSizes: List<PdfPageSize> get() = pages.map { it.size }

    override suspend fun render(
        pageIndex: Int,
        widthPx: Int,
        tone: PdfTone,
    ): ImageBitmap =
        withContext(dispatcher) {
            val page = pages[pageIndex]
            val (w, h) = renderSizeFor(page.size, widthPx)
            val bitmap = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
            page.page.renderPageBitmap(bitmap, 0, 0, w, h, renderAnnot = true, canvasColor = Color.WHITE)
            toneTable(tone)?.let { table ->
                val pixels = IntArray(w * h)
                bitmap.getPixels(pixels, 0, w, 0, 0, w, h)
                applyToneArgb(pixels, table)
                bitmap.setPixels(pixels, 0, w, 0, 0, w, h)
            }
            bitmap.asImageBitmap()
        }

    override suspend fun text(pageIndex: Int): String = withContext(dispatcher) { pages[pageIndex].chars() }

    override suspend fun select(
        pageIndex: Int,
        start: NormalizedPoint,
        end: NormalizedPoint,
    ): PdfSelection? =
        withContext(dispatcher) {
            val page = pages[pageIndex]
            val text = page.chars()
            val first = page.charAt(start)
            val last = page.charAt(end)
            val range =
                when {
                    text.isEmpty() || (first == NO_CHAR && last == NO_CHAR) -> null
                    first == NO_CHAR -> marginRange(text, last, start, end)
                    last == NO_CHAR -> marginRange(text, first, end, start)
                    else -> minOf(first, last)..maxOf(first, last)
                }?.let { snapToWords(text, it.first, it.last + 1) }
            val selected = range?.let { text.substring(it.first, it.last + 1) }?.takeIf { it.isNotBlank() }
            if (range == null || selected == null) {
                null
            } else {
                PdfSelection(pageIndex, selected, range.first, range.last + 1, page.rects(range.first, range.last + 1))
            }
        }

    override suspend fun rectsForRange(
        pageIndex: Int,
        startChar: Int,
        endChar: Int,
    ): List<NormalizedRect> = withContext(dispatcher) { pages[pageIndex].rects(startChar, endChar) }

    override suspend fun search(query: String): List<PdfHit> =
        withContext(dispatcher) {
            pages.flatMapIndexed { index, page ->
                val hits = mutableListOf<PdfHit>()
                val find = page.textPage().findStart(query, emptySet(), 0)
                if (find != null) {
                    while (find.findNext()) {
                        val start = find.getSchResultIndex()
                        val end = start + find.getSchCount()
                        hits += PdfHit(index, start, end, page.rects(start, end))
                    }
                    find.closeFind()
                }
                hits
            }
        }

    override fun close() {
        pages.forEach { page ->
            page.text?.close()
            page.page.close()
        }
        document.safeClose()
        descriptor.close()
    }

    // A drag that ends in the blank margin keeps the text between the hit and the page end on that side.
    private fun marginRange(
        text: String,
        hit: Int,
        margin: NormalizedPoint,
        other: NormalizedPoint,
    ): IntRange {
        val after = margin.y > other.y || (margin.y == other.y && margin.x > other.x)
        return if (after) hit until text.length else 0..hit
    }

    private suspend fun Page.textPage(): PdfTextPageKt = text ?: page.openTextPage().also { text = it }

    private suspend fun Page.chars(): String =
        chars ?: textPage().let { it.textPageGetText(0, it.textPageCountChars()).orEmpty() }.also { chars = it }

    private suspend fun Page.charAt(point: NormalizedPoint): Int {
        val at = toPageSpace(point.clamped(), box, rotation)
        val text = textPage()
        val near = text.textPageGetCharIndexAtPos(at.x, at.y, HIT_TOLERANCE, HIT_TOLERANCE)
        if (near >= 0) return near
        val wide = text.textPageGetCharIndexAtPos(at.x, at.y, WIDE_HIT_TOLERANCE, WIDE_HIT_TOLERANCE)
        return if (wide >= 0) wide else NO_CHAR
    }

    private suspend fun Page.rects(
        startChar: Int,
        endChar: Int,
    ): List<NormalizedRect> {
        val count = textPage().textPageCountRects(startChar, (endChar - startChar).coerceAtLeast(0))
        val rects = (0 until count).mapNotNull { textPage().textPageGetRect(it)?.toPageSpace() }
        return mergeLineRects(rects.map { fromPageSpace(it, box, rotation) }).mapNotNull(::clipToPage)
    }

    private fun NormalizedPoint.clamped() = NormalizedPoint(x.coerceIn(0.0, 1.0), y.coerceIn(0.0, 1.0))

    companion object {
        suspend fun open(
            document: PdfDocumentKt,
            descriptor: ParcelFileDescriptor,
            dispatcher: CoroutineDispatcher,
        ): PdfiumDocument {
            val pages = List(document.getPageCount()) { index -> measure(document.openPage(index)) }
            return PdfiumDocument(document, descriptor, pages, dispatcher)
        }

        private suspend fun measure(page: PdfPageKt): Page {
            val box = page.getPageBoundingBox().toPageSpace()
            val rotation = page.getPageRotation().toDegrees()
            val swapped = rotation % HALF_TURN == DEGREES_PER_TURN
            val size = if (swapped) safePageSize(box.height, box.width) else safePageSize(box.width, box.height)
            return Page(page, box, rotation, size)
        }

        // pdfium reports quarter turns (0..3); anything else is taken as degrees.
        private fun Int.toDegrees(): Int {
            val degrees = if (this in 0 until QUARTER_TURNS) this * DEGREES_PER_TURN else this
            return ((degrees % FULL_TURN) + FULL_TURN) % FULL_TURN
        }

        // pdfium rects are y-up page points with top above bottom.
        private fun RectF.toPageSpace(): PageSpaceRect =
            PageSpaceRect(
                minOf(left, right).toDouble(),
                minOf(top, bottom).toDouble(),
                abs(right - left).toDouble(),
                abs(top - bottom).toDouble(),
            )
    }
}
