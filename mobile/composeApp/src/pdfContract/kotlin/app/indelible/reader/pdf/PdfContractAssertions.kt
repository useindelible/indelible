package app.indelible.reader.pdf

import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.toPixelMap
import kotlin.math.abs
import kotlin.math.roundToInt
import kotlin.test.assertTrue

private const val CENTRE_TOLERANCE = 0.015
private const val EDGE_TOLERANCE = 0.02
private const val MIN_THICKNESS = 0.5
private const val MAX_THICKNESS = 1.8

/** Centres within [CENTRE_TOLERANCE]; the run edges within [EDGE_TOLERANCE]; the thickness within a ratio. */
internal fun assertRectMatches(
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

internal fun centreX(rect: NormalizedRect) = rect.x + rect.width / 2

internal fun centreY(rect: NormalizedRect) = rect.y + rect.height / 2

/** The mean of the channel means over the pixels inside [rect]. */
internal fun luminance(
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
