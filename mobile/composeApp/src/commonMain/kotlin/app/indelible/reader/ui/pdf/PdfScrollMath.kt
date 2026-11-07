package app.indelible.reader.ui.pdf

import app.indelible.reader.pdf.PdfPageSize
import app.indelible.reader.viewmodel.PdfScrollSample
import kotlin.math.roundToInt

/** The page under the top edge of the content: its 0-based index, the px of it above that edge, its laid-out height. */
data class PdfTopItem(
    val index: Int,
    val offsetPx: Int,
    val heightPx: Int,
)

fun pageHeightPx(
    page: PdfPageSize,
    widthPx: Int,
): Int = (widthPx * page.height / page.width).roundToInt().coerceAtLeast(1)

fun sampleOf(
    top: PdfTopItem,
    pageCount: Int,
    lastPageFullyShown: Boolean,
    scrollTopDp: Float,
): PdfScrollSample {
    val pageOffset = (top.offsetPx.toFloat() / top.heightPx.coerceAtLeast(1)).coerceIn(0f, 1f)
    val through = (top.index + pageOffset) / pageCount.coerceAtLeast(1)
    val fraction = if (lastPageFullyShown) 1f else through.coerceIn(0f, 1f)
    return PdfScrollSample(page = top.index + 1, pageOffset, fraction, scrollTopDp)
}

fun restoreOffsetPx(
    pageOffset: Float,
    pageHeightPx: Int,
): Int = (pageOffset.coerceIn(0f, 1f) * pageHeightPx).roundToInt()

/** The scroll top the reader chrome follows: it moves with the reader's own scrolling, never with a zoom commit. */
internal class PdfScrollTravel {
    private var last: Float? = null
    private var travel = 0f
    private var holding = false
    private var skipNext = false

    /** Until [release], the list moves only because a zoom is being laid out. */
    fun hold() {
        holding = true
    }

    /** The frame that lands the zoom may arrive after this, so it does not count either. */
    fun release() {
        holding = false
        skipNext = true
    }

    fun next(rawTopDp: Float): Float {
        val previous = last
        last = rawTopDp
        when {
            previous == null -> travel = rawTopDp
            holding -> Unit
            skipNext -> skipNext = false
            else -> travel += rawTopDp - previous
        }
        return travel
    }
}
