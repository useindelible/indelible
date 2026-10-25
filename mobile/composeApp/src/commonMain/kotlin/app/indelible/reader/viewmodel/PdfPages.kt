package app.indelible.reader.viewmodel

import app.indelible.core.offline.PAGE_OFFSET_SCALE
import app.indelible.core.offline.PagePosition
import kotlin.math.floor

/** Where the view scrolls before revealing the pages: a 0-based page and a fraction of its height. */
data class PdfRestoreRequest(
    val pageIndex: Int,
    val pageOffset: Float,
    val nonce: Long,
)

/** One scroll frame: the top [page] (1-based), how far into it, how far through the document. */
data class PdfScrollSample(
    val page: Int,
    val pageOffset: Float,
    val fraction: Float,
    val scrollTopDp: Float,
)

data class PdfResume(
    val at: PagePosition?,
    val percent: Float,
)

private const val PERCENT_MAX = 100f

/** The saved page clamped into the document, with its offset only when the page stood; else the percent's page. */
fun pdfResumeTarget(
    saved: PagePosition?,
    percent: Float,
    pageCount: Int,
    nonce: Long,
): PdfRestoreRequest {
    val last = (pageCount - 1).coerceAtLeast(0)
    if (saved != null) {
        val index = (saved.page - 1).coerceIn(0, last)
        val offset = if (index == saved.page - 1) saved.offset.coerceIn(0, PAGE_OFFSET_SCALE) else 0
        return PdfRestoreRequest(index, offset.toFloat() / PAGE_OFFSET_SCALE, nonce)
    }
    val fraction = (percent / PERCENT_MAX).coerceIn(0f, 1f)
    return PdfRestoreRequest(floor(fraction * pageCount).toInt().coerceIn(0, last), 0f, nonce)
}
