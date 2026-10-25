package app.indelible.reader.pdf

import kotlin.math.floor
import kotlin.math.roundToInt
import kotlin.math.sqrt

const val MAX_RENDER_WIDTH_PX = 3072
const val MAX_RENDER_HEIGHT_PX = 4096
const val MAX_RENDER_PIXELS = 12_000_000L
private const val BYTES_PER_PIXEL = 4L

/** The widest render of [page] that stays inside every bound; never above [layoutWidthPx]. */
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
