package app.indelible.reader.pdf

import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.toPixelMap
import app.indelible.reader.pdf.PdfFixtures.expectedRect
import app.indelible.reader.pdf.PdfFixtures.pointIn
import kotlinx.coroutines.test.runTest
import kotlin.math.abs
import kotlin.math.roundToInt
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

private const val CENTRE_TOLERANCE = 0.015
private const val EDGE_TOLERANCE = 0.02
private const val MIN_THICKNESS = 0.5
private const val MAX_THICKNESS = 1.8
private const val WHITE = 0xFFFFFFFF.toInt()
private const val DARK_PAGE = 0xFF1C1C1C.toInt()
private const val SEPIA_PAGE = 0xFFF5EDDA.toInt()
private const val FULL_CHANNEL = 250
private const val EMPTY_CHANNEL = 5
private const val INK_CONTRAST = 10
private const val NEAR_WHITE = 250.0

class PdfEngineContractTest {
    private suspend fun <T> open(
        name: String,
        pages: List<FixturePage>,
        block: suspend (PdfDocument) -> T,
    ): T = PdfEngine.open(PdfFixtures.write(name, pages)).use { block(it) }

    private val plain = PdfFixtures.plain.single()
    private val firstLine = plain.lines[0]
    private val secondLine = plain.lines[1]
    private val quick = pointIn(plain, firstLine, 4..8)
    private val fox = pointIn(plain, firstLine, 16..18)

    @Test
    fun opensAndCountsPages() =
        runTest {
            open("multi", PdfFixtures.multi) { assertEquals(3, it.pageCount) }
        }

    @Test
    fun pageSizesFollowTheCropBoxAndRotation() =
        runTest {
            open("plain", PdfFixtures.plain) { assertSize(PdfPageSize(612.0, 792.0), it.pageSizes[0]) }
            open("rotated", PdfFixtures.rotated) { assertSize(PdfPageSize(792.0, 612.0), it.pageSizes[0]) }
            open("cropped", PdfFixtures.cropped) { assertSize(PdfPageSize(512.0, 592.0), it.pageSizes[0]) }
            open("multi", PdfFixtures.multi) { assertSize(PdfPageSize(792.0, 612.0), it.pageSizes[1]) }
        }

    @Test
    fun textReadsThePageInOrder() =
        runTest {
            open("plain", PdfFixtures.plain) { document ->
                val text = document.text(0)
                val first = text.indexOf("The quick brown fox")
                val second = text.indexOf("jumps over the lazy dog")
                assertTrue(first in 0 until second, "text: $text")
            }
        }

    @Test
    fun dragSelectionReturnsTheWordsAndOneLineRect() =
        runTest {
            open("plain", PdfFixtures.plain) { document ->
                val selection = assertNotNull(document.select(0, quick, fox))
                assertEquals("quick brown fox", selection.text.trim())
                assertTrue(selection.endChar > selection.startChar)
                assertRectMatches(expectedRect(plain, firstLine, 4..18), selection.rects.single(), plain.rotation)
            }
        }

    @Test
    fun twoLineSelectionGivesOneRectPerLine() =
        runTest {
            open("plain", PdfFixtures.plain) { document ->
                val selection = assertNotNull(document.select(0, quick, pointIn(plain, secondLine, 6..9)))
                assertEquals(2, selection.rects.size, "${selection.rects}")
                assertRectMatches(expectedRect(plain, firstLine, 4..18), selection.rects[0], plain.rotation)
                assertRectMatches(expectedRect(plain, secondLine, 0..9), selection.rects[1], plain.rotation)
            }
        }

    @Test
    fun selectionOnARotatedPageIsInDisplayCoordinates() =
        runTest {
            val page = PdfFixtures.rotated.single()
            open("rotated", PdfFixtures.rotated) { document ->
                val line = page.lines[0]
                val selection =
                    assertNotNull(document.select(0, pointIn(page, line, 4..8), pointIn(page, line, 16..18)))
                assertEquals("quick brown fox", selection.text.trim())
                assertRectMatches(expectedRect(page, line, 4..18), selection.rects.single(), page.rotation)
            }
        }

    @Test
    fun selectionOnACroppedPageIsRelativeToTheCropBox() =
        runTest {
            val page = PdfFixtures.cropped.single()
            open("cropped", PdfFixtures.cropped) { document ->
                val line = page.lines[0]
                val selection =
                    assertNotNull(document.select(0, pointIn(page, line, 0..6), pointIn(page, line, 8..12)))
                assertEquals("Cropped words", selection.text.trim())
                assertRectMatches(expectedRect(page, line), selection.rects.single(), page.rotation)
            }
        }

    @Test
    fun rangeRectsMatchTheDragSelection() =
        runTest {
            open("plain", PdfFixtures.plain) { document ->
                val selection = assertNotNull(document.select(0, quick, fox))
                val rects = document.rectsForRange(0, selection.startChar, selection.endChar)
                assertEquals(1, rects.size)
                assertRectMatches(selection.rects.single(), rects.single(), plain.rotation)
            }
        }

    @Test
    fun dragPastThePageEdgeStaysInsideThePage() =
        runTest {
            open("plain", PdfFixtures.plain) { document ->
                val selection = assertNotNull(document.select(0, quick, NormalizedPoint(1.2, 1.5)))
                assertTrue(selection.rects.isNotEmpty())
                selection.rects.forEach(::assertInsidePage)
            }
        }

    @Test
    fun dragEndingInTheBlankMarginKeepsTheTextBeforeIt() =
        runTest {
            open("plain", PdfFixtures.plain) { document ->
                val selection = assertNotNull(document.select(0, quick, NormalizedPoint(0.9, 0.95)))
                assertTrue(selection.text.trim().endsWith("dog"), selection.text)
                assertTrue(selection.endChar > selection.startChar)
                selection.rects.forEach(::assertInsidePage)
            }
        }

    @Test
    fun dragIntoTheMarginBesideALineStopsAtItsEnd() =
        runTest {
            open("plain", PdfFixtures.plain) { document ->
                val selection = assertNotNull(document.select(0, quick, NormalizedPoint(0.95, quick.y - 0.001)))
                assertEquals("quick brown fox", selection.text.trim())
                assertTrue(selection.rects.isNotEmpty())
                selection.rects.forEach(::assertInsidePage)
            }
        }

    @Test
    fun blankMarginSelectsNothing() =
        runTest {
            open("plain", PdfFixtures.plain) { document ->
                assertNull(document.select(0, NormalizedPoint(0.5, 0.6), NormalizedPoint(0.6, 0.7)))
            }
        }

    @Test
    fun searchFindsTheWordOnEveryPage() =
        runTest {
            open("multi", PdfFixtures.multi) { document ->
                val hits = document.search("word")
                assertEquals(listOf(0, 1, 2), hits.map { it.pageIndex }.sorted())
                hits.forEach { hit ->
                    assertTrue(hit.endChar > hit.startChar)
                    hit.rects.forEach(::assertInsidePage)
                }
            }
        }

    @Test
    fun renderHasTheRequestedSizeAndTone() =
        runTest {
            open("plain", PdfFixtures.plain) { document ->
                val light = document.render(0, 300, PdfTone.NONE)
                assertEquals(300, light.width)
                assertTrue(abs(light.height - 300 * 792 / 612) <= 1, "height ${light.height}")
                assertEquals(WHITE, light.toPixelMap()[2, 2].toArgb())
                val dark = document.render(0, 300, PdfTone.DARK)
                assertEquals(DARK_PAGE, dark.toPixelMap()[2, 2].toArgb())
            }
        }

    @Test
    fun renderKeepsTheChannelOrder() =
        runTest {
            val page = PdfFixtures.swatch.single()
            open("swatch", PdfFixtures.swatch) { document ->
                val sepia = document.render(0, 300, PdfTone.SEPIA)
                assertEquals(SEPIA_PAGE, sepia.toPixelMap()[2, 2].toArgb())
                val light = document.render(0, 300, PdfTone.NONE)
                val square = fromPageSpace(checkNotNull(page.redSwatch), page.box, page.rotation)
                val x = ((square.x + square.width / 2) * light.width).toInt()
                val y = ((square.y + square.height / 2) * light.height).toInt()
                val argb = light.toPixelMap()[x, y].toArgb()
                val (red, green, blue) = listOf((argb shr 16) and 0xFF, (argb shr 8) and 0xFF, argb and 0xFF)
                assertTrue(red >= FULL_CHANNEL && green <= EMPTY_CHANNEL && blue <= EMPTY_CHANNEL, "pixel $argb")
            }
        }

    @Test
    fun rangeOutsideThePageIsClampedToIt() =
        runTest {
            open("plain", PdfFixtures.plain) { document ->
                val clamped = document.rectsForRange(0, -5, 3)
                assertEquals(1, clamped.size)
                clamped.forEach(::assertInsidePage)
                assertTrue(document.rectsForRange(0, 10_000, 10_010).isEmpty())
            }
        }

    @Test
    fun renderPutsInkWhereTheTextRectsSay() =
        runTest {
            val fixtures =
                listOf("plain" to PdfFixtures.plain, "rotated" to PdfFixtures.rotated, "cropped" to PdfFixtures.cropped)
            for ((name, pages) in fixtures) {
                val page = pages.single()
                open(name, pages) { document ->
                    val bitmap = document.render(0, 600, PdfTone.NONE)
                    page.lines.forEach { line ->
                        val rect = expectedRect(page, line)
                        assertTrue(luminance(bitmap, rect) < NEAR_WHITE - INK_CONTRAST, "$name: no ink in $rect")
                    }
                    val margin = luminance(bitmap, NormalizedRect(0.6, 0.6, 0.3, 0.3))
                    assertTrue(margin >= NEAR_WHITE, "$name: ink in the margin")
                }
            }
        }

    @Test
    fun tallPageRendersWithinTheBitmapBound() =
        runTest {
            val page = PdfFixtures.tall.single()
            open("tall", PdfFixtures.tall) { document ->
                val bitmap = document.render(0, MAX_RENDER_WIDTH_PX, PdfTone.NONE)
                assertTrue(bitmap.height <= MAX_RENDER_HEIGHT_PX)
                assertTrue(bitmap.width.toLong() * bitmap.height <= MAX_RENDER_PIXELS)
                val aspect = bitmap.height.toDouble() / bitmap.width
                assertTrue(abs(aspect - 20.0) / 20.0 <= 0.01, "aspect $aspect")
                assertTrue(luminance(bitmap, expectedRect(page, page.lines[1])) < NEAR_WHITE - INK_CONTRAST)
            }
        }

    @Test
    fun damagedFileIsRefused() =
        runTest {
            val path = PdfFixtures.writeBytes("damaged", Random.nextBytes(2048))
            assertFailsWith<PdfOpenException> { PdfEngine.open(path) }
        }

    private fun assertSize(
        expected: PdfPageSize,
        actual: PdfPageSize,
    ) {
        val close = abs(expected.width - actual.width) < 0.5 && abs(expected.height - actual.height) < 0.5
        assertTrue(close, "expected $expected, was $actual")
    }

    private fun assertInsidePage(rect: NormalizedRect) {
        assertTrue(rect.x >= 0.0 && rect.y >= 0.0 && rect.width > 0.0 && rect.height > 0.0, "$rect")
        assertTrue(rect.x + rect.width <= 1.0 && rect.y + rect.height <= 1.0, "$rect")
    }

    /** Centres within [CENTRE_TOLERANCE]; the run edges within [EDGE_TOLERANCE]; the thickness within a ratio. */
    private fun assertRectMatches(
        expected: NormalizedRect,
        actual: NormalizedRect,
        rotation: Int,
    ) {
        val message = "expected $expected, was $actual"
        assertTrue(abs(centreX(expected) - centreX(actual)) <= CENTRE_TOLERANCE, message)
        assertTrue(abs(centreY(expected) - centreY(actual)) <= CENTRE_TOLERANCE, message)
        val runsAcross = rotation % 180 == 0
        val runEdges =
            if (runsAcross) {
                listOf(expected.x to actual.x, expected.x + expected.width to actual.x + actual.width)
            } else {
                listOf(expected.y to actual.y, expected.y + expected.height to actual.y + actual.height)
            }
        val thickness = if (runsAcross) actual.height / expected.height else actual.width / expected.width
        runEdges.forEach { (e, a) -> assertTrue(abs(e - a) <= EDGE_TOLERANCE, message) }
        assertTrue(thickness in MIN_THICKNESS..MAX_THICKNESS, message)
    }

    private fun centreX(rect: NormalizedRect) = rect.x + rect.width / 2

    private fun centreY(rect: NormalizedRect) = rect.y + rect.height / 2

    /** The mean of the channel means over the pixels inside [rect]. */
    private fun luminance(
        bitmap: ImageBitmap,
        rect: NormalizedRect,
    ): Double {
        val pixels = bitmap.toPixelMap()
        val left = (rect.x * bitmap.width).roundToInt().coerceIn(0, bitmap.width - 1)
        val top = (rect.y * bitmap.height).roundToInt().coerceIn(0, bitmap.height - 1)
        val right = ((rect.x + rect.width) * bitmap.width).roundToInt().coerceIn(left + 1, bitmap.width)
        val bottom = ((rect.y + rect.height) * bitmap.height).roundToInt().coerceIn(top + 1, bitmap.height)
        var sum = 0L
        for (y in top until bottom) {
            for (x in left until right) {
                val argb = pixels[x, y].toArgb()
                sum += ((argb shr 16) and 0xFF) + ((argb shr 8) and 0xFF) + (argb and 0xFF)
            }
        }
        return sum.toDouble() / (3L * (right - left) * (bottom - top))
    }
}
