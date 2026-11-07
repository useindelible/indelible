package app.indelible.reader.pdf

import android.graphics.Bitmap
import android.graphics.Color
import android.graphics.RectF
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import io.legere.pdfiumandroid.suspend.PdfDocumentKt
import io.legere.pdfiumandroid.suspend.PdfPageKt
import io.legere.pdfiumandroid.suspend.PdfTextPageKt
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlin.math.abs

private const val JOINED_HYPHEN = '\u0002'
private const val QUARTER_TURNS = 4
private const val DEGREES_PER_TURN = 90
private const val HALF_TURN = 180
private const val NO_CHAR = -1
private const val HIT_TOLERANCE = 8.0
private const val WIDE_HIT_TOLERANCE = 24.0
private const val LINE_TOLERANCE = 16.0
private const val OPEN_PAGES = 6
private const val TONE_STRIP_ROWS = 64

/** A page's geometry, read once when the document opens. */
private class MeasuredPage(
    val box: PageSpaceRect,
    val rotation: Int,
    val size: PdfPageSize,
)

/** One pdfium document; pages are measured once at open and reopened on demand, a few at a time. */
internal class PdfiumDocument private constructor(
    private val document: PdfDocumentKt,
    private val pages: List<MeasuredPage>,
    private val dispatcher: CoroutineDispatcher,
) : PdfDocument {
    // Most recently used last; only touched on the dispatcher or inside close().
    private val open = LinkedHashMap<Int, OpenPage>()

    @Volatile
    private var closed = false

    override val pageCount: Int get() = pages.size
    override val pageSizes: List<PdfPageSize> get() = pages.map { it.size }

    override suspend fun render(
        pageIndex: Int,
        widthPx: Int,
        tone: PdfTone,
    ): ImageBitmap =
        onPdfium {
            val (w, h) = renderSizeFor(pages[pageIndex].size, widthPx)
            bufferBytes(w, h)
            val bitmap = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
            page(pageIndex).page.renderPageBitmap(bitmap, 0, 0, w, h, renderAnnot = true, canvasColor = Color.WHITE)
            toneTable(tone)?.let { applyToneInStrips(bitmap, it) }
            bitmap.asImageBitmap()
        }

    override suspend fun text(pageIndex: Int): String = onPdfium { page(pageIndex).chars() }

    override suspend fun select(
        pageIndex: Int,
        start: NormalizedPoint,
        end: NormalizedPoint,
    ): PdfSelection? =
        onPdfium {
            val opened = page(pageIndex)
            val text = opened.chars()
            val first = opened.charAt(start)
            val last = opened.charAt(end)
            val range =
                when {
                    text.isEmpty() || (first == NO_CHAR && last == NO_CHAR) -> null
                    first == NO_CHAR -> opened.marginRange(text, last, start)
                    last == NO_CHAR -> opened.marginRange(text, first, end)
                    else -> minOf(first, last)..maxOf(first, last)
                }?.let { snapToWords(text, it.first, it.last + 1) }
            val selected = range?.takeUnless { it.isEmpty() }?.let { text.substring(it.first, it.last + 1) }
            if (range == null || selected.isNullOrBlank()) {
                null
            } else {
                val rects = opened.rects(range.first, range.last + 1)
                PdfSelection(pageIndex, selected, range.first, range.last + 1, rects)
            }
        }

    override suspend fun rectsForRange(
        pageIndex: Int,
        startChar: Int,
        endChar: Int,
    ): List<NormalizedRect> = onPdfium { page(pageIndex).rects(startChar, endChar) }

    override suspend fun search(query: String): List<PdfHit> =
        onPdfium {
            pages.indices.flatMap { index ->
                val opened = page(index)
                val hits = mutableListOf<PdfHit>()
                opened.textPage().findStart(query, emptySet(), 0)?.use { find ->
                    while (find.findNext()) {
                        val start = find.getSchResultIndex()
                        val end = start + find.getSchCount()
                        hits += PdfHit(index, start, end, opened.rects(start, end))
                    }
                }
                hits
            }
        }

    override fun close() {
        val held =
            synchronized(open) {
                if (closed) return
                closed = true
                open.values.toList().also { open.clear() }
            }
        // pdfium serialises every native call, so closing here would make the caller wait for a render in flight.
        CoroutineScope(dispatcher).launch {
            held.forEach { it.close() }
            document.safeClose()
        }
    }

    private suspend fun <T> onPdfium(block: suspend () -> T): T =
        withContext(dispatcher) {
            check(!closed) { "the PDF document is closed" }
            block()
        }

    /** The open page for [index], reopened when it was let go; the least recently used page is closed first. */
    private suspend fun page(index: Int): OpenPage {
        val kept = synchronized(open) { open.remove(index)?.also { open[index] = it } }
        if (kept != null) return kept
        val opened = OpenPage(document.openPage(index), pages[index])
        val evicted =
            synchronized(open) {
                if (closed) return@synchronized opened
                open[index] = opened
                if (open.size > OPEN_PAGES) open.remove(open.keys.first()) else null
            }
        evicted?.close()
        check(!closed) { "the PDF document is closed" }
        return opened
    }

    companion object {
        suspend fun open(
            document: PdfDocumentKt,
            dispatcher: CoroutineDispatcher,
        ): PdfiumDocument {
            val count = document.getPageCount()
            if (count <= 0) throw PdfOpenException("the PDF has no pages")
            val pages = List(count) { index -> document.openPage(index).use { measure(it) } }
            return PdfiumDocument(document, pages, dispatcher)
        }

        private suspend fun measure(page: PdfPageKt): MeasuredPage {
            val box = page.getPageBoundingBox().toPageSpace()
            val rotation = page.getPageRotation().toDegrees()
            val swapped = rotation % HALF_TURN == DEGREES_PER_TURN
            val size = if (swapped) safePageSize(box.height, box.width) else safePageSize(box.width, box.height)
            return MeasuredPage(box, rotation, size)
        }

        private fun Int.toDegrees(): Int = if (this in 0 until QUARTER_TURNS) this * DEGREES_PER_TURN else 0
    }
}

/** A pdfium page handle with its text page and characters, opened the first time they are read. */
private class OpenPage(
    val page: PdfPageKt,
    private val measured: MeasuredPage,
) {
    private var text: PdfTextPageKt? = null
    private var chars: String? = null

    fun close() {
        text?.safeClose()
        page.safeClose()
    }

    suspend fun textPage(): PdfTextPageKt = text ?: page.openTextPage().also { text = it }

    // Read one character at a time: the bulk text call omits glyphs without unicode and its binding copies
    // past the returned length, while char indices and string offsets must stay one to one.
    // pdfium marks a line-end hyphen it joined to the next line as U+0002; the hyphen is what the web shows.
    suspend fun chars(): String {
        val known = chars
        if (known != null) return known
        val text = textPage()
        val count = text.textPageCountChars()
        val builder = StringBuilder(count)
        for (index in 0 until count) {
            builder.append(
                when (val unicode = text.textPageGetUnicode(index)) {
                    '\u0000' -> '\uFFFD'
                    JOINED_HYPHEN -> '-'
                    else -> unicode
                },
            )
        }
        return builder.toString().also { chars = it }
    }

    /** The character under [point]; a point in the margin beside a line takes that line's nearest character. */
    suspend fun charAt(point: NormalizedPoint): Int {
        val clamped = NormalizedPoint(point.x.coerceIn(0.0, 1.0), point.y.coerceIn(0.0, 1.0))
        val at = toPageSpace(clamped, measured.box, measured.rotation)
        val text = textPage()
        val tolerances = listOf(HIT_TOLERANCE to HIT_TOLERANCE, WIDE_HIT_TOLERANCE to WIDE_HIT_TOLERANCE)
        for ((x, y) in tolerances + (2 * measured.box.width to LINE_TOLERANCE)) {
            val index = text.textPageGetCharIndexAtPos(at.x, at.y, x, y)
            if (index >= 0) return index
        }
        return NO_CHAR
    }

    // A drag into the blank margin keeps the text from the hit to the end of the page on the margin's side.
    suspend fun marginRange(
        text: String,
        hit: Int,
        margin: NormalizedPoint,
    ): IntRange {
        val box = textPage().textPageGetCharBox(hit)?.toPageSpace()?.let(::display)
        val after = box == null || margin.y > box.y + box.height / 2
        return if (after) hit until text.length else 0..hit
    }

    suspend fun rects(
        startChar: Int,
        endChar: Int,
    ): List<NormalizedRect> {
        val text = textPage()
        val start = startChar.coerceIn(0, text.textPageCountChars())
        val count = endChar.coerceIn(start, text.textPageCountChars()) - start
        if (count == 0) return emptyList()
        val rects = List(text.textPageCountRects(start, count)) { text.textPageGetRect(it)?.toPageSpace() }
        return mergeLineRects(rects.filterNotNull().map(::display)).mapNotNull(::clipToPage)
    }

    private fun display(rect: PageSpaceRect) = fromPageSpace(rect, measured.box, measured.rotation)
}

private fun applyToneInStrips(
    bitmap: Bitmap,
    table: ByteArray,
) {
    val strip = IntArray(bitmap.width * TONE_STRIP_ROWS)
    var top = 0
    while (top < bitmap.height) {
        val rows = minOf(TONE_STRIP_ROWS, bitmap.height - top)
        val pixels = if (rows == TONE_STRIP_ROWS) strip else IntArray(bitmap.width * rows)
        bitmap.getPixels(pixels, 0, bitmap.width, 0, top, bitmap.width, rows)
        applyToneArgb(pixels, table)
        bitmap.setPixels(pixels, 0, bitmap.width, 0, top, bitmap.width, rows)
        top += rows
    }
}

// pdfium rects are y-up page points with top above bottom.
private fun RectF.toPageSpace(): PageSpaceRect =
    PageSpaceRect(
        minOf(left, right).toDouble(),
        minOf(top, bottom).toDouble(),
        abs(right - left).toDouble(),
        abs(top - bottom).toDouble(),
    )
