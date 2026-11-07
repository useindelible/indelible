@file:OptIn(ExperimentalForeignApi::class, ExperimentalAtomicApi::class)

package app.indelible.reader.pdf

import androidx.compose.ui.graphics.ImageBitmap
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.autoreleasepool
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import platform.Foundation.NSCaseInsensitiveSearch
import platform.PDFKit.PDFDocument
import platform.PDFKit.PDFPage
import platform.PDFKit.PDFSelection
import kotlin.concurrent.atomics.AtomicBoolean
import kotlin.concurrent.atomics.ExperimentalAtomicApi

internal class PdfKitDocument(
    private val document: PDFDocument,
    private val dispatcher: CoroutineDispatcher,
) : PdfDocument {
    private val measured: List<PdfKitPage> =
        List(document.pageCount.toInt()) { index ->
            measure(checkNotNull(document.pageAtIndex(index.toULong())))
        }

    private val closed = AtomicBoolean(false)

    override val pageCount: Int get() = measured.size
    override val pageSizes: List<PdfPageSize> = measured.map { it.size }

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
            val from = toPageSpace(start.clamped(), page.crop, page.rotation)
            val to = toPageSpace(end.clamped(), page.crop, page.rotation)
            val dragged = page.page.selectionFromPoint(from.cgPoint(), toPoint = to.cgPoint())
            val lines = dragged?.lineBounds(page.page).orEmpty()
            val onLines =
                dragged?.let {
                    page.page.selectionFromPoint(from.onLine(lines).cgPoint(), toPoint = to.onLine(lines).cgPoint())
                }
            val chars = onLines?.charRange(page.page)
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

    override fun close() {
        closed.store(true)
    }

    // The dispatcher's worker threads never drain a pool of their own, so each call drains what it autoreleased.
    private suspend fun <T> onPdfKit(block: () -> T): T =
        withContext(dispatcher) {
            check(!closed.load()) { "the PDF document is closed" }
            autoreleasepool(block)
        }

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

    private fun lineRects(
        selection: PDFSelection,
        page: PdfKitPage,
    ): List<NormalizedRect> {
        val rects = selection.lineBounds(page.page).map { fromPageSpace(it, page.crop, page.rotation) }
        return mergeLineRects(rects).mapNotNull(::clipToPage)
    }
}
