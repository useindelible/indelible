package app.indelible.reader.pdf

import kotlin.math.roundToInt
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

private fun heightFor(
    page: PdfPageSize,
    width: Int,
) = (width * page.height / page.width).roundToInt()

class PdfRenderBoundsTest {
    @Test
    fun fitWidthPageKeepsItsWidth() {
        assertEquals(1080, renderWidthFor(PdfPageSize(612.0, 792.0), 1080))
    }

    @Test
    fun tallPageIsNarrowedToFitTheHeightBound() {
        val page = PdfPageSize(300.0, 6000.0)

        val width = renderWidthFor(page, 3072)

        assertEquals(204, width)
        assertTrue(heightFor(page, width) <= MAX_RENDER_HEIGHT_PX)
    }

    @Test
    fun extremeAspectStaysInsideTheHeightBound() {
        val size = renderSizeFor(PdfPageSize(1.0, 5000.0), 3072)

        assertTrue(size.height <= MAX_RENDER_HEIGHT_PX, "height ${size.height}")
        assertTrue(size.width >= 1)
        bufferBytes(size.width, size.height)
    }

    @Test
    fun hugePageIsNarrowedToFitThePixelBound() {
        val page = PdfPageSize(3072.0, 4000.0)

        val width = renderWidthFor(page, 5000)

        assertTrue(width < MAX_RENDER_WIDTH_PX, "width $width")
        assertTrue(width.toLong() * heightFor(page, width) <= MAX_RENDER_PIXELS)
        assertTrue(width.toLong() * heightFor(page, width) > MAX_RENDER_PIXELS * 0.99, "not needlessly small: $width")
    }

    @Test
    fun pixelBoundHoldsAfterTheHeightIsRounded() {
        val size = renderSizeFor(PdfPageSize(2000.0, 2558.0), 5000)

        assertTrue(size.width.toLong() * size.height <= MAX_RENDER_PIXELS, "$size")
        val wider = size.width + 1
        val widerPixels = wider.toLong() * heightFor(PdfPageSize(2000.0, 2558.0), wider)
        assertTrue(widerPixels > MAX_RENDER_PIXELS, "needlessly small: $size")
    }

    @Test
    fun renderSizeFollowsThePageAspect() {
        assertEquals(PdfRenderSize(1080, 1398), renderSizeFor(PdfPageSize(612.0, 792.0), 1080))
        assertEquals(PdfRenderSize(204, 4080), renderSizeFor(PdfPageSize(300.0, 6000.0), 3072))
    }

    @Test
    fun degeneratePageSizeFallsBackToLetter() {
        val letter = PdfPageSize(612.0, 792.0)

        assertEquals(letter, safePageSize(0.0, 0.0))
        assertEquals(letter, safePageSize(Double.NaN, 100.0))
        assertEquals(letter, safePageSize(100.0, Double.POSITIVE_INFINITY))
        assertEquals(letter, safePageSize(-5.0, 100.0))
        assertEquals(PdfPageSize(100.0, 200.0), safePageSize(100.0, 200.0))
    }

    @Test
    fun bufferBytesRefusesSizesOutsideTheBounds() {
        assertEquals(50_331_648, bufferBytes(3072, 4096))
        assertFailsWith<IllegalArgumentException> { bufferBytes(3073, 10) }
        assertFailsWith<IllegalArgumentException> { bufferBytes(10, 4097) }
        assertFailsWith<IllegalArgumentException> { bufferBytes(0, 10) }
    }
}
