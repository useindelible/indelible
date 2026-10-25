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
    fun hugePageIsNarrowedToFitThePixelBound() {
        val page = PdfPageSize(3072.0, 4000.0)

        val width = renderWidthFor(page, 5000)

        assertTrue(width < MAX_RENDER_WIDTH_PX, "width $width")
        assertTrue(width.toLong() * heightFor(page, width) <= MAX_RENDER_PIXELS)
        assertTrue(width.toLong() * heightFor(page, width) > MAX_RENDER_PIXELS * 0.99, "not needlessly small: $width")
    }

    @Test
    fun bufferBytesRefusesSizesOutsideTheBounds() {
        assertEquals(50_331_648, bufferBytes(3072, 4096))
        assertFailsWith<IllegalArgumentException> { bufferBytes(3073, 10) }
        assertFailsWith<IllegalArgumentException> { bufferBytes(10, 4097) }
        assertFailsWith<IllegalArgumentException> { bufferBytes(0, 10) }
    }
}
