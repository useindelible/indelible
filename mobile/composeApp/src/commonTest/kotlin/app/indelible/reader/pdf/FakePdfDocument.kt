package app.indelible.reader.pdf

import androidx.compose.ui.graphics.ImageBitmap
import kotlin.math.roundToInt

/** A document of [pageSizes] whose selections and hits are scripted; [renders] and [closes] record the calls. */
class FakePdfDocument(
    override val pageSizes: List<PdfPageSize> = List(3) { PdfPageSize(612.0, 792.0) },
    private val texts: List<String> = pageSizes.map { "Page text" },
    private val selectionFor: suspend (Int, NormalizedPoint, NormalizedPoint) -> PdfSelection? = { _, _, _ -> null },
    private val hits: List<PdfHit> = emptyList(),
) : PdfDocument {
    override val pageCount: Int get() = pageSizes.size
    val renders = mutableListOf<Triple<Int, Int, PdfTone>>()
    var closes = 0
        private set
    var beforeRender: (suspend () -> Unit)? = null

    override suspend fun render(
        pageIndex: Int,
        widthPx: Int,
        tone: PdfTone,
    ): ImageBitmap {
        renders += Triple(pageIndex, widthPx, tone)
        beforeRender?.invoke()
        val size = pageSizes[pageIndex]
        return ImageBitmap(widthPx, (widthPx * size.height / size.width).roundToInt())
    }

    override suspend fun text(pageIndex: Int): String = texts[pageIndex]

    override suspend fun select(
        pageIndex: Int,
        start: NormalizedPoint,
        end: NormalizedPoint,
    ): PdfSelection? = selectionFor(pageIndex, start, end)

    override suspend fun rectsForRange(
        pageIndex: Int,
        startChar: Int,
        endChar: Int,
    ): List<NormalizedRect> = hits.filter { it.pageIndex == pageIndex }.flatMap { it.rects }

    override suspend fun search(query: String): List<PdfHit> = hits

    override fun close() {
        closes++
    }
}
