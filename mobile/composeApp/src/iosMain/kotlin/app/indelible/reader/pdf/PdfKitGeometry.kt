@file:OptIn(ExperimentalForeignApi::class)

package app.indelible.reader.pdf

import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.useContents
import platform.CoreGraphics.CGPointMake
import platform.Foundation.NSMakeRange
import platform.Foundation.NSNotFound
import platform.PDFKit.PDFPage
import platform.PDFKit.PDFSelection
import platform.PDFKit.kPDFDisplayBoxCropBox
import kotlin.math.abs

private const val FULL_TURN = 360
private const val HALF_TURN = 180
private const val QUARTER_TURN = 90
private const val LINE_TOLERANCE = 16.0
private const val EDGE_INSET = 0.5

internal class PdfKitPage(
    val page: PDFPage,
    val crop: PageSpaceRect,
    val rotation: Int,
    val size: PdfPageSize,
) {
    val text: String by lazy { page.string.orEmpty() }
}

internal fun measure(page: PDFPage): PdfKitPage {
    val crop =
        page.boundsForBox(kPDFDisplayBoxCropBox).useContents {
            PageSpaceRect(origin.x, origin.y, size.width, size.height)
        }
    val rotation = ((page.rotation.toInt() % FULL_TURN) + FULL_TURN) % FULL_TURN
    val swapped = rotation % HALF_TURN == QUARTER_TURN
    val size = if (swapped) safePageSize(crop.height, crop.width) else safePageSize(crop.width, crop.height)
    return PdfKitPage(page, crop, rotation, size)
}

internal fun PDFSelection.lineBounds(page: PDFPage): List<PageSpaceRect> =
    selectionsByLine().filterIsInstance<PDFSelection>().map { line ->
        line.boundsForPage(page).useContents { PageSpaceRect(origin.x, origin.y, size.width, size.height) }
    }

// PDFKit runs a drag that ends in the margin to the edge of the page; the line beside the point is the limit.
internal fun PageSpacePoint.onLine(lines: List<PageSpaceRect>): PageSpacePoint {
    val beside = lines.filter { y in it.y - LINE_TOLERANCE / 2..it.y + it.height + LINE_TOLERANCE / 2 }
    val inside = beside.any { x in it.x..it.x + it.width }
    val line = beside.minByOrNull { minOf(abs(x - it.x), abs(x - it.x - it.width)) }
    if (inside || line == null) return this
    val inset = minOf(EDGE_INSET, line.width / 2)
    return PageSpacePoint(x.coerceIn(line.x + inset, line.x + line.width - inset), y)
}

// The min location and max end over PDFKit's ranges; NSNotFound is positive, so it is checked by value.
internal fun PDFSelection.charRange(page: PDFPage): IntRange? {
    var start = Int.MAX_VALUE
    var end = -1
    for (i in 0 until numberOfTextRangesOnPage(page).toInt()) {
        rangeAtIndex(i.toULong(), onPage = page).useContents {
            if (location.toLong() != NSNotFound && length > 0uL) {
                start = minOf(start, location.toInt())
                end = maxOf(end, (location + length).toInt())
            }
        }
    }
    return if (end > start) start until end else null
}

internal fun IntRange.nsRange() = NSMakeRange(first.toULong(), (last + 1 - first).toULong())

internal fun PageSpacePoint.cgPoint() = CGPointMake(x, y)

internal fun NormalizedPoint.clamped() = NormalizedPoint(x.coerceIn(0.0, 1.0), y.coerceIn(0.0, 1.0))
