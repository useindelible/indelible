package app.indelible.reader.ui.pdf

import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculateCentroid
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import kotlin.math.roundToInt

const val MIN_ZOOM = 1f
const val MAX_ZOOM = 4f

/** The first laid-out page and how many px of it sit above the content's top edge. */
data class LazyAnchor(
    val index: Int,
    val offsetPx: Int,
)

data class ZoomCommit(
    val zoom: Float,
    val anchor: LazyAnchor,
    val horizontalScroll: Int,
)

/** The layout after a pinch of [gestureScale] around [centroid]: the content under the fingers stays there. */
fun commitZoom(
    zoom: Float,
    gestureScale: Float,
    centroid: Offset,
    first: LazyAnchor,
    horizontalScroll: Int,
): ZoomCommit {
    val next = (zoom * gestureScale).coerceIn(MIN_ZOOM, MAX_ZOOM)
    val ratio = next / zoom
    val offset = (ratio * (first.offsetPx + centroid.y) - centroid.y).roundToInt().coerceAtLeast(0)
    val scrolled = (ratio * (horizontalScroll + centroid.x) - centroid.x).roundToInt().coerceAtLeast(0)
    return ZoomCommit(next, LazyAnchor(first.index, offset), if (next == MIN_ZOOM) 0 else scrolled)
}

internal data class PinchPreview(
    val scale: Float,
    val centroid: Offset,
)

@Stable
class PdfZoomState {
    var zoom by mutableFloatStateOf(MIN_ZOOM)
        internal set
    internal var preview by mutableStateOf<PinchPreview?>(null)
}

@Composable
fun rememberPdfZoomState(): PdfZoomState = remember { PdfZoomState() }

/** Follows a two-finger pinch from the first pass, so the lists below never see it as a scroll. */
internal fun Modifier.pinchToZoom(
    state: PdfZoomState,
    onCommit: (scale: Float, centroid: Offset) -> Unit,
): Modifier =
    pointerInput(state) {
        awaitEachGesture {
            awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
            var scale = 1f
            var centroid = Offset.Zero
            var pinched = false
            do {
                val event = awaitPointerEvent(PointerEventPass.Initial)
                if (event.changes.count { it.pressed } >= 2) {
                    pinched = true
                    scale = (scale * event.calculateZoom()).coerceIn(MIN_ZOOM / state.zoom, MAX_ZOOM / state.zoom)
                    centroid = event.calculateCentroid()
                    event.changes.forEach { it.consume() }
                    state.preview = PinchPreview(scale, centroid)
                }
            } while (event.changes.any { it.pressed })
            if (pinched) onCommit(scale, centroid)
        }
    }
