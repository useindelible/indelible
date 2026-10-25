package app.indelible.reader.ui.pdf

import androidx.compose.ui.geometry.Offset
import kotlin.test.Test
import kotlin.test.assertEquals

class PdfZoomTest {
    private val centroid = Offset(100f, 200f)
    private val top = LazyAnchor(0, 0)

    private fun commit(
        zoom: Float,
        scale: Float,
        first: LazyAnchor,
        horizontalScroll: Int = 0,
        focus: Offset = centroid,
    ) = commitZoom(zoom, scale, focus, first, horizontalScroll)

    @Test
    fun zoomIsClampedBetweenFitWidthAndFourTimes() {
        val shrunk = commit(zoom = 1f, scale = 0.5f, first = top)
        val grown = commit(zoom = 3f, scale = 2f, first = top)

        assertEquals(MIN_ZOOM, shrunk.zoom)
        assertEquals(MAX_ZOOM, grown.zoom)
    }

    @Test
    fun commitKeepsTheFocalPointUnderTheFingers() {
        val commit = commit(zoom = 1f, scale = 2f, first = LazyAnchor(3, 50))

        assertEquals(2f, commit.zoom)
        assertEquals(100, commit.horizontalScroll)
        assertEquals(LazyAnchor(3, 300), commit.anchor)
    }

    @Test
    fun zoomingBackToFitWidthResetsHorizontalScroll() {
        val commit = commit(zoom = 2f, scale = 0.5f, first = LazyAnchor(3, 300), horizontalScroll = 150)

        assertEquals(1f, commit.zoom)
        assertEquals(0, commit.horizontalScroll)
        assertEquals(LazyAnchor(3, 50), commit.anchor)
    }

    @Test
    fun focalPointAboveTheAnchorPageStaysOnIt() {
        val commit = commit(zoom = 2f, scale = 0.5f, first = LazyAnchor(3, 0), focus = Offset(0f, 10f))

        assertEquals(LazyAnchor(3, 0), commit.anchor)
    }
}
