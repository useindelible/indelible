@file:OptIn(ExperimentalForeignApi::class)

package app.indelible.reader.pdf

import androidx.compose.ui.graphics.ImageBitmap
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.autoreleasepool
import kotlinx.cinterop.useContents
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import platform.CoreGraphics.CGPointMake
import platform.Foundation.NSCaseInsensitiveSearch
import platform.Foundation.NSMakeRange
import platform.Foundation.NSNotFound
import platform.PDFKit.PDFDocument
import platform.PDFKit.PDFPage
import platform.PDFKit.PDFSelection
import platform.PDFKit.kPDFDisplayBoxCropBox

private const val FULL_TURN = 360
private const val HALF_TURN = 180
private const val QUARTER_TURN = 90

internal class PdfKitDocument(
    private val document: PDFDocument,
    private val dispatcher: CoroutineDispatcher,
) : PdfDocument {
    private class Page(
        val page: PDFPage,
        val crop: PageSpaceRect,
        val rotation: Int,
        val size: PdfPageSize,
    ) {
        val text: String by lazy { page.string.orEmpty() }
    }

    private val measured: List<Page> =
        List(document.pageCount.toInt()) { index ->
            measure(checkNotNull(document.pageAtIndex(index.toULong())))
        }

    override val pageCount: Int get() = measured.size
    override val pageSizes: List<PdfPageSize> get() = measured.map { it.size }

    override suspend fun render(
        pageIndex: Int,
        widthPx: Int,
        tone: PdfTone,
    ): ImageBitmap =
        onPdfKit {
            val page = measured[pageIndex]
            renderPage(page.page, page.size, widthPx, tone)
        }

    override suspend fun text(pageIndex: Int): String = onPdfKit { measured[pageIndex].text }

    override suspend fun select(
        pageIndex: Int,
        start: NormalizedPoint,
        end: NormalizedPoint,
    ): PdfSelection? =
        onPdfKit {
            val page = measured[pageIndex]
            val from = toPageSpace(start.clamped(), page.crop, page.rotation).cgPoint()
            val to = toPageSpace(end.clamped(), page.crop, page.rotation).cgPoint()
            val dragged = page.page.selectionFromPoint(from, toPoint = to)
            val chars = dragged?.charRange(page.page)
            val range = chars?.let { snapToWords(page.text, it.first, it.last + 1) }
            val selection = range?.takeUnless { it.isEmpty() }?.let { page.page.selectionForRange(it.nsRange()) }
            val text = selection?.string?.takeIf { it.isNotBlank() }
            if (selection == null || range == null || text == null) {
                null
            } else {
                PdfSelection(pageIndex, text, range.first, range.last + 1, lineRects(selection, page))
            }
        }

    override suspend fun rectsForRange(
        pageIndex: Int,
        startChar: Int,
        endChar: Int,
    ): List<NormalizedRect> =
        onPdfKit {
            val page = measured[pageIndex]
            val start = startChar.coerceIn(0, page.text.length)
            val range = start until endChar.coerceIn(start, page.text.length)
            val selection = if (range.isEmpty()) null else page.page.selectionForRange(range.nsRange())
            selection?.let { lineRects(it, page) }.orEmpty()
        }

    override suspend fun search(query: String): List<PdfHit> =
        onPdfKit {
            document
                .findString(query, withOptions = NSCaseInsensitiveSearch)
                .filterIsInstance<PDFSelection>()
                .mapNotNull { hit -> hit.toHit() }
        }

    override fun close() = Unit

    // The dispatcher's worker threads never drain a pool of their own, so each call drains what it autoreleased.
    private suspend fun <T> onPdfKit(block: () -> T): T = withContext(dispatcher) { autoreleasepool(block) }

    private fun PDFSelection.toHit(): PdfHit? {
        val pdfPage = pages.firstOrNull() as? PDFPage
        val index = pdfPage?.let { document.indexForPage(it).toInt() }
        val page = index?.let(measured::getOrNull)
        val range = pdfPage?.let { charRange(it) }
        return if (index == null || page == null || range == null) {
            null
        } else {
            PdfHit(index, range.first, range.last + 1, lineRects(this, page))
        }
    }

    private fun measure(page: PDFPage): Page {
        val crop =
            page.boundsForBox(kPDFDisplayBoxCropBox).useContents {
                PageSpaceRect(origin.x, origin.y, size.width, size.height)
            }
        val rotation = ((page.rotation.toInt() % FULL_TURN) + FULL_TURN) % FULL_TURN
        val swapped = rotation % HALF_TURN == QUARTER_TURN
        val size = if (swapped) safePageSize(crop.height, crop.width) else safePageSize(crop.width, crop.height)
        return Page(page, crop, rotation, size)
    }

    private fun lineRects(
        selection: PDFSelection,
        page: Page,
    ): List<NormalizedRect> {
        val rects =
            selection.selectionsByLine().filterIsInstance<PDFSelection>().map { line ->
                line.boundsForPage(page.page).useContents {
                    fromPageSpace(PageSpaceRect(origin.x, origin.y, size.width, size.height), page.crop, page.rotation)
                }
            }
        return mergeLineRects(rects).mapNotNull(::clipToPage)
    }

    // The min location and max end over PDFKit's ranges; NSNotFound is positive, so it is checked by value.
    private fun PDFSelection.charRange(page: PDFPage): IntRange? {
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

    private fun IntRange.nsRange() = NSMakeRange(first.toULong(), (last + 1 - first).toULong())

    private fun PageSpacePoint.cgPoint() = CGPointMake(x, y)

    private fun NormalizedPoint.clamped() = NormalizedPoint(x.coerceIn(0.0, 1.0), y.coerceIn(0.0, 1.0))
}
