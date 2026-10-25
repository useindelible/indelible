package app.indelible.reader.pdf

import androidx.compose.ui.graphics.ImageBitmap

expect object PdfEngine {
    suspend fun open(path: String): PdfDocument
}

/** One open PDF; calls are serialised by the engine; every page index is 0-based. */
interface PdfDocument : AutoCloseable {
    val pageCount: Int

    /** Points, after the crop box and /Rotate are applied (display orientation). */
    val pageSizes: List<PdfPageSize>

    /** A bitmap [widthPx] wide, height from the page's aspect, with [tone] applied. */
    suspend fun render(
        pageIndex: Int,
        widthPx: Int,
        tone: PdfTone,
    ): ImageBitmap

    suspend fun text(pageIndex: Int): String

    /** The text between two drag points; null when nothing is selected. */
    suspend fun select(
        pageIndex: Int,
        start: NormalizedPoint,
        end: NormalizedPoint,
    ): PdfSelection?

    suspend fun rectsForRange(
        pageIndex: Int,
        startChar: Int,
        endChar: Int,
    ): List<NormalizedRect>

    suspend fun search(query: String): List<PdfHit>

    override fun close()
}

data class PdfPageSize(
    val width: Double,
    val height: Double,
)

/** Fractions of the rendered page box, origin top-left, y down. */
data class NormalizedPoint(
    val x: Double,
    val y: Double,
)

data class NormalizedRect(
    val x: Double,
    val y: Double,
    val width: Double,
    val height: Double,
)

/** [rects] are line rects already merged; [endChar] is exclusive. */
data class PdfSelection(
    val pageIndex: Int,
    val text: String,
    val startChar: Int,
    val endChar: Int,
    val rects: List<NormalizedRect>,
)

data class PdfHit(
    val pageIndex: Int,
    val startChar: Int,
    val endChar: Int,
    val rects: List<NormalizedRect>,
)

enum class PdfTone { NONE, SEPIA, DARK }

class PdfOpenException(
    message: String,
    cause: Throwable? = null,
) : Exception(message, cause)
