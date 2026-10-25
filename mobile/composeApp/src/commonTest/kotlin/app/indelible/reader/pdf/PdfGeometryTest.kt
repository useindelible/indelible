package app.indelible.reader.pdf

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

private const val EPS = 1e-9
private val LETTER = PageSpaceRect(0.0, 0.0, 612.0, 792.0)

private fun assertClose(
    expected: NormalizedRect,
    actual: NormalizedRect,
) {
    assertTrue(
        kotlin.math.abs(expected.x - actual.x) < EPS &&
            kotlin.math.abs(expected.y - actual.y) < EPS &&
            kotlin.math.abs(expected.width - actual.width) < EPS &&
            kotlin.math.abs(expected.height - actual.height) < EPS,
        "expected $expected but was $actual",
    )
}

private fun assertClose(
    expected: NormalizedPoint,
    actual: NormalizedPoint,
) {
    assertTrue(
        kotlin.math.abs(expected.x - actual.x) < EPS && kotlin.math.abs(expected.y - actual.y) < EPS,
        "expected $expected but was $actual",
    )
}

class PdfGeometryTest {
    @Test
    fun rectsOnOneLineMergeIntoOne() {
        val merged =
            mergeLineRects(
                listOf(
                    NormalizedRect(0.3, 0.10, 0.1, 0.02),
                    NormalizedRect(0.1, 0.10, 0.1, 0.02),
                    NormalizedRect(0.2, 0.105, 0.1, 0.02),
                ),
            )

        assertEquals(1, merged.size)
        assertClose(NormalizedRect(0.1, 0.10, 0.3, 0.025), merged.single())
    }

    @Test
    fun rectsOverlappingLessThanHalfStaySeparate() {
        val merged =
            mergeLineRects(
                listOf(
                    NormalizedRect(0.1, 0.10, 0.1, 0.02),
                    NormalizedRect(0.1, 0.112, 0.1, 0.02),
                ),
            )

        assertEquals(2, merged.size)
    }

    @Test
    fun linesAreOrderedTopToBottom() {
        val merged =
            mergeLineRects(
                listOf(
                    NormalizedRect(0.1, 0.50, 0.2, 0.02),
                    NormalizedRect(0.1, 0.10, 0.2, 0.02),
                    NormalizedRect(0.1, 0.30, 0.2, 0.02),
                ),
            )

        assertEquals(listOf(0.10, 0.30, 0.50), merged.map { it.y })
    }

    @Test
    fun unrotatedPageFlipsY() {
        val rect = fromPageSpace(PageSpaceRect(61.2, 712.8, 122.4, 39.6), LETTER, 0)

        assertClose(NormalizedRect(0.1, 0.05, 0.2, 0.05), rect)
    }

    @Test
    fun quarterTurnPageSwapsAxes() {
        val rect = fromPageSpace(PageSpaceRect(61.2, 712.8, 122.4, 39.6), LETTER, 90)

        assertClose(NormalizedRect(0.9, 0.1, 0.05, 0.2), rect)
    }

    @Test
    fun halfTurnPageMirrorsBoth() {
        val rect = fromPageSpace(PageSpaceRect(61.2, 712.8, 122.4, 39.6), LETTER, 180)

        assertClose(NormalizedRect(0.7, 0.9, 0.2, 0.05), rect)
    }

    @Test
    fun threeQuarterTurnPage() {
        val rect = fromPageSpace(PageSpaceRect(61.2, 712.8, 122.4, 39.6), LETTER, 270)

        assertClose(NormalizedRect(0.05, 0.7, 0.05, 0.2), rect)
    }

    @Test
    fun cropBoxOriginIsSubtracted() {
        val crop = PageSpaceRect(50.0, 100.0, 500.0, 600.0)

        val rect = fromPageSpace(PageSpaceRect(100.0, 640.0, 50.0, 60.0), crop, 0)

        assertClose(NormalizedRect(0.1, 0.0, 0.1, 0.1), rect)
    }

    @Test
    fun pointRoundTripsThroughPageSpaceAtEveryRotation() {
        val crop = PageSpaceRect(50.0, 100.0, 500.0, 600.0)
        val point = NormalizedPoint(0.3, 0.8)

        for (rotation in listOf(0, 90, 180, 270)) {
            val page = toPageSpace(point, crop, rotation)
            val back = fromPageSpace(PageSpaceRect(page.x, page.y, 0.0, 0.0), crop, rotation)
            assertClose(point, NormalizedPoint(back.x, back.y))
        }
    }

    @Test
    fun negativeRotationIsNormalised() {
        val source = PageSpaceRect(61.2, 712.8, 122.4, 39.6)

        assertEquals(fromPageSpace(source, LETTER, 270), fromPageSpace(source, LETTER, -90))
        assertEquals(fromPageSpace(source, LETTER, 90), fromPageSpace(source, LETTER, 450))
    }

    @Test
    fun unionOfRectsIsTheirBoundingBox() {
        val union = unionOf(listOf(NormalizedRect(0.1, 0.2, 0.2, 0.1), NormalizedRect(0.5, 0.1, 0.1, 0.3)))

        assertClose(NormalizedRect(0.1, 0.1, 0.5, 0.3), checkNotNull(union))
        assertEquals(null, unionOf(emptyList()))
    }
}
