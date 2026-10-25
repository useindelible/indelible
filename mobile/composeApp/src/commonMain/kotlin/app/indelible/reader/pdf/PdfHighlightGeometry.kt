package app.indelible.reader.pdf

import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightLocator
import app.indelible.reader.model.PdfHighlightTarget
import app.indelible.reader.model.PdfRect

private const val SNAPSHOT_LENGTH = 100
private const val PDF_LOCATOR = "pdf"
private const val BOOKMARK = "bookmark"
private const val BOX_SIDES = 4

/** The highlight the backend accepts for [selection]: rects clipped to the page, then their clipped union. */
fun pdfHighlightTarget(selection: PdfSelection): PdfHighlightTarget? {
    val text = selection.text.trim().takeIf { it.isNotEmpty() }
    val rects = selection.rects.mapNotNull(::clipToPage)
    val box = unionOf(rects)?.let(::clipToPage)
    return if (text == null || box == null) {
        null
    } else {
        PdfHighlightTarget(
            page = selection.pageIndex + 1,
            x = box.x,
            y = box.y,
            width = box.width,
            height = box.height,
            textSnapshot = text.take(SNAPSHOT_LENGTH),
            rects = rects.map { PdfRect(it.x, it.y, it.width, it.height) },
        )
    }
}

data class PdfHighlightBox(
    val highlightId: String,
    val color: String,
    val rects: List<NormalizedRect>,
)

/** PDF highlights by 0-based page; bookmarks are never drawn, and a highlight without rects draws its box. */
fun pdfHighlightsByPage(all: List<HighlightData>): Map<Int, List<PdfHighlightBox>> =
    all
        .asSequence()
        .filter { it.color != BOOKMARK }
        .mapNotNull { highlight ->
            val locator = highlight.locator?.takeIf { it.type == PDF_LOCATOR } ?: return@mapNotNull null
            val page = locator.page ?: return@mapNotNull null
            val rects = locator.drawnRects() ?: return@mapNotNull null
            (page - 1) to PdfHighlightBox(highlight.id, highlight.color, rects)
        }.groupBy({ it.first }, { it.second })

private fun HighlightLocator.drawnRects(): List<NormalizedRect>? {
    val listed = rects.orEmpty().map { NormalizedRect(it.x, it.y, it.width, it.height) }
    val sides = listOfNotNull(x, y, width, height)
    return when {
        listed.isNotEmpty() -> listed
        sides.size == BOX_SIDES -> listOf(NormalizedRect(sides[0], sides[1], sides[2], sides[3]))
        else -> null
    }
}
