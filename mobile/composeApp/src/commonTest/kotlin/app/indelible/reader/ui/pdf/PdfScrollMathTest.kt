package app.indelible.reader.ui.pdf

import app.indelible.reader.pdf.PdfPageSize
import kotlin.test.Test
import kotlin.test.assertEquals

class PdfScrollMathTest {
    private fun sample(
        index: Int,
        offsetPx: Int,
        lastPageFullyShown: Boolean = false,
        scrollTopDp: Float = 0f,
    ) = sampleOf(PdfTopItem(index, offsetPx, heightPx = 600), pageCount = 20, lastPageFullyShown, scrollTopDp)

    @Test
    fun sampleNamesTheTopPageAndHowFarIntoIt() {
        val sample = sample(index = 2, offsetPx = 150, scrollTopDp = 942f)

        assertEquals(3, sample.page)
        assertEquals(0.25f, sample.pageOffset)
        assertEquals(942f, sample.scrollTopDp)
    }

    @Test
    fun fractionCountsWholePagesBefore() {
        val sample = sample(index = 2, offsetPx = 150)

        assertEquals(0.1125f, sample.fraction)
    }

    @Test
    fun lastPageFullyShownIsTheEnd() {
        val sample = sample(index = 18, offsetPx = 500, lastPageFullyShown = true)

        assertEquals(1f, sample.fraction)
        assertEquals(19, sample.page)
    }

    @Test
    fun restoreOffsetIsTheFractionOfTheLaidOutPageHeight() {
        assertEquals(420, restoreOffsetPx(pageOffset = 0.42f, pageHeightPx = 1000))
    }

    @Test
    fun pageHeightFollowsTheAspectAtTheLaidOutWidth() {
        assertEquals(388, pageHeightPx(PdfPageSize(612.0, 792.0), widthPx = 300))
        assertEquals(1, pageHeightPx(PdfPageSize(612.0, 1.0), widthPx = 300))
    }
}
