package app.indelible.reader.pdf

/** Unrotated page space in points, origin bottom-left, as the PDFKit and pdfium text APIs report. */
data class PageSpaceRect(
    val x: Double,
    val y: Double,
    val width: Double,
    val height: Double,
)

data class PageSpacePoint(
    val x: Double,
    val y: Double,
)

private const val FULL_TURN = 360
private const val QUARTER_TURN = 90
private const val HALF_TURN = 180
private const val THREE_QUARTER_TURN = 270
private const val LINE_OVERLAP = 0.5

/** How far inside the far edges a clipped side ends; the server's float parse can land one ulp above the sum. */
const val EDGE_MARGIN = 1e-9

/** Glyph rects on one text line become one rect, the web's rule: vertical overlap over half the smaller height. */
fun mergeLineRects(rects: List<NormalizedRect>): List<NormalizedRect> {
    val lines = mutableListOf<NormalizedRect>()
    for (rect in rects.filter(::hasArea).sortedWith(compareBy({ it.y }, { it.x }))) {
        val last = lines.lastOrNull()
        if (last != null && overlapY(last, rect) > minOf(last.height, rect.height) * LINE_OVERLAP) {
            lines[lines.lastIndex] = checkNotNull(unionOf(listOf(last, rect)))
        } else {
            lines += rect
        }
    }
    return lines
}

private fun hasArea(rect: NormalizedRect): Boolean =
    rect.x.isFinite() && rect.y.isFinite() && rect.width > 0 && rect.height > 0

private fun overlapY(
    a: NormalizedRect,
    b: NormalizedRect,
): Double = minOf(a.y + a.height, b.y + b.height) - maxOf(a.y, b.y)

/** The part of [rect] inside the page, or null when nothing is left or a side is NaN. */
fun clipToPage(rect: NormalizedRect): NormalizedRect? {
    val left = maxOf(0.0, rect.x)
    val top = maxOf(0.0, rect.y)
    val right = minOf(1.0 - EDGE_MARGIN, rect.x + rect.width)
    val bottom = minOf(1.0 - EDGE_MARGIN, rect.y + rect.height)
    if (!(right > left && bottom > top)) return null
    return NormalizedRect(left, top, right - left, bottom - top)
}

fun unionOf(rects: List<NormalizedRect>): NormalizedRect? {
    if (rects.isEmpty()) return null
    val left = rects.minOf { it.x }
    val top = rects.minOf { it.y }
    val right = rects.maxOf { it.x + it.width }
    val bottom = rects.maxOf { it.y + it.height }
    return NormalizedRect(left, top, right - left, bottom - top)
}

/** [rotation] in degrees clockwise as displayed (multiples of 90, normalised mod 360). */
fun fromPageSpace(
    rect: PageSpaceRect,
    crop: PageSpaceRect,
    rotation: Int,
): NormalizedRect {
    val a = toDisplay(PageSpacePoint(rect.x, rect.y), crop, rotation)
    val b = toDisplay(PageSpacePoint(rect.x + rect.width, rect.y + rect.height), crop, rotation)
    val left = minOf(a.x, b.x)
    val top = minOf(a.y, b.y)
    return NormalizedRect(left, top, maxOf(a.x, b.x) - left, maxOf(a.y, b.y) - top)
}

fun toPageSpace(
    point: NormalizedPoint,
    crop: PageSpaceRect,
    rotation: Int,
): PageSpacePoint {
    val (u, t) =
        when (normalised(rotation)) {
            QUARTER_TURN -> point.y to 1 - point.x
            HALF_TURN -> 1 - point.x to 1 - point.y
            THREE_QUARTER_TURN -> 1 - point.y to point.x
            else -> point.x to point.y
        }
    return PageSpacePoint(crop.x + u * crop.width, crop.y + (1 - t) * crop.height)
}

private fun toDisplay(
    point: PageSpacePoint,
    crop: PageSpaceRect,
    rotation: Int,
): NormalizedPoint {
    val u = (point.x - crop.x) / crop.width
    val t = 1 - (point.y - crop.y) / crop.height
    return when (normalised(rotation)) {
        QUARTER_TURN -> NormalizedPoint(1 - t, u)
        HALF_TURN -> NormalizedPoint(1 - u, 1 - t)
        THREE_QUARTER_TURN -> NormalizedPoint(t, 1 - u)
        else -> NormalizedPoint(u, t)
    }
}

private fun normalised(rotation: Int): Int = ((rotation % FULL_TURN) + FULL_TURN) % FULL_TURN
