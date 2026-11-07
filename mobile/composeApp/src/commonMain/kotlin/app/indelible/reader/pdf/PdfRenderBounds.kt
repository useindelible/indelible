package app.indelible.reader.pdf

import kotlin.math.floor
import kotlin.math.roundToInt
import kotlin.math.sqrt

const val MAX_RENDER_WIDTH_PX = 3072
const val MAX_RENDER_HEIGHT_PX = 4096
const val MAX_RENDER_PIXELS = 12_000_000L
private const val BYTES_PER_PIXEL = 4L
private const val LETTER_WIDTH = 612.0
private const val LETTER_HEIGHT = 792.0

data class PdfRenderSize(
    val width: Int,
    val height: Int,
)

/** A page the engine could not measure (0x0, NaN or negative) renders as US Letter rather than crashing. */
fun safePageSize(
    width: Double,
    height: Double,
): PdfPageSize {
    val measured = listOf(width, height).all { it.isFinite() && it > 0.0 }
    return if (measured) PdfPageSize(width, height) else PdfPageSize(LETTER_WIDTH, LETTER_HEIGHT)
}

/** The bitmap for [page] at most [layoutWidthPx] wide that stays inside every bound, height from the aspect. */
fun renderSizeFor(
    page: PdfPageSize,
    layoutWidthPx: Int,
): PdfRenderSize {
    val aspect = page.height / page.width
    var width = renderWidthFor(page, layoutWidthPx)
    while (width > 1 && width.toLong() * heightFor(width, aspect) > MAX_RENDER_PIXELS) width--
    return PdfRenderSize(width, heightFor(width, aspect))
}

// A page over 4096 times taller than wide cannot keep its aspect at width 1; the height bound wins over it.
private fun heightFor(
    width: Int,
    aspect: Double,
): Int = (width * aspect).roundToInt().coerceIn(1, MAX_RENDER_HEIGHT_PX)

fun renderWidthFor(
    page: PdfPageSize,
    layoutWidthPx: Int,
): Int {
    val aspect = page.height / page.width
    var width = minOf(layoutWidthPx, MAX_RENDER_WIDTH_PX)
    if ((width * aspect).roundToInt() > MAX_RENDER_HEIGHT_PX) width = floor(MAX_RENDER_HEIGHT_PX / aspect).toInt()
    if (width.toLong() * (width * aspect).roundToInt() > MAX_RENDER_PIXELS) {
        width = floor(sqrt(MAX_RENDER_PIXELS / aspect)).toInt()
    }
    return maxOf(1, width)
}

/** RGBA bytes for a [w] x [h] bitmap; both sides are checked first, so the product always fits an Int. */
fun bufferBytes(
    w: Int,
    h: Int,
): Int {
    require(w in 1..MAX_RENDER_WIDTH_PX && h in 1..MAX_RENDER_HEIGHT_PX) {
        "bitmap $w x $h is outside the render bounds"
    }
    return (w.toLong() * h.toLong() * BYTES_PER_PIXEL).toInt()
}
