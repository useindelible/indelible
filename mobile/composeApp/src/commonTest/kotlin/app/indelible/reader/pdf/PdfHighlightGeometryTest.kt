package app.indelible.reader.pdf

import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightLocator
import app.indelible.reader.model.PdfRect
import kotlinx.datetime.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

private val AT = Instant.parse("2026-04-01T12:00:00Z")

private fun selection(
    text: String = "quick brown",
    rects: List<NormalizedRect> = listOf(NormalizedRect(0.1, 0.1, 0.2, 0.02)),
    pageIndex: Int = 2,
) = PdfSelection(pageIndex, text, 4, 15, rects)

private fun highlight(
    id: String,
    color: String = "yellow",
    locator: HighlightLocator?,
    text: String = "text",
) = HighlightData(id, color, text, locator, emptyList(), AT, AT)

private fun pdfLocator(
    page: Int,
    rects: List<PdfRect>? = listOf(PdfRect(0.1, 0.1, 0.2, 0.02)),
) = HighlightLocator(type = "pdf", page = page, x = 0.1, y = 0.1, width = 0.2, height = 0.02, rects = rects)

class PdfHighlightGeometryTest {
    @Test
    fun rectPastTheRightEdgeIsClippedToTheEdge() {
        val target = checkNotNull(pdfHighlightTarget(selection(rects = listOf(NormalizedRect(0.9, 0.1, 0.3, 0.02)))))

        val rect = target.rects.single()
        assertEquals(0.9, rect.x)
        assertTrue(rect.width > 0.0 && rect.x + rect.width <= 1.0, "clipped to the edge: $rect")
        assertTrue(kotlin.math.abs(rect.width - 0.1) < 1e-9)
    }

    @Test
    fun negativeOriginIsClippedNotShifted() {
        val target = checkNotNull(pdfHighlightTarget(selection(rects = listOf(NormalizedRect(-0.05, 0.1, 0.2, 0.02)))))

        val rect = target.rects.single()
        assertEquals(0.0, rect.x)
        assertTrue(kotlin.math.abs(rect.width - 0.15) < 1e-9, "width $rect")
    }

    @Test
    fun rectStartingPastThePageIsDropped() {
        val rects = listOf(NormalizedRect(1.2, 0.1, 0.2, 0.02), NormalizedRect(0.1, 0.1, 0.2, 0.02))

        val target = checkNotNull(pdfHighlightTarget(selection(rects = rects)))

        assertEquals(1, target.rects.size)
    }

    @Test
    fun nanRectIsDropped() {
        val rects = listOf(NormalizedRect(Double.NaN, 0.1, 0.2, 0.02), NormalizedRect(0.1, 0.1, 0.2, 0.02))

        val target = checkNotNull(pdfHighlightTarget(selection(rects = rects)))

        assertEquals(1, target.rects.size)
    }

    @Test
    fun edgeRectSumsToAtMostOne() {
        for (step in 0..999) {
            val x = step / 1000.0
            val target = checkNotNull(pdfHighlightTarget(selection(rects = listOf(NormalizedRect(x, x, 2.0, 2.0)))))
            val rect = target.rects.single()
            assertTrue(rect.width > 0.0 && rect.height > 0.0, "empty at $x: $rect")
            assertTrue(rect.x + rect.width <= 1.0 - EDGE_MARGIN / 2, "past the edge at $x: $rect")
            assertTrue(rect.y + rect.height <= 1.0 - EDGE_MARGIN / 2, "past the edge at $x: $rect")
            assertTrue(target.x + target.width <= 1.0 - EDGE_MARGIN / 2, "box past the edge at $x")
            assertTrue(target.y + target.height <= 1.0 - EDGE_MARGIN / 2, "box past the edge at $x")
        }
    }

    @Test
    fun edgeRectLeavesRoomForTheServersFloatParse() {
        val x = 0.9417539788507979
        val target = checkNotNull(pdfHighlightTarget(selection(rects = listOf(NormalizedRect(x, 0.1, 0.5, 0.02)))))

        val rect = target.rects.single()
        assertEquals(x, rect.x)
        assertTrue(rect.x + rect.width <= 1.0 - EDGE_MARGIN / 2, "a sum parsing just above 1.0 is refused: $rect")
    }

    @Test
    fun unionBoxIsClippedAfterTheRects() {
        val rects = listOf(NormalizedRect(0.8, 0.1, 0.5, 0.02), NormalizedRect(-0.2, 0.2, 0.5, 0.02))

        val target = checkNotNull(pdfHighlightTarget(selection(rects = rects)))

        assertEquals(0.0, target.x)
        assertTrue(target.x + target.width <= 1.0 && target.width > 0.9, "box $target")
    }

    @Test
    fun blankSelectionMakesNoTarget() {
        assertNull(pdfHighlightTarget(selection(text = "  \n ")))
        assertNull(pdfHighlightTarget(selection(rects = listOf(NormalizedRect(1.5, 1.5, 0.1, 0.1)))))
        assertNull(pdfHighlightTarget(selection(rects = emptyList())))
    }

    @Test
    fun snapshotIsTheFirstHundredCharacters() {
        val text = "x".repeat(250)

        val target = checkNotNull(pdfHighlightTarget(selection(text = text)))

        assertEquals(100, target.textSnapshot.length)
    }

    @Test
    fun textIsTrimmed() {
        val target = checkNotNull(pdfHighlightTarget(selection(text = "  quick \n")))

        assertEquals("quick", target.textSnapshot)
    }

    @Test
    fun pageIsOneBased() {
        assertEquals(3, checkNotNull(pdfHighlightTarget(selection(pageIndex = 2))).page)
    }

    @Test
    fun highlightsAreGroupedByPage() {
        val all =
            listOf(
                highlight("a", locator = pdfLocator(1)),
                highlight("b", locator = pdfLocator(3)),
                highlight("c", locator = pdfLocator(3)),
            )

        val byPage = pdfHighlightsByPage(all)

        assertEquals(setOf(0, 2), byPage.keys)
        assertEquals(listOf("b", "c"), byPage.getValue(2).map { it.highlightId })
        assertEquals(listOf(NormalizedRect(0.1, 0.1, 0.2, 0.02)), byPage.getValue(0).single().rects)
        assertEquals("yellow", byPage.getValue(0).single().color)
    }

    @Test
    fun boxesCarryTheHighlightsWords() {
        val all = listOf(highlight("a", locator = pdfLocator(1), text = "  quick brown fox "))

        assertEquals("quick brown fox", pdfHighlightsByPage(all).getValue(0).single().text)
    }

    @Test
    fun bookmarksAreNeverDrawn() {
        val byPage = pdfHighlightsByPage(listOf(highlight("a", color = "bookmark", locator = pdfLocator(1))))

        assertTrue(byPage.isEmpty())
    }

    @Test
    fun highlightWithoutRectsDrawsItsBox() {
        val byPage = pdfHighlightsByPage(listOf(highlight("a", locator = pdfLocator(1, rects = emptyList()))))

        assertEquals(listOf(NormalizedRect(0.1, 0.1, 0.2, 0.02)), byPage.getValue(0).single().rects)
    }

    @Test
    fun nonPdfHighlightsAreIgnored() {
        val epub = HighlightLocator(type = "epub", startOffset = 1, endOffset = 5, chapter = "e1")

        val byPage = pdfHighlightsByPage(listOf(highlight("a", locator = epub), highlight("b", locator = null)))

        assertTrue(byPage.isEmpty())
    }
}
